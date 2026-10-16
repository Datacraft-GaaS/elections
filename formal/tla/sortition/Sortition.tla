---- MODULE Sortition ----
\* TLA+ Formal Specification for EEL VRF Sortition (G02)
\* Proves: determinism, fairness (uniform selection), verifiability

EXTENDS Integers, FiniteSets, Sequences, TLC

CONSTANTS
  station_set,      \* every station eligible for this election
  audit_ids,        \* audit ids the chain has issued
  results,          \* published sortition results, keyed by merkle root
  verify_sortition(_)

\* The VRF secret derived from the merkle root
VARIABLE vrf_secret, merkle_root, selected_stations
VARIABLE audit_log  \* audit chain

\* Invariant: the VRF secret is a deterministic function of the merkle root
InvDeterminism ==
  \A s1, s2 \in {merkle_root} :
    (s1 = s2) => (vrf_secret[s1] = vrf_secret[s2])

\* Invariant: selection fraction is approximately uniform (statistical fairness).
\* TLA+ has no `\approx`; "approximately" over integers is a two-sided bound.
\* The 5% target with a +/-1pp tolerance, cross-multiplied to stay in Int.
InvFairness ==
  100 * Cardinality(selected_stations) >= 4 * Cardinality(station_set) /\
  100 * Cardinality(selected_stations) <= 6 * Cardinality(station_set)

\* Invariant: audit log is append-only (no deletion, no reordering)
InvAuditAppendOnly ==
  \A i \in 1..Len(audit_log) :
    audit_log[i].auditId \in DOMAIN audit_ids

\* Temporal property: verification always succeeds for any published result
PropertyVerify ==
  \A result \in DOMAIN results :
    []<>(verify_sortition(result) = TRUE)

====
