-- Copyright (c) 2026 Nyimbi Odero
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

-- M41 post-election-review schema
CREATE TABLE IF NOT EXISTS findings (
    id              TEXT PRIMARY KEY,
    election_cycle  TEXT NOT NULL,
    submitter_category TEXT NOT NULL,
    submitter_id    TEXT,
    finding_type    TEXT NOT NULL,
    severity        TEXT NOT NULL CHECK (severity IN ('HIGH','MEDIUM','LOW')),
    description     TEXT NOT NULL,
    evidence_refs   TEXT,
    recommended_action TEXT,
    is_long_drip    BOOLEAN NOT NULL DEFAULT FALSE,
    duplicate_of    TEXT,
    status          TEXT NOT NULL DEFAULT 'SUBMITTED' CHECK (status IN ('SUBMITTED','REVIEWED','APPROVED','CLOSED')),
    submitted_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommendations (
    id                      TEXT PRIMARY KEY,
    finding_ids             TEXT NOT NULL,
    recommendation_text     TEXT NOT NULL,
    eel_response_type       TEXT CHECK (eel_response_type IN ('ACCEPTED','ACCEPTED_WITH_MODIFICATION','DECLINED','DEFERRED')),
    eel_response_rationale  TEXT,
    owner_dept_id           TEXT,
    implementation_date     TEXT,
    requires_legislation    BOOLEAN NOT NULL DEFAULT FALSE,
    parliament_referral_id  TEXT,
    chair_signed_at         TIMESTAMPTZ,
    chair_signed_by         TEXT,
    status                  TEXT NOT NULL DEFAULT 'DRAFT',
    next_cycle_handoff_at   TIMESTAMPTZ,
    last_long_drip_review   TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS parliament_referrals (
    id                      TEXT PRIMARY KEY,
    recommendation_ids      TEXT NOT NULL,
    referral_package_hash   TEXT,
    chair_signed_at         TIMESTAMPTZ,
    clerk_received_at       TIMESTAMPTZ,
    parliament_response     TEXT CHECK (parliament_response IN ('ADOPTED','REJECTED','DEFERRED')),
    bill_reference          TEXT,
    parliament_response_at  TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS long_drip_alerts (
    id                      TEXT PRIMARY KEY,
    finding_id              TEXT NOT NULL REFERENCES findings(id),
    review_due_at           TIMESTAMPTZ NOT NULL,
    reviewer_id             TEXT,
    reviewed_at             TIMESTAMPTZ,
    escalated_to_commissioner BOOLEAN NOT NULL DEFAULT FALSE,
    escalated_at            TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_findings_status ON findings(status);
CREATE INDEX IF NOT EXISTS idx_findings_severity ON findings(severity);
CREATE INDEX IF NOT EXISTS idx_long_drip_due ON long_drip_alerts(review_due_at);
