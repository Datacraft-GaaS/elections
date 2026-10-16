---- MODULE DataDiode ----
\* Network data-diode — egress-only policy enforcement
\* Applied to: platform/cilium/data-diode-policy.yaml

CONSTANTS kits, can_connect(_, _)

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

====
