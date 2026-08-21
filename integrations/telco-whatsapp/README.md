# integrations/telco-whatsapp

Telco-managed WhatsApp Business API fallback (separate from Meta direct API).

**Status: stub - MoU / integration spec pending.**

## Purpose

This integration will provide WhatsApp messaging via Safaricom WhatsApp Business gateway as a fallback to direct Meta WhatsApp API. This supports M37 (voter-education-cms) for multilingual voter education delivery and M23 (grievance-evidence) for grievance acknowledgment over WhatsApp.

## Planned Protocol / Auth

Expected REST over HTTPS via Safaricom WhatsApp Business API. OAuth2 bearer tokens for API authentication. May require webhook callback registration for message delivery status.

## Modules

- M37 (voter-education-cms) - WhatsApp education content delivery
- M23 (grievance-evidence) - grievance acknowledgment messages

## Next steps

1. Contact Safaricom for WhatsApp Business API access
2. Review Safaricom WhatsApp gateway documentation and rate limits
3. Negotiate MoU and WhatsApp Message Template approval
4. Arrange OAuth2 credentials and webhook endpoint setup
5. Implement message templating and delivery status tracking
