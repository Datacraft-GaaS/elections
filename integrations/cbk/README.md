# integrations/cbk

Central Bank of Kenya general banking and monetary policy integration.

**Status: stub - MoU / integration spec pending.**

## Purpose

This integration will handle general CBK banking operations, monetary policy inquiries, and banking system queries. Note: specific financial intelligence flows are handled by the cbk-frc integration. This module supports M22 (campaign-finance) and M66 (financial-forensics) for non-intelligence banking inquiries.

## Planned Protocol / Auth

Expected REST over HTTPS with mTLS client certificates. OAuth2 bearer tokens may be required depending on CBK endpoint tier.

## Modules

- M22 (campaign-finance) - banking data lookups
- M66 (financial-forensics) - banking system inquiries

## Next steps

1. Review CBK API documentation and security requirements
2. Negotiate MoU and data-sharing agreement
3. Arrange mTLS certificate provisioning
4. Define banking inquiry scope and response schema
5. Implement adapter and contract tests
