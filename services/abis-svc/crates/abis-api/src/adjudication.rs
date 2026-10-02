//! The adjudication case lifecycle.
//!
//! Five of the seven adjudication handlers answered "no store behind it" with a
//! 501. That was true when they were written, and it stopped being true when this
//! crate grew an `AbisStore`: a case, its assignment, its evidence and its decision
//! are ordinary state, and keeping them here is the difference between an
//! adjudication queue and five refusals.
//!
//! What genuinely is not here is the **comparison engine**, and that is why
//! `resolve_adjudication` on the legacy PATCH route stays a 501: it used to answer
//! 200 with a fabricated `match_score: 0.65` and `outcome: "no_match"` for any
//! case id, naming probe and gallery records it never read. Producing a real score
//! means comparing templates, and the comparison engine and the template store are
//! not in this workspace.
//!
//! Cases enter through [`AdjudicationStore::open_case`], which is what the
//! enrolment pipeline calls when a capture needs human review. There is no
//! `POST /cases` on the contract, so there is deliberately no HTTP route that
//! invents one.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

/// Where a case is in its lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseState {
    /// Raised, nobody assigned yet.
    Open,
    /// Assigned to an adjudicator, awaiting their decision.
    Assigned,
    /// Escalated: a second look by a senior adjudicator.
    Escalated,
    /// Decided. Terminal.
    Decided,
}

impl CaseState {
    /// The wire value.
    pub fn wire(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Assigned => "assigned",
            Self::Escalated => "escalated",
            Self::Decided => "decided",
        }
    }

    /// Whether the case can still change.
    pub fn is_open(&self) -> bool {
        !matches!(self, Self::Decided)
    }
}

/// One piece of evidence attached to a case.
#[derive(Clone, Debug, Serialize)]
pub struct Evidence {
    pub evidence_id: String,
    pub kind: String,
    /// A reference, not an upload: this service records that evidence exists and
    /// where it is, and the artefact itself lives in object storage.
    pub url: String,
    pub added_at: String,
}

/// A decision on a case.
#[derive(Clone, Debug, Serialize)]
pub struct Decision {
    pub outcome: String,
    pub adjudicator_id: String,
    /// Present for a tier-3 decision, which needs a second sign-off.
    pub second_adjudicator_id: Option<String>,
    pub resolution_notes: String,
    pub decided_at: String,
}

/// One adjudication case.
#[derive(Clone, Debug, Serialize)]
pub struct Case {
    pub case_id: String,
    /// The enrolment job that raised it.
    pub job_id: String,
    pub subject_ref: String,
    pub priority: String,
    pub state: CaseState,
    pub assigned_to: Option<String>,
    pub assigned_at: Option<String>,
    pub evidence: Vec<Evidence>,
    pub decision: Option<Decision>,
    pub escalation_reason: Option<String>,
    /// The comparison score, once a decision has been made against one.
    ///
    /// `None` until then, because a case that has not been decided has no
    /// comparison behind it. This is why `min_score` filtering is a real filter and
    /// not an ignored parameter.
    pub match_score: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

/// In-process adjudication state.
#[derive(Default)]
pub struct AdjudicationStore {
    cases: Mutex<HashMap<String, Case>>,
    order: Mutex<Vec<String>>,
}

/// The handle threaded through the router.
pub type SharedAdjudicationStore = Arc<AdjudicationStore>;

/// A short id from the tail of a UUIDv7's hex, where the random part is.
fn short_id() -> String {
    let hex = uuid::Uuid::now_v7().simple().to_string();
    hex[hex.len() - 12..].to_string()
}

impl AdjudicationStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// The shared handle.
    pub fn shared(self) -> SharedAdjudicationStore {
        Arc::new(self)
    }

