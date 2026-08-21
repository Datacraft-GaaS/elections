# Inventory Admin

Django admin console for election inventory management. Manages assets (ballot boxes, seals, scanners), custody events and chains, consumables inventory, reserve pools, ballot print orders, and diaspora ballot logistics.

## Overview

Module: M20 (Inventory Management), M39 (Ballot Printing Chain), M65 (Diaspora Logistics)

Warehouse and logistics back-office for tracking physical election materials, managing custody chains, reconciling print orders, and coordinating diaspora ballot shipments.

## Key Features

- **Asset Management**: Create, register, and track all election materials (boxes, scanners, consumables)
- **Custody Chain**: Log custody transfer events with seal verification and timestamps
- **Seal Registry**: Track tamper-evident seal serial numbers and assignment
- **Consumables Tracking**: Monitor paper, ink, and other consumable usage
- **Reserve Pools**: Manage emergency reserve inventory at county/national level
- **Print Order Management**: Track ballot print orders from printer with QC reconciliation
- **Diaspora Logistics**: Coordinate ballot shipments to embassies with delivery tracking

## HTMX Endpoints

- GET `/inventory/admin/asset/{id}/custody-chain` - Timeline view of asset custody events
- POST `/inventory/admin/lot/{id}/reconcile` - Submit print lot reconciliation QC results

## Models (Managed=False)

- Asset - Inventory items (boxes, scanners, etc.)
- CustodyEvent - Asset transfer events with seal ID
- Seal - Tamper-evident seal registry (read-only)
- Consumable - Paper, ink, and other consumables
- ReservePool - Emergency inventory allocations
- PrintOrder - Ballot printing jobs (read-only)
- LotData - Print order lot verification results
- BallotContainer - Diaspora ballot shipment containers (read-only)

## Environment Variables

- `DJANGO_SECRET_KEY` - Django secret key
- `DATABASE_URL` - PostgreSQL connection string
- `INVENTORY_API_URL` - Inventory service API endpoint
- `KEBS_ADAPTER_URL` - KEBS (Kenya Bureau of Standards) certification
- `PRINTER_MES_ADAPTER_URL` - Printer MES (Manufacturing Execution System) tracking

## Running Locally

```bash
uv sync
uv run manage.py runserver
```

## Tests

```bash
uv run pytest -vxs tests/ci
```

## Deployment

WCAG 2.2 AA compliant. Pod resource: CPU 500m, Memory 512Mi.

Kubernetes manifest: `services/inventory-svc/deploy/base/admin-deployment.yaml`

Production run:

```bash
gunicorn inventory_admin.wsgi:application
```

## Permissions

INVENTORY_OFFICER, INVENTORY_SUPERVISOR, WAREHOUSE_MANAGER roles via SpiceDB. Supervisors approve custody transfers; officers record events.

## Notes

- All custody events are immutable (audit compliance)
- Seal serials are cryptographically signed in custody events
- Print lot reconciliation must be completed before ballot release
- Reserve pool draws require supervisor authorization
