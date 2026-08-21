# integrations/kebs

Kenya Bureau of Standards product certification and conformity services.

**Status: stub - MoU / integration spec pending.**

## Purpose

This integration will query KEBS product certification database for compliance verification. Note: ballot printer hardware warranty tracking is handled by kebs-warranty integration. This module supports M39 (ballot-printing-chain) for verifying compliance of printing equipment before procurement.

## Planned Protocol / Auth

Expected REST over HTTPS with API-key authentication. May require registration number and sector code for product classification.

## Modules

- M39 (ballot-printing-chain) - printer equipment certification compliance

## Next steps

1. Review KEBS certification API and product database schema
2. Negotiate KEBS partner agreement
3. Arrange API-key provisioning and access tier
4. Define product compliance query parameters (equipment type, model, certification status)
5. Implement adapter with certification cache/TTL handling
