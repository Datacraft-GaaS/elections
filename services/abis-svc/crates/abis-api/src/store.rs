// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! In-memory projection for the ABIS API.
//!
//! `AppState` holds a lazily-connected Postgres pool and nothing else, so the
//! handlers that needed a record built one instead:
//!
//! - `create_enrolment_job` returned a `job_id` and `status: "queued"` and stored
//!   nothing, so the job could never be read back.
//! - `get_enrolment_job` returned `subject_id: "subject-001"`,
//!   `status: "processing"` and `modalities_completed: ["fingerprint"]` for **any**
//!   id. That is a fabricated subject identity and a fabricated capture state for
//!   every enrolment job in the country.
//! - `get_quality_report` returned `fingerprint_nfiq2: 60.0`, `face_quality: 0.92`,
//!   `iris_quality: 0.88` and **`overall_pass: true`** for any id. A quality report
//!   that passes is the record on which an enrolment is allowed to proceed.
//! - `withdraw_enrolment_job` returned `status: "withdrawn"` for any id.
//! - `get_fmr_fnmr` returned `fmr: 0.001, fnmr: 0.005` — **false match and false
//!   non-match rates** — as constants, for any slice. Those two numbers are what
//!   determines whether a biometric system is fit for electoral use, and they are
//!   measured against a test corpus that is not in this workspace.
//! - `get_dedup_summary` returned zeros for any date range, which reads as "no
//!   duplicates were found" rather than "nothing was compared".
//! - `get_fairness` returned empty groups, which reads as "no disparity".
//! - `get_enrolment_throughput` returned zeroed percentiles, which reads as a
//!   measured latency distribution.
//! - `list_vendors` returned a single `Mock Vendor` reported as `healthy` with
//!   fingerprint, face and iris capabilities.
//!
//! Everything derivable from the jobs this service has accepted is derived here.
//! What is not derivable — FMR/FNMR against a test corpus, capture quality from a
//! real sensor — is reported as unmeasured, with the reason.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

/// Where an enrolment job is in its lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Accepted; no capture has started.
    Queued,
    /// A capture is under way.
    Processing,
    /// Every modality the kit holds has been captured.
    Captured,
    /// The job was withdrawn before capture completed.
    Withdrawn,
}

impl JobStatus {
    /// The wire value.
    pub fn wire(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Captured => "captured",
            Self::Withdrawn => "withdrawn",
        }
    }

    /// Whether the job can still be withdrawn.
    ///
    /// A captured job cannot: the templates are already in the system, and
    /// withdrawing it would imply they are not.
    pub fn is_withdrawable(&self) -> bool {
        matches!(self, Self::Queued | Self::Processing)
    }
}

/// One accepted enrolment job.
#[derive(Clone, Debug, Serialize)]
pub struct EnrolmentJob {
    pub job_id: String,
    /// The subject's hashed identity, as the kit reported it.
    pub subject_id: String,
    pub status: JobStatus,
    /// Modalities whose templates this service holds, in the order captured.
    pub modalities_completed: Vec<String>,
    /// The modalities the kit was asked to capture.
    pub requested_modalities: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    /// How long the job took, once captured, in milliseconds.
    pub elapsed_ms: Option<u64>,
    /// Why capture quality cannot be reported, when it cannot.
    pub blocking_reason: Option<String>,
}

/// A registered biometric vendor.
#[derive(Clone, Debug, Serialize)]
pub struct Vendor {
    pub id: String,
    pub name: String,
    /// `unregistered` unless a kit from it has enrolled anything here.
    pub status: String,
    pub capabilities: Vec<String>,
    pub enrolled_count: u64,
    pub last_seen_at: Option<String>,
}

/// Shared state for the ABIS API.
#[derive(Default)]
pub struct AbisStore {
    jobs: Mutex<HashMap<String, EnrolmentJob>>,
    /// Job ids in arrival order, so the reports have a deterministic sequence.
    order: Mutex<Vec<String>>,
    vendors: Mutex<HashMap<String, Vendor>>,
}

/// The handle threaded through the router.
pub type SharedAbisStore = Arc<AbisStore>;

impl AbisStore {
    /// Construct an empty store.
    pub fn new() -> SharedAbisStore {
        Arc::new(Self::default())
    }

    /// Accept an enrolment job.
    pub fn create_job(&self, subject_id: &str, requested_modalities: Vec<String>) -> EnrolmentJob {
        let now = chrono::Utc::now().to_rfc3339();
        let job = EnrolmentJob {
            job_id: format!("job-{}", short_id()),
            subject_id: subject_id.to_string(),
            status: JobStatus::Queued,
            modalities_completed: Vec::new(),
            requested_modalities,
            created_at: now.clone(),
            updated_at: now,
            elapsed_ms: None,
            blocking_reason: Some(
                "capture quality is produced by the kit's sensor and the matcher; neither \
                 is reachable from this process"
                    .to_string(),
            ),
        };
        self.order
            .lock()
            .expect("lock poisoned")
            .push(job.job_id.clone());
        self.jobs
            .lock()
            .expect("lock poisoned")
            .insert(job.job_id.clone(), job.clone());
        job
    }

