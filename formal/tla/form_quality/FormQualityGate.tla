---- MODULE FormQualityGate ----
\* Form quality gate — threshold monotonicity
\* Applied to: mobile-bff/handlers/form_quality_gate.rs

EXTENDS Naturals, Reals, TLC

CONSTANTS MinWidth, MinHeight, MinOcrConf, MaxBlur, MinLux, Time, Thresholds(_)

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

====
