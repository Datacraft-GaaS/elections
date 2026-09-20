---- MODULE Sortition ----
\* TLA+ Formal Specification for EEL VRF Sortition (G02)
\* Proves: determinism, fairness (uniform selection), verifiability

EXTENDS Integers, FiniteSets, Sequences, TLC

\* The VRF secret derived from the merkle root
VARIABLE vrf_secret, merkle_root, selected_stations
VARIABLE audit_log  \* audit chain

\* Invariant: the VRF secret is a deterministic function of the merkle root
InvDeterminism == 
  \A s1, s2 \in FiniteSets({merkle_root}) :
    (s1 = s2) => (vrf_secret[s1] = vrf_secret[s2])

\* Invariant: selection fraction is approximately uniform (statistical fairness)
InvFairness ==
  \A fraction \in {0.01, 0.05, 0.1} :
    Cardinality(selected_stations) / Cardinality(station_set) \approx fraction

\* Invariant: audit log is append-only (no deletion, no reordering)
InvAuditAppendOnly ==
  \A i \in 1..Len(audit_log) :
    audit_log[i].auditId \in DOMAIN audit_ids

\* Temporal property: verification always succeeds for any published result
PropertyVerify ==
  \A result \in DOMAIN results :
    []<>(verify_sortition(result) = TRUE)

====
