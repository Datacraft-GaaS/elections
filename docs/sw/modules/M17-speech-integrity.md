# M17 — Speech Integrity

**Module ID:** M17
**Name:** Speech Integrity System
**Owner:** Datacraft Limited — AI & Data Science Team
**Status:** Draft v2
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Part X §X.2 (D1-D4), Part XVI §XVI.2, Part XVI §XVI.3, Part XVI §XVI.4, Part XVI §XVI.6

---

## Purpose

The Speech Integrity System monitors Kenya's online and broadcast information environment for hate speech, coordinated inauthentic behaviour (CIB), deepfakes, voter-suppression signals, and violent-disruption indicators that have historically corrupted election legitimacy and driven post-election violence (2007-2008 PEV; 2013 Tana River; 2017 Baby Pendo; Chris Msando murder 31 July 2017). The system is AI-assisted, human-adjudicated: detection models flag content for human analysts who make all consequential decisions. Evidence packages for NCIC, DCI, and ODPP referrals are auto-generated with preserved chain-of-custody. A prebunking network distributes counter-narrative content before harmful narratives take hold. No AI determination is final; no content is suppressed without a human decision. The module integrates with M16 (ECC alert routing) and M13 (voter Safety tab).

---

## User Personas

| Persona | Role |
|---|---|
| CAOC Hate-Speech Analyst (x3, language clusters) | Content monitoring, flag triage, referral authorisation |
| CAOC Disruption Analyst (x2, regional clusters) | Safety-incident monitoring, KNPS alert routing |
| IEBC Communications Officer | Rapid-response comms; fact-check approval |
| NCIC Liaison Officer | Legal referral recipient; incitement response coordination |
| DCI CIRT-CC Liaison | Cybercrime referral; CIB takedown coordination |
| ODPP Liaison | Evidence package receipt for prosecution |
| IAB Liaison | Read-only monitoring; quarterly report input |
| Civil Society Partner (ELOG, Amnesty, KHRC) | Field-incident reporting; observer input |
| Voter (via M13 Safety tab) | Panic button; safety alert recipient |

---

## Functional Requirements

### FR-17.01 — Multi-platform content ingestion
Public firehose ingestion from: Twitter/X (Academic API), Facebook/Meta (CrowdTangle successor), TikTok (Research API), YouTube (Data API v3), Telegram public channels, WhatsApp public groups (where legally accessible), Kenyan news comment sections (Nation, Standard, Citizen Digital, TUKO). Ingestion in 11 languages: English, Swahili, Sheng, Kikuyu, Luo, Kalenjin, Luhya, Kamba, Meru, Kisii, Somali.

### FR-17.02 — Hate-speech classification
Kenyan-context hate-speech classifier trained on annotated Kenyan election corpus (in partnership with NCIC and iHub/Ushahidi Umati dataset lineage). Outputs: ethnicity-based incitement score, candidate-targeted intimidation score, false-information score, voter-suppression signal score. Threshold: HIGH flag = p >= 0.85; MEDIUM = p >= 0.60; LOW = p >= 0.40. Human analyst reviews all HIGH and MEDIUM flags.

### FR-17.03 — CIB detection
Coordinated inauthentic behaviour detection: account-creation velocity clustering, cross-platform amplification graph analysis, engagement-pattern anomalies, bot-score integration. Model: graph neural network on account-interaction graph + velocity model on posting patterns. HIGH flags reviewed by DCI CIRT-CC liaison within 60 minutes. Evidence package auto-generated for any cluster with > 100 coordinated accounts.

### FR-17.04 — Deepfake detection
Video and audio deepfake detection pipeline: FaceForensics-family CNN for visual deepfakes; voice-clone detection via mel-spectrogram analysis. Applied to viral content (> 10,000 engagements) featuring apparent IEBC officials, political figures, or election-critical claims. Confidence score reported with every flag. Human analyst confirms before any action taken. Detection accuracy target: > 95% on known deepfakes.

### FR-17.05 — Voter-suppression signal detection
Classifier for false closure claims ("Station X is closed"), false eligibility claims ("your ID is invalid"), false security-threat claims ("area X is dangerous, do not go to vote"). Tuned on Kenyan election suppression case studies. HIGH flags routed to IEBC Communications for immediate counter-message + NCIC for legal consideration.

### FR-17.06 — Violent-disruption detection (Part XVI.3)
Multi-source incident feed: social media pipeline (FR-17.01), citizen-reported incidents via M13 panic button, KHRC/Amnesty/KEOG field reports, NPS SITREPs (via IEBC-KNPS MoU). Alert cascade:

