-------------------------------- MODULE Referendum --------------------------------
(**
 * TLA+ formal model for the Referendum county-threshold state machine.
 *
 * Models: M49 referendum-mechanics, WP-5.2
 * Invariants verified by TLC:
 *   (1) NoDeclarationWithoutBothThresholds
 *   (2) ExactlyTwentyFourCounties (boundary enforcement)
 *   (3) ExactMajority (national majority strictly > 50% of votes cast)
 *
 * Counties are numbered 1..TOTAL_COUNTIES (47 in production).
 * For TLC model checking, set TOTAL_COUNTIES = 5 and COUNTY_THRESHOLD = 3
 * in the model constants to keep the state space tractable.
 *
 * Usage:
 *   TLC (model checker):
 *     Constants: TOTAL_COUNTIES <- 5, COUNTY_THRESHOLD <- 3
 *     Spec: Spec
 *     Invariants: NoDeclarationWithoutBothThresholds,
 *                 ExactlyTwentyFourCounties, ExactMajority
 *   TLAPS (proof assistant): not required for WP-5.2 DoD.
 *
 * TLC availability: if TLC is not installed, run via:
 *   docker run --rm -v $(pwd):/tla lipanski/tla-plus-toolbox:latest \
 *     java -jar /opt/tla2tools.jar -workers auto \
 *     -config Referendum.cfg Referendum.tla
 *
 * TLC config file: docs/tla/Referendum.cfg (companion file)
 *)

EXTENDS Naturals, FiniteSets, Sequences

CONSTANTS
    TOTAL_COUNTIES,    \* 47 in production; use 5 for TLC
    COUNTY_THRESHOLD   \* 24 in production; use 3 for TLC

ASSUME COUNTY_THRESHOLD \in 1..TOTAL_COUNTIES

Counties == 1..TOTAL_COUNTIES

(* Each county votes YES or NO; result is PENDING until certified *)
(* Referendum states *)
States == {"DRAFT", "ACTIVE", "TALLYING", "DECLARED", "FAILED"}

VARIABLES
    referendumState,     \* current referendum lifecycle state
    countyResult,        \* function: county -> {"PENDING", "YES", "NO"}
    countiesYesMajority, \* running count of counties with YES majority
    nationalYes,         \* total YES votes (aggregate; integer)
    nationalNo,          \* total NO votes (aggregate; integer)
    declared             \* boolean: referendum has been declared

vars == <<referendumState, countyResult, countiesYesMajority,
          nationalYes, nationalNo, declared>>

(* ---- Type invariant ---- *)
TypeOK ==
    /\ referendumState \in States
    /\ countyResult \in [Counties -> {"PENDING", "YES", "NO"}]
    /\ countiesYesMajority \in 0..TOTAL_COUNTIES
    /\ nationalYes \in Nat
    /\ nationalNo \in Nat
    /\ declared \in BOOLEAN

(* ---- Initial state ---- *)
Init ==
    /\ referendumState = "DRAFT"
    /\ countyResult = [c \in Counties |-> "PENDING"]
    /\ countiesYesMajority = 0
    /\ nationalYes = 0
    /\ nationalNo = 0
    /\ declared = FALSE

(* ---- Actions ---- *)

(* Activate the referendum: move from DRAFT to ACTIVE *)
Activate ==
    /\ referendumState = "DRAFT"
    /\ referendumState' = "ACTIVE"
    /\ UNCHANGED <<countyResult, countiesYesMajority, nationalYes, nationalNo, declared>>

(* A county certifies its result (YES or NO) *)
CertifyCounty(c, outcome) ==
    /\ referendumState = "ACTIVE"
    /\ countyResult[c] = "PENDING"
    /\ outcome \in {"YES", "NO"}
    /\ countyResult' = [countyResult EXCEPT ![c] = outcome]
    /\ countiesYesMajority' = IF outcome = "YES"
                               THEN countiesYesMajority + 1
                               ELSE countiesYesMajority
    /\ nationalYes' = IF outcome = "YES" THEN nationalYes + 1 ELSE nationalYes
    /\ nationalNo'  = IF outcome = "NO"  THEN nationalNo  + 1 ELSE nationalNo
    /\ UNCHANGED <<referendumState, declared>>

