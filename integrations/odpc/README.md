# integrations/odpc

Office of the Data Protection Commissioner data subject access requests and compliance.

**Status: stub - MoU / integration spec pending.**

## Purpose

This integration will handle data subject access requests (DSAR), privacy impact assessments, and general ODPC compliance inquiries. Note: breach notification reporting is handled by the odpc-breach integration. This module supports M28 (ai-monitoring-stack) for privacy compliance and M09 (odpc-breach) coordination.

## Planned Protocol / Auth

Expected REST over HTTPS with mTLS client certificates. RSA-PSS payload signing required for sensitive queries. Bearer tokens for non-sensitive DSAR status queries.

## Modules

- M28 (ai-monitoring-stack) - privacy compliance and impact assessments
- M09 (odpc-breach) - breach coordination and escalation

## Next steps

1. Review ODPC DSAR API and privacy compliance workflow
2. Negotiate Data Processor agreement with ODPC
3. Arrange mTLS certificate provisioning
4. Define DSAR request schema and response format
5. Implement payload signing and audit trail logging