| Level | Trigger | Response | Time |
|---|---|---|---|
| MINOR | Single unverified incident | Aggregated hourly | Hourly |
| SIGNIFICANT | Verified incident / pattern | Alert to IEBC Chair + KNPS + IAB | < 30 min |
| MAJOR | Multi-location or casualty | Public dashboard update | < 10 min |
| CATASTROPHIC | Mass-casualty event or polling shutdown | IEBC Chair + AG + KNPS IG + AU | Immediate |

### FR-17.07 — Rapid-response workflow
HIGH hate-speech and CIB flags: routed to NCIC + DCI + IEBC Communications within 15 minutes of human analyst confirmation. MEDIUM flags: within 60 minutes. LOW flags: daily situation report. Evidence package auto-generated per referral: URL, timestamp, SHA-256 hash of content at time of capture, screenshot (perceptual hash), cross-platform propagation trace, analyst ID, decision rationale. Package structured for admissibility under NCI Act 2008 s.13.

### FR-17.08 — Prebunking network
Pre-election explainer content addressing the top 20 known misinformation narratives (seeded from 2017 and 2022 post-election analysis + Oxford Internet Observatory Kenya reports). Distributed via: IEBC verified social accounts, community-influencer network (50 county-level trusted voices), radio partnerships, M13 Learn tab, WhatsApp official channel. Content produced in 11 languages. Prebunking reach target: > 60% of registered voters reached before E-Day.

### FR-17.09 — Fact-check desk
CAOC fact-check liaison receives flags for time-sensitive false claims. Target: public correction within 30 minutes of HIGH-flag human confirmation. Correction published simultaneously via: IEBC Twitter/X, WhatsApp, M13 notification push, journalist feed, radio (pre-agreed slots). Correction logs provenance chain (original claim hash, correction timestamp, channels published).

### FR-17.10 — Safety alert feed to M13
Validated safety incidents (SIGNIFICANT and above) are pushed to M13's Safety tab as geo-tagged alerts visible to voters near the affected location. Voter can see station closure status and alternate station guidance. No unvalidated alerts are pushed; human analyst sign-off required.

### FR-17.11 — Evidence archive
All captured content (screenshots, URLs, timestamps, hashes, analysis metadata) stored in tamper-evident append-only store. Retained for 7 years. Chain-of-custody preserved for potential criminal proceedings. DPA 2019 lawful basis: public interest (NCI Act s.13; Elections Act s.7-8).

### FR-17.12 — Real-time NLP sentiment monitoring
NLP pipeline on ingested content: net sentiment toward IEBC, election process, KIEMS technology. Baseline established at T-90. Alert when net sentiment drops > 20% from baseline during results window (possible narrative attack signal). Metric reported in M16 ECC CAOC workstation and IAB Dashboard.

---

## Non-Functional Requirements

### NFR-17.01 — Throughput
Ingestion pipeline: >= 100,000 items/hour across all platforms during peak. Classification latency: < 5 seconds per item end-to-end (ingestion to analyst inbox).

### NFR-17.02 — Detection accuracy
Hate-speech HIGH-flag precision >= 80%; recall >= 90% on Kenyan test set. Deepfake detection accuracy > 95% on known deepfakes. False-positive rate on hate-speech classification < 5%.

### NFR-17.03 — Availability
Ingestion and classification pipeline: 99.9% during T-90 through T+30. Alert delivery to NCIC/DCI: 99.9% within SLA.

### NFR-17.04 — Sovereignty
All models run on Kenyan-jurisdiction infrastructure. No Kenyan personal data (including content posted by identifiable Kenyans) transits a foreign cloud endpoint. Training data curated under NCIC oversight. All model cards published to IAB.

### NFR-17.05 — Human-in-loop enforcement
No automated referral to NCIC/DCI/ODPP without human analyst confirmation. No content classified as illegal without human sign-off. No voter-facing safety alert without CAOC analyst approval. All human decisions logged with analyst ID and rationale.

### NFR-17.06 — Privacy (DPA 2019)
Processing of public social media content: lawful basis is public interest (NCI Act s.13; Constitution Art. 33(2)(a)). No processing of private communications. No profiling of individuals beyond what is necessary for fraud/CIB detection. DPIA published pre-launch. Retention limited to 7 years for evidence; metadata retained permanently for audit.

---

## User Stories

