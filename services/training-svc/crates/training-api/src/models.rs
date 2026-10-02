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

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// CMS (M37)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub id: Uuid,
    pub title: String,
    pub language: String,
    pub format: String,
    pub topic_category: String,
    pub status: String,
    pub expiry_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItemCreateReq {
    pub title: String,
    pub language: String,
    pub format: String,
    pub topic_category: String,
    pub target_audience: String,
    pub election_cycle_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactCheckDecisionReq {
    pub decision: String,
    pub annotations: serde_json::Value,
}

// LMS (M38)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseModule {
    pub id: Uuid,
    pub title: String,
    pub target_audience: String,
    pub module_type: String,
    pub pass_mark_pct: u8,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSubmitReq {
    pub answers: serde_json::Value,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCertificate {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub certificate_hash: String,
    pub qr_verification_url: String,
    pub issued_at: String,
    pub expires_at: String,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationStatus {
    pub worker_id: Uuid,
    pub status: String, // CERTIFIED/REFRESHER_PENDING/NOT_CERTIFIED
}

// Accreditation (M42+M43)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAccreditationReq {
    pub applicant_name: String,
    pub id_number: String,
    pub organisation_id: Uuid,
    pub mck_membership_number: String,
    pub access_zones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeVerifyResp {
    pub serial: String,
    pub status: String,
    pub access_zones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrlResp {
    pub generated_at: String,
    pub revoked_serials: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverApplicationReq {
    pub organisation_id: Uuid,
    pub full_name: String,
    pub id_number: String,
    pub nationality: String,
    pub deployment_county_codes: Vec<String>,
}

// Medical (M60)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalIncidentReq {
    pub station_id: String,
    pub incident_type: String,
    pub severity: String,
    pub persons_affected: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalIncident {
    pub id: Uuid,
    pub station_id: String,
    pub incident_type: String,
    pub severity: String,
    pub status: String,
    pub reported_at: String,
    pub persons_affected: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchReq {
    pub dispatched_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterAlert {
    pub id: Uuid,
    pub county_code: String,
    pub incident_count: u32,
    pub alert_threshold: u32,
    pub raised_at: String,
}
