# M-Pesa Daraja Gateway

Safaricom merchant payment API for C2B customer-to-business payments (Paybill) and B2C bulk payouts. Handles nomination fee collection, HR payroll distribution, and citizen verification callbacks.

## MoU Status

Daraja commercial agreement with Safaricom (signed). NDA covers shortcode licensing (shortcode 123456) and monthly subscription. Currently valid for production use.

## Endpoint

Production: https://api.safaricom.co.ke/mpesa/
Sandbox: https://sandbox.safaricom.co.ke/mpesa/

## Protocol and Authentication

REST over HTTPS with JSON payloads. OAuth2 client-credentials flow for token generation:

```
POST https://api.safaricom.co.ke/oauth/v1/generate
Authorization: Basic base64(consumer_key:consumer_secret)
grant_type=client_credentials
```

Token TTL: 3600 seconds. Adapter caches and auto-refreshes.

## Request and Response Shape

**C2B Callback (incoming):**
`{TransactionType, TransID, TransTime, TransAmount, BusinessShortCode, BillRefNumber, MSISDN, FirstName, LastName, MiddleName}`

**STK Push (outgoing):**
`{BusinessShortCode, Password, Timestamp, TransactionType, Amount, PartyA, PartyB, PhoneNumber, CallBackURL, AccountReference, TransactionDesc}`

**STK Push Response:**
`{MerchantRequestID, CheckoutRequestID, ResponseCode, ResponseDescription, CustomerMessage}`

**B2C Callback (result):**
`{Result: {ResultType, ResultCode, ResultDesc, OriginatorConversationID, ConversationID, TransactionID, ResultParameters}}`

## Sample Curl

```bash
# OAuth token request
curl -u "$MPESA_CONSUMER_KEY:$MPESA_CONSUMER_SECRET" \
  -d "grant_type=client_credentials" \
  https://api.safaricom.co.ke/oauth/v1/generate

# STK push
curl -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"BusinessShortCode":"123456","Password":"encoded","Timestamp":"20241321120000","TransactionType":"CustomerPayBillOnline","Amount":"50","PartyA":"254700000000","PartyB":"123456","PhoneNumber":"254700000000","CallBackURL":"https://eel.iebc.or.ke/mpesa/callback","AccountReference":"M47-FEE-001","TransactionDesc":"Nomination fee"}' \
  https://api.safaricom.co.ke/mpesa/stkpush/v1/processrequest
```

## Health Probe

GET to OAuth token endpoint with dummy credentials. Returns 401 if gateway is reachable.

## Rate Limits

C2B callback: 1000 requests/minute per shortcode.
B2C transaction: 5000 transactions/day per employer (configurable via Safaricom).
STK push: synchronous with 180s client timeout.

## PII Enforcement

MSISDN and names are medium sensitivity. Logs mask phone numbers and redact full names. Response callbacks are stripped of customer PII. No phone or name storage in logs beyond request correlation.

## Chaos Toggle

MPESA_CHAOS=true enables injection in double (test only). Causes random delays and 5% error rate.

## Fallback Behavior

On adapter unavailability or timeout, payment is queued in m47_payment_pending. Manual reconciliation job (hourly) fetches status from Daraja and reconciles. Operator can manually resolve via admin UI.

## Circuit Breaker Policy

Threshold: 5 consecutive failures
Open duration: 30 seconds
Rationale: prevents duplicate callback retry storms during Daraja slowdown
Half-open: permits 1 probe request before reset

## Environment Variables

MPESA_ENV: "sandbox" or "production" (default: sandbox)
MPESA_CONSUMER_KEY: OAuth consumer key
MPESA_CONSUMER_SECRET: OAuth consumer secret
MPESA_SHORTCODE: Business short code (123456)
MPESA_PASSKEY: Online passkey for STK push
MPESA_CALLBACK_URL: Public HTTPS URL for C2B callbacks
MPESA_TIMEOUT_SEC: Request timeout (default: 30)
MPESA_CHAOS: "true" enables chaos (sandbox only)

## Related Modules

M47 (nomination-fees), M55 (hr-payroll), M13 (citizen-verify)

## Runbook

docs/runbooks/mpesa-daraja-reconciliation.md (manual reconciliation, dispute resolution)

