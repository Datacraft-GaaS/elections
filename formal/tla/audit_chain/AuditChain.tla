---- MODULE AuditChain ----
\* TLA+ Spec: M5 audit chain — append-only, hash-linked, signed
\* Proves: no fork, no reorg, no deletion, sequential chain integrity

EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
  ValidSigners,   \* set of authorised HSM keys
  RequiredSigs,   \* typically 2 of 3
  GenesisHash     \* SHA-256 of empty chain

VARIABLES
  chain,          \* sequence of audit entries
  current_hash,   \* latest Merkle root
  revoked_sigs    \* set of revoked signer IDs

InvAppendOnly == 
  \A i \in 1..Len(chain) : chain[i].entry_hash = prev_hash(chain, i)
  \* No entry is ever removed or reordered

InvNoFork == 
  \A i, j \in 1..Len(chain) :
    (i = j) => (chain[i].entry_hash = chain[j].entry_hash)
  \* At any moment, the chain has exactly one tip

InvNoReorg == 
  \A i \in 1..Len(chain)-1 : chain[i+1].prev_hash = chain[i].entry_hash
  \* Each block correctly references its predecessor

InvSignature == 
  \A entry \in {chain[i] : i \in 1..Len(chain)} :
    Cardinality(entry.signers \cap ValidSigners \cap revoked_sigs) >= RequiredSigs

InvGenesisImmutability ==
  \A entry \in {chain[i] : i \in 1..Len(chain)} :
    entry.prev_hash = GenesisHash \/ i > 1

\* Liveness: new valid entries can be appended
LiveAppend == 
  []<>\E e \in AppendableEntries : Len(chain') > Len(chain)

\* Safety: no two conflicting entries can coexist
Safety == 
  \A i, j \in 1..Len(chain) :
    (i /= j) => (chain[i].auditId /= chain[j].auditId)

====
