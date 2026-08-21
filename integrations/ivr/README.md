# integrations/ivr

IVR call-centre platform for citizen support and grievance inquiries.

**Status: stub - MoU / integration spec pending.**

## Purpose

This integration will manage IVR call-centre interactions for citizen support, grievance submission, and election inquiry responses. It will support M23 (grievance evidence intake) with voice/DTMF workflows and M37 (voter-education-cms) with multilingual education content distribution.

## Planned Protocol / Auth

Expected REST over HTTPS with OAuth2 bearer tokens. SOAP/TAPI endpoints may be offered by the IVR platform for call control and recording management.

## Modules

- M23 (grievance-evidence) - grievance voice intake and logging
- M37 (voter-education-cms) - education content delivery via IVR

## Next steps

1. Identify IVR platform vendor and architecture
2. Review IVR API capabilities (call routing, recording, DTMF, voice prompts)
3. Negotiate MoU for citizen support service
4. Define call flow schema and grievance intake template
5. Arrange OAuth2 credentials and recording storage access
