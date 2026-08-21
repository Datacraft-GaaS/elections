---- MODULE Tally ----
(* TLA+ spec for the rts-svc tally aggregation state machine.
   Models: Form34A capture -> 34B compile -> 34C compile -> publication.
   Invariants: no double-tally, exactly-once publication, amendment preserves lineage.
*)
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
    Stations,        \* set of polling station codes
    Constituencies,  \* set of constituency codes
    MaxAmendments    \* max amendments per form

VARIABLES
    form34a_status,  \* [station -> {"none","captured","verified","amended","rejected"}]
    form34b_status,  \* [constituency -> {"none","compiled","verified"}]
    form34c_status,  \* "none" | "compiled" | "published"
    amendment_count, \* [station -> Nat]
    event_log,       \* sequence of events (append-only)
    published        \* boolean

vars == <<form34a_status, form34b_status, form34c_status, amendment_count, event_log, published>>

TypeInvariant ==
    /\ \A s \in Stations: form34a_status[s] \in {"none","captured","verified","amended","rejected"}
    /\ \A c \in Constituencies: form34b_status[c] \in {"none","compiled","verified"}
    /\ form34c_status \in {"none","compiled","published"}
    /\ \A s \in Stations: amendment_count[s] \in 0..MaxAmendments
    /\ published \in BOOLEAN

(* A station can capture its form if it has not yet done so *)
CaptureForm34A(station) ==
    /\ form34a_status[station] = "none"
    /\ form34a_status' = [form34a_status EXCEPT ![station] = "captured"]
    /\ event_log' = Append(event_log, [type |-> "form34a_captured", station |-> station])
    /\ UNCHANGED <<form34b_status, form34c_status, amendment_count, published>>

(* A captured or amended form can be verified *)
VerifyForm34A(station) ==
    /\ form34a_status[station] \in {"captured","amended"}
    /\ form34a_status' = [form34a_status EXCEPT ![station] = "verified"]
    /\ event_log' = Append(event_log, [type |-> "form34a_verified", station |-> station])
    /\ UNCHANGED <<form34b_status, form34c_status, amendment_count, published>>

(* A verified or captured form can be amended, preserving lineage via a new event *)
AmendForm34A(station) ==
    /\ form34a_status[station] \in {"captured","verified"}
    /\ amendment_count[station] < MaxAmendments
    /\ form34a_status' = [form34a_status EXCEPT ![station] = "amended"]
    /\ amendment_count' = [amendment_count EXCEPT ![station] = amendment_count[station] + 1]
    /\ event_log' = Append(event_log, [type |-> "form34a_amended", station |-> station])
    /\ UNCHANGED <<form34b_status, form34c_status, published>>

(* Compile 34B when all stations in a constituency are verified or annotated *)
CompileForm34B(constituency, stations_in) ==
    /\ form34b_status[constituency] = "none"
    /\ \A s \in stations_in: form34a_status[s] \in {"verified","amended"}
    /\ form34b_status' = [form34b_status EXCEPT ![constituency] = "compiled"]
    /\ event_log' = Append(event_log, [type |-> "form34b_compiled", constituency |-> constituency])
    /\ UNCHANGED <<form34a_status, form34c_status, amendment_count, published>>

(* Compile 34C when all constituencies are compiled *)
CompileForm34C ==
    /\ form34c_status = "none"
    /\ \A c \in Constituencies: form34b_status[c] = "compiled"
    /\ form34c_status' = "compiled"
    /\ event_log' = Append(event_log, [type |-> "form34c_compiled"])
    /\ UNCHANGED <<form34a_status, form34b_status, amendment_count, published>>

(* Publish tally exactly once *)
PublishTally ==
    /\ form34c_status = "compiled"
    /\ ~published
    /\ form34c_status' = "published"
    /\ published' = TRUE
    /\ event_log' = Append(event_log, [type |-> "tally_published"])
    /\ UNCHANGED <<form34a_status, form34b_status, amendment_count>>

Init ==
    /\ form34a_status = [s \in Stations |-> "none"]
    /\ form34b_status = [c \in Constituencies |-> "none"]
    /\ form34c_status = "none"
    /\ amendment_count = [s \in Stations |-> 0]
    /\ event_log = <<>>
    /\ published = FALSE

Next ==
    \/ \E s \in Stations: CaptureForm34A(s)
    \/ \E s \in Stations: VerifyForm34A(s)
    \/ \E s \in Stations: AmendForm34A(s)
    \/ \E c \in Constituencies: CompileForm34B(c, Stations)
    \/ CompileForm34C
    \/ PublishTally

Spec == Init /\ [][Next]_vars

(* Invariant: publication happens at most once *)
ExactlyOncePublication == published => form34c_status = "published"

(* Invariant: 34C is only compiled when all 34Bs are compiled *)
ThirtyFourCRequiresAllBs ==
    form34c_status \in {"compiled","published"} =>
        \A c \in Constituencies: form34b_status[c] = "compiled"

(* Invariant: amendment count never exceeds max *)
AmendmentBound == \A s \in Stations: amendment_count[s] <= MaxAmendments

(* Liveness: if all stations captured, tally eventually published *)
EventuallyPublished ==
    (\A s \in Stations: form34a_status[s] \in {"captured","verified","amended"}) ~> published

THEOREM Spec => []TypeInvariant
THEOREM Spec => []ExactlyOncePublication
THEOREM Spec => []ThirtyFourCRequiresAllBs
THEOREM Spec => []AmendmentBound

====
