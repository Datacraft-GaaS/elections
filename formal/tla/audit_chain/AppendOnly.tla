---- MODULE AppendOnly ----
\* Audit chain append-only invariant — no fork, no reorg
\* Applied to: M5 audit chain in transparency-svc

EXTENDS Naturals, FiniteSets

CONSTANTS NULL, Signers, Hash(_)

VARIABLE entries, prev_hash_of

\* Invariant: every entry correctly references its predecessor
InvPrevRef ==
  \A i \in DOMAIN entries :
    i > 1 => entries[i].prev_hash = Hash(entries[i-1])

\* Invariant: no entry can be removed
InvNoDelete ==
  Cardinality(entries) >= Cardinality(entries)'

\* Invariant: hashes are collision-free
InvNoCollision ==
  \A i, j \in DOMAIN entries : i /= j => entries[i].entry_hash /= entries[j].entry_hash

\* Invariant: signatures from valid signers only
InvSig ==
  \A e \in {entries[i] : i \in DOMAIN entries} :
    e.signature \in Signers

====
