# M-Pesa B2C Short-Code Provisioning (G12)

## Safaricom Business Paybill Configuration

| Field | Value |
|---|---|
| Short Code | 20822 (dedicated to IEBC election results) |
| Service | USSD (*384#) + SMS query (20822) |
| Provider | Safaricom (primary), Airtel (backup) |
| Agreement | 24-month contract |
| SLA | 99.9% uptime, <2s response time |
| Monthly Fee | KES 85,000 (primary); KES 65,000 (backup) |
| Integration | REST + SMPP |
| M5 Audit | every SMS event (query/response) emits auditId=g22-sms |

## Daraja API Integration

The M-Pesa B2C adapter (`integrations/mpesa-b2c/adapter.py`) connects to Safaricom Daraja API for poll-worker stipend payments.

### Configuration

```python
from integrations.mpesa_b2c.adapter import MpesaB2CConfig, PaymentRequest, initiate_payment

config = MpesaB2CConfig(
    consumer_key="<safaricom-consumer-key>",
    consumer_secret="<safaricom-consumer-secret>",
    initiator_name="IEBC_Election_2026",
    security_credential="<encrypted-password>",
    shortcode=20822,
    result_url="https://api.eel.internal/mpesa/callback",
    queue_timeout_url="https://api.eel.internal/mpesa/timeout",
)

payment = PaymentRequest(
    worker_id="WORKER-001",
    national_id="<national-id>",
    amount=2000,
    remark="Poll worker stipend",
)

result = initiate_payment(config, payment, token)
```

### Payment Result

```python
result.conversation_id   # e.g., "AG.20260821.1234567890"
result.originator_conversation_id  # e.g., "AG.20260821.1234567890"
result.response_code       # "0" = success, "1" = failed, "17" = insufficient funds
result.response_description  # human-readable message
result.audit_id            # auditId=g12-mpesa-{conversation_id}
```

## Verification

```bash
# Verify payment
python integrations/mpesa-b2c/adapter.py verify <transaction_id> <token>

# Check callback
curl -X POST https://api.eel.internal/mpesa/callback \
  -H "Content-Type: application/json" \
  -d '{"Result": {"ResultCode": 0, "TransactionID": "AG.2026..."}}'
```
