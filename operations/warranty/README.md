# operations/warranty

Device warranty tracking and RMA (Return Merchandise Authorization) workflow for the eel
KIEMS device fleet. Covers all Android Enterprise kits and Commission iOS devices under
the procurement warranty period.

## Purpose

The procurement contract for KIEMS devices includes a multi-year warranty. This directory
documents:

1. Which devices are covered, for how long, and by which vendor.
2. The RMA workflow for failed devices during registration and election day.
3. Coverage gaps and spare-device buffer calculations per county.

## Contents

| File | Description |
|------|-------------|
| `coverage-matrix.yaml` | Per-device-category warranty coverage: vendor, duration, SLA, contact |
| `rma-workflow.md` | Step-by-step RMA process from field failure to replacement delivery |
| `README.md` | This file |

## Coverage matrix summary

See `coverage-matrix.yaml` for the full matrix. Key entries:

| Device category | Warranty period | On-site SLA | Spare ratio |
|----------------|----------------|-------------|-------------|
| BVI/BVR/RTS kit | 3 years from delivery | Next business day (NBD) | 5% per county |
| Party agent mobile | 2 years | 3 business days | 3% per region |
| iOS staff device | 1 year (ABM) | Next business day | 10% of fleet |

A 5% spare ratio for kits means: for every 100 kits allocated to a county, 5 replacement
units are pre-positioned at the county returns office on election day.

## RMA workflow

See `rma-workflow.md` for the full process. Summary:

1. Field officer reports failure via helpdesk (kit hardware failure ticket, 4hr SLA).
2. Helpdesk triages and confirms hardware fault (not software recoverable).
3. Dispatch issues replacement kit from county spare stock.
4. Faulty kit tagged and shipped to returns office for vendor RMA.
5. `inventory-svc` updated with custody change at each step.

```bash
# Initiate an RMA from the inventory-svc admin API
curl -X POST \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  https://inventory.eel.internal/api/v1/rma \
  -d '{"device_id":"<uuid>","reason":"screen_failure","replacement_serial":"<serial>"}'

# Check spare stock levels per county
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  "https://inventory.eel.internal/api/v1/spare-stock?county_code=<code>" \
  | jq '.available_spares'
```

## Config and environment

| Variable | Description |
|----------|-------------|
| `ADMIN_TOKEN` | Admin bearer token for `inventory-svc` API |
| `INVENTORY_URL` | Base URL for `inventory-svc` |
| `WARRANTY_VENDOR_CONTACT` | Vendor support contact (in vault; not stored here) |

## Dependencies

- `inventory-svc` (M20): device custody and RMA event source of truth
- `field-mobile-svc` (M32): creates helpdesk ticket on kit failure report
- `operations/helpdesk/`: helpdesk SLA governs kit replacement time

## Test and verify

```bash
# Verify coverage-matrix.yaml is valid YAML
python3 -c "import yaml; yaml.safe_load(open('operations/warranty/coverage-matrix.yaml'))"

# Check inventory-svc RMA endpoint is accessible
curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  https://inventory.eel.internal/api/v1/rma
# Expected: 200 or 405 (endpoint exists)
```

## Related docs

- `operations/mdm/README.md`: device lifecycle including replacement and wipe steps
- `operations/dispatch/README.md`: field dispatch for delivering replacement kits
- `operations/helpdesk/README.md`: kit failure ticket SLAs

## Limitations

- Spare stock levels are maintained in `inventory-svc`, not in this directory. The
  coverage matrix is policy documentation, not a live inventory.
- Vendor NBD SLA applies to business days; election day is a public holiday. Confirm
  with the vendor that election-day SLA applies before deployment.
- RMA for devices in remote areas (Turkana, Mandera) may exceed the nominal SLA due to
  logistics; plan for a higher spare ratio in those counties.
