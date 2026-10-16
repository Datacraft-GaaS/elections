---- MODULE CcbMultiSig ----
\* CCB multi-signature log — 2-of-3 HSM authorization
\* Applied to: services/hsm-signer/ccb_log.rs

EXTENDS Naturals, FiniteSets, Sequences

CONSTANTS Signers, RequiredSigs, Hash(_)  \* RequiredSigs = 2

VARIABLE ccb_log, signer_status

Inv2Of3 ==
  \A entry \in ccb_log :
    Cardinality(entry.approvers \cap signer_status) >= RequiredSigs

InvPreviousHashChain ==
  \A i \in 2..Len(ccb_log) :
    ccb_log[i].prev_hash = Hash(ccb_log[i-1])

InvNoRetroactiveAmendment ==
  \A entry \in ccb_log : entry.final = TRUE => entry.signers >= RequiredSigs

====
