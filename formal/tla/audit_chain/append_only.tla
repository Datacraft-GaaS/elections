---- MODULE AppendOnly ----
\* Audit chain append-only invariant — no fork, no reorg
\* Applied to: M5 audit chain in transparency-svc

CONSTANTS NULL, Signers

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
  \A e \in Range(entries) :
    e.signature \in Signers

==== TLA

cat > ./formal/tla/ballot_chain/BallotChain.tla << 'TLA'
---- MODULE BallotChain ----
\* Ballot paper chain-of-custody — sequential handover with HMAC integrity
\* Applied to: inventory-svc/ballot_chain.py

CONSTANTS NULL, HMACKey

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

==== TLA

cat > ./formal/tla/ccb/CcbMultiSig.tla << 'TLA'
---- MODULE CcbMultiSig ----
\* CCB multi-signature log — 2-of-3 HSM authorization
\* Applied to: services/hsm-signer/ccb_log.rs

CONSTANTS Signers, RequiredSigs  \* RequiredSigs = 2

VARIABLE ccb_log, signer_status

Inv2Of3 ==
  \A entry \in ccb_log :
    Cardinality(entry.approvers \cap signer_status) >= RequiredSigs

InvPreviousHashChain ==
  \A i \in 2..Len(ccb_log) :
    ccb_log[i].prev_hash = Hash(ccb_log[i-1])

InvNoRetroactiveAmendment ==
  \A entry \in ccb_log : entry.final = TRUE => entry.signers >= RequiredSigs

==== TLA

cat > ./formal/tla/data_diode/DataDiode.tla << 'TLA'
---- MODULE DataDiode ----
\* Network data-diode — egress-only policy enforcement
\* Applied to: platform/cilium/data-diode-policy.yaml

VARIABLES network_policy, allowed_ingress, allowed_egress

\* Invariant: ingress is always denied during polling hours
InvIngressDenied ==
  allowed_ingress = {}  \* No ingress allowed

\* Invariant: egress is restricted to RTS namespace only
InvEgressRestricted ==
  allowed_egress \subseteq {"rts-svc"}

\* Invariant: no peer-to-peer communication between kits
InvNoP2P ==
  \A kit1, kit2 \in kits :
    kit1 /= kit2 => \neg can_connect(kit1, kit2)

==== TLA

cat > ./formal/tla/form_quality/FormQualityGate.tla << 'TLA'
---- MODULE FormQualityGate ----
\* Form quality gate — threshold monotonicity
\* Applied to: mobile-bff/handlers/form_quality_gate.rs

CONSTANTS MinWidth, MinHeight, MinOcrConf, MaxBlur, MinLux

VARIABLE form_state

\* Invariant: all accepted forms pass every threshold
InvAcceptImpliesPass ==
  form_state.status = "accepted" =>
    /\ form_state.width >= MinWidth
    /\ form_state.height >= MinHeight
    /\ form_state.ocr_confidence >= MinOcrConf
    /\ form_state.blur <= MaxBlur
    /\ form_state.illumination >= MinLux

\* Invariant: rejected forms fail at least one threshold
InvRejectImpliesFail ==
  form_state.status = "rejected" =>
    \/ form_state.width < MinWidth
    \/ form_state.height < MinHeight
    \/ form_state.ocr_confidence < MinOcrConf
    \/ form_state.blur > MaxBlur
    \/ form_state.illumination < MinLux

\* Invariant: no threshold is ever lowered (monotonicity)
InvMonotonicThresholds ==
  \A t1, t2 \in Time : t1 < t2 =>
    /\ Thresholds(t2).MinWidth >= Thresholds(t1).MinWidth
    /\ Thresholds(t2).MinOcrConf >= Thresholds(t1).MinOcrConf

==== TLA
echo "All 5 TLA+ specs written"
