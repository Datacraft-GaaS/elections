# integrations/frc

Financial Reporting Centre general financial intelligence and compliance.

**Status: stub - MoU / integration spec pending.**

## Purpose

This integration will handle general FRC financial reporting, STR (Suspicious Transaction Report) inquiries, and financial compliance queries. Note: specific joint CBK/FRC financial intelligence flows are handled by the cbk-frc integration. This module supports M22 (campaign-finance) and M66 (financial-forensics) for FRC-specific compliance queries.

## Planned Protocol / Auth

Expected REST over HTTPS with mTLS client certificates. FRC may require signed payloads using HMAC-SHA256 depending on endpoint sensitivity.

## Modules

- M22 (campaign-finance) - FRC compliance lookups
- M66 (financial-forensics) - financial intelligence system inquiries

## Next steps

1. Review FRC API documentation and STR schema
2. Negotiate MoU for financial intelligence access
3. Arrange mTLS certificate provisioning
4. Define query scope (STR, CTR, compliance status)
5. Implement adapter with payload signing support
