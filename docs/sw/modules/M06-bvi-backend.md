# M06 — BVI Back-End (Polling-Day Central)

**Runs on:** Primary + mirror DC
**Owner:** Datacraft
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Central authority for polling-day operations: extract per-station register bundles, provision kit configuration, ingest identification events, expose live turnout dashboards, orchestrate remote-lock / wipe for compromised kits.

## 2. Personas

- BVI Device (M05) — kit principal
- Presiding Officer / Returning Officer — via read APIs
- IEBC Command Centre — live dashboards
- Party agent — read-only turnout counters
- IEBC Administrator — kit-level remote actions

## 3. Functional Requirements

- **FR-M06-001** Generate per-station register-extract config bundle (voters + polygon + config) at kit-prep time and refresh on demand.
- **FR-M06-002** Distribute config bundles to kits via mTLS from Kit Gateway.
- **FR-M06-003** Kit whitelist / blacklist central management; MDM-like remote wipe of rogue kit; sub-minute propagation.
- **FR-M06-004** Ingest identification events from kits; verify per-event signature + kit certificate.
- **FR-M06-005** Live turnout dashboard: station / ward / constituency / county / national rollup with cumulative + real-time.
- **FR-M06-006** Live kit-health dashboard: battery, connectivity, GPS fix, last-seen, temperature, error counts.
- **FR-M06-007** Session lifecycle: PO opens → configurable window → PO closes; state transitions audited.
- **FR-M06-008** Emergency-close records justification + Elections Act §55 reference.
- **FR-M06-009** Distress-signal handler routes to war-room silently.
- **FR-M06-010** Per-election configuration: opening / closing time, quorum rules, form templates, geo-fence polygon updates.
- **FR-M06-011** Kit lost / stolen: revoke certificate, propagate ≤ 60 s, alert.
- **FR-M06-012** Voter-not-found events → auto-search central register for correct station → return redirect info to kit.
- **FR-M06-013** Party-agent view: aggregate turnout numbers per station (no PII, no biometric); rate-limited.
- **FR-M06-014** Every state change signed + Merkle-logged + escrow-mirrored.
- **FR-M06-015** Publish live turnout to Public Portal (M11) with delay/aggregation policy to prevent gaming.

## 4. Non-Functional Requirements

- **NFR-M06-001** Ingest 60,000 kits × 700 events/day = 42M events/day sustained; peak 5,000 events/sec.
- **NFR-M06-002** Live turnout dashboard latency ≤ 30 s from event to dashboard.
- **NFR-M06-003** RTO ≤ 30 min; RPO ≤ 5 min.
- **NFR-M06-004** Kit revocation propagation ≤ 60 s.
- **NFR-M06-005** 99.99% availability during polling window.

## 5. Integrations

- BVI Device (M05)
- BVR back-end (M02) for register lookup
- Kit certificate / PKI (P04)
- MDM (O01)
- Audit / Merkle log (P05)
- Public Portal (M11) for live turnout feed
- HSM (P06)

## 6. Failure Modes

| Failure | Response |
|---|---|
| Kit disconnected > 30 min | Alert to war-room; investigate |
| Kit revocation propagation fails | Retry + escalate; independent monitoring |
| Turnout ingest lag | Auto-scale; alert |
| Register extract stale | Force refresh on kit reboot; alert on discrepancy |
| Insider bulk kit-wipe attempt | Blocked by dual-control policy engine |

## 7. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M06-01 | 5,000 events/sec sustained | Load test |
| A-M06-02 | Turnout dashboard lag ≤ 30 s | Field trial |
| A-M06-03 | Kit revocation propagates ≤ 60 s | Chaos exercise |
| A-M06-04 | Emergency-close event correctly logged with §55 justification | UAT |
| A-M06-05 | Voter-not-found returns correct redirect | UAT |
| A-M06-06 | Distress-signal reaches war-room silently within 5 s | Red team |
| A-M06-07 | 99.99% availability over polling window | SLO measurement |

## 8. Cross-References

- `M05-bvi-device.md`
- `M11-public-portal.md`
- `../SECURITY.md` §3.12