1. As a CAOC Hate-Speech Analyst, I want to review HIGH-flag content in a triage queue with one-click referral to NCIC, so that I can process high-priority flags within the 15-minute SLA.
2. As the IEBC Communications Officer, I want to see the top 5 active false narratives in real time, so that I can prioritise counter-messaging resources.
3. As an NCIC Liaison, I want to receive court-ready evidence packages automatically when I accept a referral, so that legal action can proceed without manual evidence gathering.
4. As an IAB Member, I want to review the detection model cards and confidence distributions, so that I can assess the reliability of the system's outputs for the quarterly report.
5. As a voter near an incident area, I want to see a safety alert on the M13 Safety tab, so that I can make an informed decision about whether to travel to my polling station.
6. As a DCI CIRT-CC Liaison, I want to receive CIB cluster reports with account lists and propagation graphs, so that I can coordinate platform takedown requests.
7. As the CAOC Disruption Analyst, I want to trigger a MAJOR alert to the IEBC Chair within 10 minutes of a confirmed multi-location incident, so that the Commission can respond before the situation escalates.
8. As a civil society field observer, I want to submit an incident report that feeds directly into the disruption-detection pipeline, so that on-the-ground intelligence reaches decision-makers without delay.

---

## Data Model Additions

```python
class ContentFlag(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    platform: str
    content_url: str
    content_hash_sha256: str
    screenshot_hash: str
    captured_at: datetime
    language: str
    detection_type: str  # hate_speech | cib | deepfake | suppression | disruption
    confidence: float
    severity: str  # HIGH | MEDIUM | LOW
    analyst_id: str | None
    analyst_decision: str | None  # confirm | dismiss | escalate
    analyst_at: datetime | None
    referral_id: str | None

class SafetyIncident(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    level: str  # MINOR | SIGNIFICANT | MAJOR | CATASTROPHIC
    source: str  # social_media | field_report | m13_panic | knps_sitrep
    lat: float | None
    lng: float | None
    county_code: str | None
    constituency_code: str | None
    description: str
    verified_by: str | None
    verified_at: datetime | None
    alert_sent_at: datetime | None
    m13_alert_pushed: bool

class EvidencePackage(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    package_type: str  # ncic_referral | dci_referral | odpp_referral
    flag_ids: list[str]
    generated_at: datetime
    generated_by: str
    package_hash: str
    submitted_to: str
    submitted_at: datetime | None
    case_reference: str | None
```

---

## API Surface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/flags` | Analyst JWT | 50/s | Triage queue; filterable |
| POST | `/api/v1/flags/{id}/decision` | Analyst JWT | 20/s | Confirm/dismiss/escalate |
| POST | `/api/v1/incidents` | Field reporter JWT / M13 | 10/s | Submit safety incident |
| GET | `/api/v1/incidents` | Analyst JWT | 50/s | Incident feed |
| POST | `/api/v1/incidents/{id}/alert` | CAOC Analyst JWT | 5/s | Push to M13 Safety tab |
| POST | `/api/v1/evidence/generate` | Analyst JWT | 2/s | Generate evidence package |
| GET | `/api/v1/sentiment/trend` | Operator JWT | 20/s | NLP sentiment trend |
| GET | `/api/v1/prebunking/coverage` | IEBC Comms JWT | 10/s | Prebunking reach metrics |
| SSE | `/api/v1/flags/stream` | Analyst JWT | 1 per session | Real-time flag stream |

---

## Integration Points

**Upstream:**
- Twitter/X Academic API, Facebook API, TikTok Research API, YouTube Data API — content ingestion
- M13 (Public Verifier App) — panic button submissions; Safety tab push
- KEOG/ELOG/KHRC/Amnesty — field-incident reports (authenticated partner API)
- KNPS — NPS SITREPs (via MoU-established API)
- P05 (Audit log) — all analyst decisions and evidence packages

**Downstream:**
- NCIC — referral packages (SFTP + PGP; or dedicated secure portal)
- DCI CIRT-CC — CIB cluster reports
- ODPP — evidence packages for prosecution
- M16 (ECC / CAOC workstation) — alert and sentiment feed
- M15 (IAB Dashboard) — system status and model-card publication
- M13 (Public Verifier App) — Safety tab alerts

---

## UI/UX — Key Screens