    /// A job by id.
    pub fn job(&self, job_id: &str) -> Option<EnrolmentJob> {
        self.jobs
            .lock()
            .expect("lock poisoned")
            .get(job_id)
            .cloned()
    }

    /// Every job, in arrival order.
    pub fn jobs(&self) -> Vec<EnrolmentJob> {
        let order = self.order.lock().expect("lock poisoned");
        let guard = self.jobs.lock().expect("lock poisoned");
        order
            .iter()
            .filter_map(|id| guard.get(id).cloned())
            .collect()
    }

    /// Advance a job's state.
    pub fn set_status(&self, job_id: &str, status: JobStatus) -> Result<EnrolmentJob, String> {
        let mut guard = self.jobs.lock().expect("lock poisoned");
        let job = guard
            .get_mut(job_id)
            .ok_or_else(|| format!("no enrolment job with id {job_id}"))?;
        // A withdrawn job is a closed record.
        if job.status == JobStatus::Withdrawn {
            return Err(format!("job {job_id} was withdrawn; it does not reopen"));
        }
        job.status = status;
        job.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(job.clone())
    }

    /// Record a captured modality against a job.
    pub fn record_modality(&self, job_id: &str, modality: &str) -> Result<EnrolmentJob, String> {
        let mut guard = self.jobs.lock().expect("lock poisoned");
        let job = guard
            .get_mut(job_id)
            .ok_or_else(|| format!("no enrolment job with id {job_id}"))?;
        if job.status == JobStatus::Withdrawn {
            return Err(format!(
                "job {job_id} was withdrawn; no capture can be recorded"
            ));
        }
        if !job.requested_modalities.iter().any(|m| m == modality) {
            return Err(format!(
                "modality {modality} was not requested for job {job_id}"
            ));
        }
        if !job.modalities_completed.iter().any(|m| m == modality) {
            job.modalities_completed.push(modality.to_string());
        }
        if job.status == JobStatus::Queued {
            job.status = JobStatus::Processing;
        }
        // Every requested modality present means the capture is complete.
        if job
            .requested_modalities
            .iter()
            .all(|m| job.modalities_completed.contains(m))
        {
            job.status = JobStatus::Captured;
        }
        job.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(job.clone())
    }

    /// Withdraw a job, if it can still be withdrawn.
    pub fn withdraw(&self, job_id: &str) -> Result<EnrolmentJob, String> {
        let mut guard = self.jobs.lock().expect("lock poisoned");
        let job = guard
            .get_mut(job_id)
            .ok_or_else(|| format!("no enrolment job with id {job_id}"))?;
        if !job.status.is_withdrawable() {
            return Err(format!(
                "job is {}; a {} job cannot be withdrawn",
                job.status.wire(),
                job.status.wire()
            ));
        }
        job.status = JobStatus::Withdrawn;
        job.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(job.clone())
    }

    /// The jobs completed within the last `hours`, oldest first.
    pub fn jobs_in_window(&self, hours: i64) -> Vec<EnrolmentJob> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours);
        self.jobs()
            .into_iter()
            .filter(|j| {
                chrono::DateTime::parse_from_rfc3339(&j.created_at)
                    .map(|t| t.with_timezone(&chrono::Utc) >= cutoff)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Register a vendor, or return the one already registered.
    pub fn register_vendor(&self, id: &str, name: &str, capabilities: Vec<String>) -> Vendor {
        let mut guard = self.vendors.lock().expect("lock poisoned");
        guard
            .entry(id.to_string())
            .or_insert_with(|| Vendor {
                id: id.to_string(),
                name: name.to_string(),
                // A vendor is `unregistered` until a kit from it has enrolled
                // anything. It is never reported healthy on the strength of being
                // listed.
                status: "unregistered".to_string(),
                capabilities,
                enrolled_count: 0,
                last_seen_at: None,
            })
            .clone()
    }

    /// Record an enrolment against a vendor, which is what makes it registered.
    pub fn note_enrolment(&self, vendor_id: &str) {
        let mut guard = self.vendors.lock().expect("lock poisoned");
        if let Some(v) = guard.get_mut(vendor_id) {
            v.enrolled_count += 1;
            v.status = "registered".to_string();
            v.last_seen_at = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    /// Every registered vendor, by id.
    pub fn vendors(&self) -> Vec<Vendor> {
        let guard = self.vendors.lock().expect("lock poisoned");
        let mut items: Vec<Vendor> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.id.cmp(&b.id));
        items
    }

    /// The number of jobs this service holds.
    pub fn job_count(&self) -> usize {
        self.jobs.lock().expect("lock poisoned").len()
    }
}

/// A short id from a UUID.
///
/// The slice is taken from the **end** of the hex, not the start. A UUIDv7's
/// leading hex is its millisecond timestamp, so two ids minted in the same
/// millisecond share a prefix and slicing from the front produces collisions --
/// which is exactly what `two_jobs_have_different_subjects` caught when it found
/// both jobs reporting the second subject.
fn short_id() -> String {
    let hex = uuid::Uuid::now_v7().simple().to_string();
    hex[hex.len() - 12..].to_string()
}