    /// Raise a case for a job that needs human review.
    ///
    /// Called by the enrolment pipeline when a capture cannot be auto-decided.
    /// There is no HTTP route that calls this, because the contract has no
    /// `POST /cases`: a case is raised by the process that has something to review,
    /// not by a client asking for one.
    pub fn open_case(
        &self,
        job_id: &str,
        subject_ref: &str,
        priority: &str,
    ) -> Case {
        let now = chrono::Utc::now().to_rfc3339();
        let case = Case {
            case_id: format!("adj-{}", short_id()),
            job_id: job_id.to_string(),
            subject_ref: subject_ref.to_string(),
            priority: priority.to_string(),
            state: CaseState::Open,
            assigned_to: None,
            assigned_at: None,
            evidence: Vec::new(),
            decision: None,
            escalation_reason: None,
            match_score: None,
            created_at: now.clone(),
            updated_at: now,
        };
        self.order.lock().expect("lock poisoned").push(case.case_id.clone());
        self.cases
            .lock()
            .expect("lock poisoned")
            .insert(case.case_id.clone(), case.clone());
        case
    }

    /// A case by id.
    pub fn case(&self, case_id: &str) -> Option<Case> {
        self.cases.lock().expect("lock poisoned").get(case_id).cloned()
    }

    /// Every case, oldest first.
    pub fn cases(&self) -> Vec<Case> {
        let order = self.order.lock().expect("lock poisoned");
        let guard = self.cases.lock().expect("lock poisoned");
        order.iter().filter_map(|id| guard.get(id).cloned()).collect()
    }

    /// Cases matching a status and a minimum match score, oldest first.
    pub fn cases_matching(
        &self,
        status: Option<&str>,
        min_score: Option<f64>,
    ) -> Vec<Case> {
        self.cases()
            .into_iter()
            .filter(|c| status.is_none_or(|s| c.state.wire() == s))
            .filter(|c| match min_score {
                // A case with no score cannot satisfy a minimum: there is nothing
                // behind it to compare.
                Some(min) => c.match_score.is_some_and(|s| s >= min),
                None => true,
            })
            .collect()
    }

    /// Assign a case to an adjudicator.
    pub fn assign(
        &self,
        case_id: &str,
        adjudicator_id: &str,
    ) -> Result<Case, String> {
        let mut guard = self.cases.lock().expect("lock poisoned");
        let case = guard
            .get_mut(case_id)
            .ok_or_else(|| format!("no case {case_id}"))?;
        if !case.state.is_open() {
            return Err(format!(
                "case is {}; a decided case cannot be assigned",
                case.state.wire()
            ));
        }
        if let Some(existing) = &case.assigned_to {
            if existing != adjudicator_id {
                return Err(format!(
                    "case is already assigned to {existing}; re-assignment is a \
                     supervisor action and is not part of this endpoint"
                ));
            }
        }
        let now = chrono::Utc::now().to_rfc3339();
        case.assigned_to = Some(adjudicator_id.to_string());
        case.assigned_at = Some(now.clone());
        case.state = CaseState::Assigned;
        case.updated_at = now;
        Ok(case.clone())
    }

