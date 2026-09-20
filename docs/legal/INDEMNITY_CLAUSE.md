# G20 — Back-to-Back Indemnity Clause (CLOSED)
## Scope
Datacraft provides indemnity for: (a) system failure during election, (b) audit chain corruption, (c) data breach by vendor negligence, (d) supply chain failure.
## Insurance
Minimum coverage: $50M professional indemnity + $25M cyber liability (NCC Group broker).
## Limitation
Maximum liability: contract value × 3 (per IEBC tender clause 11.4).
## Trigger
Activation requires: (1) IAB determination, (2) M5 audit log verification, (3) court order or IEBC Chair directive.
## M5 Audit
Every indemnity claim emits audit event (auditId=g20-indemnity, proofHash, sig).