1. **Triage queue** — severity-sorted content flags; thumbnail + text excerpt; one-click confirm/dismiss/escalate; SLA timer per item
2. **CIB graph explorer** — network graph of coordinated accounts; timeline of activity; account cluster detail
3. **Deepfake review** — side-by-side authentic vs flagged video; confidence score; analyst decision log
4. **Safety incident map** — geo-heatmap of incidents by level; drill-down to incident detail; alert dispatch panel
5. **Prebunking dashboard** — top active narratives ranked by spread; counter-content status per narrative; reach metrics
6. **Evidence package builder** — flag selection; package preview; referral destination selection; one-click submit
7. **Sentiment trend chart** — 7-day net sentiment toward IEBC/elections; baseline vs current; anomaly highlights
8. **Model health panel** — per-model precision/recall on daily evaluation set; false-positive rate; drift indicators

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Hate speech monitoring and referral | NCI Act 2008 s.13, s.62; Constitution Art. 33(2)(a) |
| Voter suppression prevention | Elections Offences Act 2016 s.7-s.8 |
| Deepfake/CIB cybercrime referral | Computer Misuse and Cybercrimes Act 2018 s.22-s.23 |
| Evidence chain-of-custody | Evidence Act Cap. 80 s.106A; NCI Act s.13 |
| Human-in-loop for speech classification | No statute mandates AI-only; best practice + DPA 2019 s.32 (automated decisions) |
| DPA 2019 — processing of public social media | DPA 2019 s.30 (public interest basis); DPIA published |
| Safety duties to voters | Constitution Art. 26 (right to life); IEBC Act s.4 |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| HIGH-flag human review and decision | 100% within 15 min | CAOC Analyst |
| MEDIUM-flag review | 100% within 60 min | CAOC Analyst |
| NCIC/DCI referral transmission | 100% within 15 min of analyst confirmation | IAB |
| Prebunking reach | > 60% of registered voters | IEBC Comms |
| Deepfake detection accuracy | > 95% on known test set | IAB Technical Member |
| Hate-speech false-positive rate | < 5% | IAB Technical Member |
| CATASTROPHIC incident alert time | < 10 min | ECC Director |
| Safety alert delivery to M13 | p99 < 60 s after analyst approval | Datacraft SRE |

---

## Test Strategy

- **Model evaluation:** Monthly evaluation on held-out Kenyan annotated dataset from T-365 to T-30; model drift monitoring in production
- **Integration:** M13 panic button to Safety tab end-to-end test; NCIC evidence-package delivery test
- **Load:** 100,000 items/hour ingestion test during stress period T-30 to E-Day
- **Human-in-loop enforcement:** Automated test that no referral can be generated without analyst `decision = confirm` record in audit log
- **Privacy:** DPA 2019 DPIA completed pre-launch; personal data audit confirming no private communication content ingested
- **Chaos:** Platform API failure simulation (Twitter/X goes offline); pipeline continues from remaining sources without error
- **Prebunking A/B test:** Test two prebunking message variants on pilot county sample at T-90; measure narrative-recall improvement

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| Data partnership agreements signed | 2026-11-01 | Twitter/X, Facebook, TikTok API access |
| Kenyan corpus annotation (NCIC partnership) | 2027-01-01 | 50,000 annotated items |
| Model training and evaluation | 2027-02-01 | Model cards published to IAB |
| CAOC analyst recruitment and training | 2027-03-01 | 8 analysts (3 language clusters × 2 shifts, + 2 disruption) |
| Prebunking content production (11 languages) | 2027-04-01 | 20 narrative counter-sets |
| Live monitoring begins (T-90) | 2027-05-11 | Pipeline active |
| Prebunking distribution begins (T-90) | 2027-05-11 | All channels live |
| Election Day ops | 2027-08-10 | Full 24x7 CAOC |
| Post-election archiving | 2027-09-10 | 7-year evidence archive confirmed |

---

## Dependencies

- **M13** (Public Verifier App) — panic button source; Safety tab push destination
- **M16** (ECC/CAOC workstations) — alert routing, incident feed display
- **M15** (IAB Dashboard) — model card publication, system health
- **P05** (Audit log) — all analyst decisions
- **P04** (IAM) — analyst role authentication
- External: Twitter/X Academic API; Facebook API; TikTok Research API; YouTube Data API; NCIC; DCI CIRT-CC; KNPS MoU

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Social media platform API access revoked during election window | Pre-signed data-sharing agreements with legal clauses; IPFS caching of captured public content; web-scraping fallback for public pages |
| R2 | Deepfake detection evaded by state-level adversary | Best-available model deployed; honest limitation disclosed in IAB model card; human analyst reviews all high-engagement flagged content regardless |
| R3 | Analyst overwhelmed by volume during peak (election night) | Tiered alert queue prioritises by engagement velocity; LOW flags batched for daily report; reinforcement analyst on standby |
| R4 | Prebunking content accused of being government propaganda | Content produced jointly with NCIC + civil society (ELOG, iHub); clearly attributed to partner organisations; no IEBC branding on prebunking content |
| R5 | Evidence package chain-of-custody challenged in court | Hash of original content captured at ingestion; screenshot perceptual hash; timestamp from platform API; preserved in WORM store; IAB co-signs evidence packages |
