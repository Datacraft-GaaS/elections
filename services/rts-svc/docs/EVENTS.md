# RTS Service Events

All events use the CloudEvents 1.0 envelope with eel_* extensions. Source is `spiffe://eel.local/rts-svc`.

| Event Type | Topic | Data Class | Description |
|---|---|---|---|
| `eel.rts.form34a_captured.v1` | `eel.rts.form34a.v1` | B | Polling station submits Form 34A tally sheet |
| `eel.rts.form34a_verified.v1` | `eel.rts.form34a.v1` | B | Returning Officer verifies Form 34A data |
| `eel.rts.form34a_amended.v1` | `eel.rts.form34a.v1` | B | Form 34A corrected after initial capture (preserves original in log) |
| `eel.rts.form34a_rejected.v1` | `eel.rts.form34a.v1` | B | Form 34A rejected with reason (e.g. failed EC-DAT-10) |
| `eel.rts.form34b_compiled.v1` | `eel.rts.form34b.v1` | B | Constituency tally compiled from verified Form 34A sheets |
| `eel.rts.form34c_compiled.v1` | `eel.rts.form34c.v1` | B | National tally compiled from all Form 34B totals |
| `eel.rts.tally_published.v1` | `eel.rts.publication.v1` | B | Final tally published with Merkle root and transparency log entry |
| `eel.rts.parallel_tally_drift.v1` | `eel.rts.audit.v1` | B | Drift detected between parallel independent aggregators |