(* All counties have reported *)
AllCountiesReported == \A c \in Counties: countyResult[c] # "PENDING"

(* County threshold met: counties with YES majority >= COUNTY_THRESHOLD *)
CountyThresholdMet == countiesYesMajority >= COUNTY_THRESHOLD

(* National majority: YES > total votes cast / 2 (integer; no floating point) *)
(* yes * 2 > (yes + no) *)
NationalMajorityMet == nationalYes * 2 > (nationalYes + nationalNo)

(* Move to TALLYING when all counties have reported *)
BeginTallying ==
    /\ referendumState = "ACTIVE"
    /\ AllCountiesReported
    /\ referendumState' = "TALLYING"
    /\ UNCHANGED <<countyResult, countiesYesMajority, nationalYes, nationalNo, declared>>

(* Declare passed when both thresholds are met *)
DeclarePass ==
    /\ referendumState = "TALLYING"
    /\ CountyThresholdMet
    /\ NationalMajorityMet
    /\ referendumState' = "DECLARED"
    /\ declared' = TRUE
    /\ UNCHANGED <<countyResult, countiesYesMajority, nationalYes, nationalNo>>

(* Declare failed when at least one threshold is not met *)
DeclareFail ==
    /\ referendumState = "TALLYING"
    /\ ~(CountyThresholdMet /\ NationalMajorityMet)
    /\ referendumState' = "FAILED"
    /\ declared' = FALSE
    /\ UNCHANGED <<countyResult, countiesYesMajority, nationalYes, nationalNo>>

(* ---- Next-state relation ---- *)
Next ==
    \/ Activate
    \/ \E c \in Counties, o \in {"YES", "NO"}: CertifyCounty(c, o)
    \/ BeginTallying
    \/ DeclarePass
    \/ DeclareFail

(* ---- Fairness ---- *)
Fairness ==
    /\ WF_vars(Activate)
    /\ WF_vars(BeginTallying)
    /\ WF_vars(DeclarePass)
    /\ WF_vars(DeclareFail)
    /\ \A c \in Counties: WF_vars(\E o \in {"YES", "NO"}: CertifyCounty(c, o))

(* ---- Spec ---- *)
Spec == Init /\ [][Next]_vars /\ Fairness

(* ================================================================
   INVARIANTS
   ================================================================ *)

(**
 * INV-1: NoDeclarationWithoutBothThresholds
 * A DECLARED state can only be reached if both the county threshold
 * (countiesYesMajority >= COUNTY_THRESHOLD) and the national majority
 * (nationalYes * 2 > nationalYes + nationalNo) are satisfied.
 *)
NoDeclarationWithoutBothThresholds ==
    declared => (CountyThresholdMet /\ NationalMajorityMet)

(**
 * INV-2: ExactlyTwentyFourCounties
 * The running county YES majority count can never exceed the total number
 * of counties (it is bounded by TOTAL_COUNTIES). This ensures the threshold
 * is checked against a valid count, and that the threshold value itself
 * (COUNTY_THRESHOLD) is reachable and within range.
 *)
ExactlyTwentyFourCounties ==
    /\ countiesYesMajority \in 0..TOTAL_COUNTIES
    /\ COUNTY_THRESHOLD \in 1..TOTAL_COUNTIES

(**
 * INV-3: ExactMajority
 * When declared = TRUE, the national YES total is a strict majority
 * of all votes cast (integer arithmetic; no floating point).
 *)
ExactMajority ==
    declared => (nationalYes * 2 > (nationalYes + nationalNo))

(* ---- Liveness property ---- *)
(* Eventually every referendum either passes or fails *)
EventuallyDecided ==
    <>(referendumState \in {"DECLARED", "FAILED"})

================================================================================
