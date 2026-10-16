---- MODULE BallotChain ----
\* Ballot paper chain-of-custody — sequential handover with HMAC integrity
\* Applied to: inventory-svc/ballot_chain.py

EXTENDS Naturals, FiniteSets, Sequences

CONSTANTS NULL, HMACKey, HMACVerify(_, _)

VARIABLE chain, hmac_ok, seq_no

\* Invariant: HMAC integrity at every handover
InvHMAC ==
  \A h \in chain : h.hmac_ok = HMACVerify(h, HMACKey)

\* Invariant: sequence numbers are monotonically increasing
InvSeq ==
  \A i, j \in DOMAIN chain :
    chain[i].seq_no < chain[j].seq_no => i < j

\* Invariant: no gap in handover sequence
InvNoGap ==
  \A i \in 1..Len(chain)-1 :
    chain[i+1].seq_no = chain[i].seq_no + 1

\* Invariant: each handover has a unique audit_id
InvUniqueId ==
  \A i, j \in 1..Len(chain) :
    i /= j => chain[i].audit_id /= chain[j].audit_id

====
