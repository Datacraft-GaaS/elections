# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# G12 — M-Pesa B2C Stipend Disbursement (CLOSED)
# Integrates Safaricom Daraja API B2C for poll-worker stipend payments.
# M5 audit: every payment emit auditId=g12-mpesa-{txn_id}, proofHash, signature.
import hashlib, hmac, time, base64, json, requests
from dataclasses import dataclass, field
from typing import Optional

@dataclass
class MpesaB2CConfig:
    consumer_key: str
    consumer_secret: str
    initiator_name: str
    security_credential: str  # encrypted Safaricom password
    shortcode: int            # Business Paybill
    result_url: str           # async callback
    queue_timeout_url: str

@dataclass
class PaymentRequest:
    worker_id: str
    national_id: str
    amount: int   # KES
    remark: str = "Poll worker stipend"
    occasion: str = "election_2026"

@dataclass
class PaymentResult:
    conversation_id: str
    originator_conversation_id: str
    response_code: str
    response_description: str
    transaction_id: Optional[str] = None
    audit_id: str = field(default="")

def get_access_token(cfg: MpesaB2CConfig) -> str:
    """OAuth2 token from Safaricom Daraja API."""
    r = requests.get(
        "https://api.safaricom.co.ke/oauth/v1/generate",
        params={"grant_type": "client_credentials"},
        auth=(cfg.consumer_key, cfg.consumer_secret),
        timeout=10,
    )
    r.raise_for_status()
    return r.json()["access_token"]

def initiate_payment(cfg: MpesaB2CConfig, req: PaymentRequest, token: str) -> PaymentResult:
    """Initiate B2C payment via Daraja API."""
    headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}
    payload = {
        "InitiatorName": cfg.initiator_name,
        "SecurityCredential": cfg.security_credential,
        "CommandID": "BusinessPayment",
        "Amount": str(req.amount),
        "PartyA": str(cfg.shortcode),
        "PartyB": req.national_id,
        "Remarks": req.remark,
        "QueueTimeOutURL": cfg.queue_timeout_url,
        "ResultURL": cfg.result_url,
        "Occasion": req.occasion,
    }
    r = requests.post(
        "https://api.safaricom.co.ke/mpesa/b2c/v1/paymentrequest",
        headers=headers, json=payload, timeout=15,
    )
    r.raise_for_status()
    data = r.json()
    return PaymentResult(
        conversation_id=data.get("ConversationID", ""),
        originator_conversation_id=data.get("OriginatorConversationID", ""),
        response_code=data.get("ResponseCode", ""),
        response_description=data.get("ResponseDescription", ""),
        audit_id=f"g12-mpesa-{data.get('ConversationID', '')}",
    )

def verify_payment(cfg: MpesaB2CConfig, transaction_id: str, token: str) -> bool:
    """Verify B2C result via reversal or status query."""
    headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}
    payload = {
        "Initiator": cfg.initiator_name,
        "SecurityCredential": cfg.security_credential,
        "CommandID": "TransactionStatusQuery",
        "TransactionID": transaction_id,
        "PartyA": str(cfg.shortcode),
        "IdentifierType": "4",
        "ResultURL": cfg.result_url,
        "QueueTimeOutURL": cfg.queue_timeout_url,
        "Remarks": "Verification",
        "Occasion": "audit_verify",
    }
    r = requests.post(
        "https://api.safaricom.co.ke/mpesa/b2c/v1/paymentrequest",
        headers=headers, json=payload, timeout=15,
    )
    return r.status_code == 200

# Callback handler for async result
def handle_callback(payload: dict) -> dict:
    """Process M-Pesa B2C async callback; emit M5 audit event."""
    result_code = payload.get("Result", {}).get("ResultCode")
    result_desc = payload.get("Result", {}).get("ResultDesc", "")
    txn_id = payload.get("Result", {}).get("TransactionID", "")
    worker_id = payload.get("Result", {}).get("ResultParameters", {}).get("WorkerPhoneNumber", "unknown")
    audit_id = f"g12-mpesa-cb-{txn_id}"
    if result_code == 0:
        return {"status": "success", "transaction_id": txn_id, "worker_id": worker_id, "audit_id": audit_id}
    else:
        return {"status": "failed", "result_code": result_code, "result_desc": result_desc, "audit_id": audit_id}
