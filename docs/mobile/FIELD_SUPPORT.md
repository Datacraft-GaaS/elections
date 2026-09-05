# Field Support App — Complete Reference (M29)

**Code:** `services/field-mobile-svc/apps/field-support/`
**Module spec:** M29
**Screens:** shift_report, diagnostics, incident, warranty, ticket_queue
**Status:** All screens rewritten with `eel_design`

---

## Purpose

Field technician and support staff tool for station diagnostics, warranty claims, incident documentation, shift reports, and ticket queue management.

---

## Functions

### F1 — Shift Report (FR-001)
Auto-compiled from session data. Dual biometric signature (PO + DPO). UUID7 idempotent submit.

### F2 — Diagnostics (FR-002)
Kit status (battery, connectivity, GPS, form 34A sync status). Degraded state shown with `EelBanner` (amber) + `EelStatusChip`.

### F3 — Incident (FR-003)
Structured form: 7 categories + evidence + GPS consent + escalation chain.

### F4 — Warranty (FR-004)
Warranty claim submission with photo evidence and serial number tracking.

### F5 — Ticket Queue (FR-005, NEW)
Open tickets with priority sub-queue. Status chips (open/pending/resolved), `EelResultCard` per ticket, `EelButton` actions.

---

## Design Reference

All screens: `EelScaffold` + `EelResultCard` (info cards) + `EelSecureScreen` (warranty/incident) + `EelButton` (actions) + `EelStatusChip` (severity) + `EelBanner` (context).
