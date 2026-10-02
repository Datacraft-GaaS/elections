# ballot discrepancy response runbook

Module: M39 ballot-printing-chain. WP-4.3.

## Alert: station_reconciliation_discrepancy

Fires within 2 min of discrepancy detection (AC-M39-11). Event: `eel.inventory.station_reconciliation_discrepancy.v1`.

Steps:
1. ECC receives alert. Locate station in `station_reconciliations` by station_code.
2. Constituency Returning Officer must acknowledge within 1h or escalation triggers.
3. Check discrepancy arithmetic: `received - issued - unused - spoilt != 0`.
4. Review BVI turnout count from M06 vs ballots_issued in reconciliation record.
5. If RFID mismatch: check `rfid_mismatch_count` in `print_custody_events`; emit `rfid_mismatch_alert.v1`.
6. Station result placed on hold automatically until discrepancy resolved.

## Alert: quantity_variance_alert

Fires within 1 min of lot-data submission when `variance_pct > 0.01%`.

Steps:
1. Check lot record in `lot_data`; compare `quantity_authorised` (from `print_orders`) vs `quantity_printed`.
2. If within 0.01% tolerance: false alarm; verify and clear.
3. If beyond tolerance: notify the Commission Commissioner immediately.
4. Shipment flagged for hold; RFID spot-check required before allocation.

## the Commission Commissioner notification

For all P0 discrepancies:
1. ECC sends signed alert via M16 to Commissioner on-call.
2. Commissioner has 30 min to acknowledge before the Commission Ops Director escalation.
3. All escalations logged with timestamp in `print_orders_audit`.