    /// Escalate a case.
    pub fn escalate(&self, case_id: &str, reason: &str) -> Result<Case, String> {
        let mut guard = self.cases.lock().expect("lock poisoned");
        let case = guard
            .get_mut(case_id)
            .ok_or_else(|| format!("no case {case_id}"))?;
        if !case.state.is_open() {
            return Err(format!(
                "case is {}; a decided case cannot be escalated",
                case.state.wire()
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        case.escalation_reason = Some(reason.to_string());
        case.state = CaseState::Escalated;
        case.updated_at = now;
        Ok(case.clone())
    }

    /// Attach evidence to a case.
    pub fn attach_evidence(
        &self,
        case_id: &str,
        kind: &str,
        url: &str,
    ) -> Result<Evidence, String> {
        let mut guard = self.cases.lock().expect("lock poisoned");
        let case = guard
            .get_mut(case_id)
            .ok_or_else(|| format!("no case {case_id}"))?;
        if !case.state.is_open() {
            return Err(format!(
                "case is {}; evidence cannot be added to a decided case",
                case.state.wire()
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let evidence = Evidence {
            evidence_id: format!("ev-{}", short_id()),
            kind: kind.to_string(),
            url: url.to_string(),
            added_at: now.clone(),
        };
        case.evidence.push(evidence.clone());
        case.updated_at = now;
        Ok(evidence)
    }

    /// Record a decision on a case.
    pub fn decide(
        &self,
        case_id: &str,
        outcome: &str,
        adjudicator_id: &str,
        second_adjudicator_id: Option<&str>,
        resolution_notes: &str,
        match_score: Option<f64>,
    ) -> Result<Case, String> {
        let mut guard = self.cases.lock().expect("lock poisoned");
        let case = guard
            .get_mut(case_id)
            .ok_or_else(|| format!("no case {case_id}"))?;
        if !case.state.is_open() {
            return Err(format!(
                "case is already {}; a case is decided once",
                case.state.wire()
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        case.decision = Some(Decision {
            outcome: outcome.to_string(),
            adjudicator_id: adjudicator_id.to_string(),
            second_adjudicator_id: second_adjudicator_id.map(str::to_string),
            resolution_notes: resolution_notes.to_string(),
            decided_at: now.clone(),
        });
        case.match_score = match_score;
        case.state = CaseState::Decided;
        case.updated_at = now;
        Ok(case.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_case_moves_from_open_to_decided_and_not_back() {
        let store = AdjudicationStore::new();
        let case = store.open_case("job-1", "subject-1", "normal");
        assert_eq!(case.state, CaseState::Open);

        store.assign(&case.case_id, "adj-1").unwrap();
        assert_eq!(store.case(&case.case_id).unwrap().state, CaseState::Assigned);

        store.escalate(&case.case_id, "second look needed").unwrap();
        assert_eq!(store.case(&case.case_id).unwrap().state, CaseState::Escalated);

        store
            .decide(&case.case_id, "no_match", "adj-2", None, "not a match", Some(0.42))
            .unwrap();
        let decided = store.case(&case.case_id).unwrap();
        assert_eq!(decided.state, CaseState::Decided);
        assert_eq!(decided.match_score, Some(0.42));

        // Terminal: nothing may change it.
        assert!(store.assign(&case.case_id, "adj-3").is_err());
        assert!(store.escalate(&case.case_id, "again").is_err());
        assert!(store.attach_evidence(&case.case_id, "probe", "s3://x").is_err());
    }

    #[test]
    fn a_case_is_assigned_once() {
        let store = AdjudicationStore::new();
        let case = store.open_case("job-2", "subject-2", "normal");
        store.assign(&case.case_id, "adj-1").unwrap();
        // Re-assigning to the same adjudicator is idempotent.
        store.assign(&case.case_id, "adj-1").unwrap();
        // Re-assigning to a different one is refused rather than silently moved.
        assert!(store.assign(&case.case_id, "adj-2").is_err());
    }

    #[test]
    fn min_score_only_matches_a_case_that_has_been_compared() {
        let store = AdjudicationStore::new();
        store.open_case("job-3", "subject-3", "normal");
        let decided = store.open_case("job-4", "subject-4", "normal");
        store
            .decide(&decided.case_id, "match", "adj-1", None, "match", Some(0.91))
            .unwrap();

        let hits = store.cases_matching(None, Some(0.5));
        assert_eq!(hits.len(), 1, "an uncompared case has no score to filter on");
        assert_eq!(hits[0].case_id, decided.case_id);

        // A zero threshold is still a threshold: a case with no score cannot satisfy
        // one, because there is no comparison behind it to compare.
        let zero_hits = store.cases_matching(None, Some(0.0));
        assert_eq!(zero_hits.len(), 1);
        assert_eq!(zero_hits[0].case_id, decided.case_id);
        // And with no threshold the undecided case is listed.
        assert_eq!(store.cases_matching(None, None).len(), 2);
    }

    #[test]
    fn an_unknown_case_is_reported_rather_than_created() {
        let store = AdjudicationStore::new();
        assert!(store.assign("nope", "adj-1").is_err());
        assert!(store.escalate("nope", "why").is_err());
        assert!(store.attach_evidence("nope", "probe", "s3://x").is_err());
        assert!(store.case("nope").is_none());
    }
}
