# integrations/kra

Kenya Revenue Authority general tax services and PIN verification.

**Status: stub - MoU / integration spec pending.**

## Purpose

This integration will handle general KRA tax inquiry services and PIN verification outside iTax and eClearance flows. Note: diaspora ballot import via e-Clearance is handled by kra-eclearance integration, and iTax integration is handled by kra-itax. This module supports M22 (campaign-finance) and M66 (financial-forensics) for general tax compliance queries.

## Planned Protocol / Auth

Expected REST over HTTPS with mTLS client certificates. Bearer tokens may be required for specific query tiers. KRA PIN authentication required for secure operations.

## Modules

- M22 (campaign-finance) - tax compliance queries
- M66 (financial-forensics) - KRA tax record inquiry

## Next steps

1. Clarify KRA service boundaries (general vs. iTax vs. eClearance)
2. Review KRA tax inquiry API schema
3. Negotiate MoU for tax data access
4. Arrange mTLS certificate provisioning
5. Define PIN authentication workflow and error handling
