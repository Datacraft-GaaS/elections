# M13 — Public Verifier App

**Module ID:** M13
**Name:** Public Verifier App
**Owner:** Datacraft Limited — Mobile & Citizen Engagement Team
**Status:** Draft v2
**IP Classification:** Custom Software — IPR vests in the Commission per GCC 15.4 at Operational Acceptance; source released as open-source (MIT licence) per Strategy PB-02 commitment

**Strategy traceback:** Part III §PB-02, Part III §PB-03, Part IX §IX.2.A1-A3, Part XII §XII.3.2, Part XV §XV.1, Part XVII §XVII.3 Element 5

---

## Purpose

The Public Verifier App is Kenya's sovereign citizen-facing election integrity tool. Any Kenyan — smartphone or feature phone — can enter their polling station code and receive the official Form 34A tally, its cryptographic hash, and a Merkle inclusion proof verifiable against the Commission public transparency log. The app removes the information asymmetry that made the "#WapiForm34A" narrative dispositive in Raila Odinga v the Commission [2017] eKLR and converts the citizenry into 30 million independent auditors. It also carries queue status, participation token lookup (M14 integration), safety alerts (M17 integration), and grievance filing in one place.

---

## User Personas

| Persona | Device | Primary task |
|---|---|---|
| Registered Voter (smartphone) | Android / iOS | Verify own station result, check queue, file grievance |
| Registered Voter (feature phone) | 2G/3G handset | SMS/USSD short-code result query |
| WhatsApp User | WhatsApp + Android/iOS | Query via WhatsApp bot (WA Business API) |
| Party Agent / Observer | Android tablet | Live station status, Form 34A monitoring |
| Data Journalist | Web browser | Dataset download, hash verification, API access |
| IAB Member | Web browser / app | Audit feed, hash cross-check |
| Diaspora Kenyan | Web browser | Station code lookup for home constituency |
| Voter with visual disability | Screen reader (TalkBack/VoiceOver) | Audio-guided WCAG 2.2 AA workflow |

---

## Functional Requirements

### FR-13.01 — Station result lookup
A user who supplies a valid polling station code receives: candidate names, votes per candidate per contest (all six simultaneous contests), ballots issued, ballots cast, rejected ballots, the Form 34A PDF download link, the SHA-256 hash of that PDF, and the Merkle path from that hash to the published national root.

### FR-13.02 — Hash self-verification
The app independently fetches the published Merkle root from the Commission transparency log (and cross-checks against four secondary channels: GitHub, IPFS, IAB domain, Twitter/X) and displays VERIFIED / MISMATCH with explanation. Verification is client-side; no result data transits any third-party server.

### FR-13.03 — Participation token lookup
Integration with M14 reproducibility bundle: voter enters participation token received at polling station and confirms they appear in the participation roll (no voter name or ID visible in response — only "your token is recorded in station X's participation list").

### FR-13.04 — WhatsApp and SMS/USSD channels
A WhatsApp Business API bot answers station code queries with a text summary + PDF link. A USSD short-code (*the Commission# or equivalent obtained from Communications Authority) answers the same query on any 2G/3G handset with no data plan. Both channels are backed by the same API as the app.

### FR-13.05 — Live queue status
The "Queue" tab shows current estimated wait time at the voter's home station (from M19 queue data), colour-coded green/amber/red, updated every 5 minutes. Opt-in SMS alert when wait drops below 30 minutes.

### FR-13.06 — Safety tab
The "Safety" tab shows station operational status (open/closed/suspended), the nearest alternative station if suspended, and a panic button that sends an anonymous geo-tagged alert to the M16 Election Command Centre.

### FR-13.07 — Grievance filing
In-app grievance form (text + optional photo) routed to M23 grievance channel. Voter receives a ticket ID and status updates. Filing is anonymous-optional; voter can attach their participation token for follow-up.

### FR-13.08 — Learn tab
Interactive voter-education flow explaining the election process in 11 languages, including a practice ballot marking exercise and a "what happens after polls close" timeline.

### FR-13.09 — Offline mode
Core hash verification works offline if the app has cached the current Merkle root. Queue and safety data require connectivity but degrade gracefully (last known state displayed with timestamp).

### FR-13.10 — Open-source publication
Complete Android and iOS source code published under MIT licence to the Commission GitHub organisation at T-30, along with a reproducible build recipe (Docker, pinned SDK). Anyone can build and compare their binary to the published app store binary using the published APK hash.

### FR-13.11 — Binary integrity check
On each launch and every 24 hours: app computes SHA-256 of its own binary and compares against the signed hash published at `https://eel.example/verifier/integrity`. MISMATCH triggers a full-screen warning and reports to M16 SOC.

### FR-13.12 — Accessibility
Full WCAG 2.2 AA compliance. TalkBack and VoiceOver support. Font scaling up to 200% without layout breakage. High-contrast mode. Audio-guided station-code entry in EN, SW, plus 9 regional languages.

### FR-13.13 — Multilingual content
EN + SW baseline. Additional: Kikuyu, Luo, Kalenjin, Luhya, Kamba, Meru, Kisii, Somali, Turkana for all voter-facing content. Maasai for safety alerts in relevant counties.

### FR-13.14 — Downloadable open dataset
From the "Data" tab, any user can download the full signed Form 34A dataset (CSV + JSON + PDF bundle) for any constituency, with hash manifest.

---

## Non-Functional Requirements

### NFR-13.01 — Availability
99.99% during the polling window (E-Day through E+30). Served from three geographically separate read-only replicas (primary: the Commission DC; secondary: Mombasa CDN PoP; tertiary: Kampala or Dar CDN PoP). DDoS on primary does not take down service.

### NFR-13.02 — Performance
Portal page load p99 < 2 s at 10M concurrent unique users. Form 34A hash lookup p99 < 500 ms. WhatsApp/SMS response p99 < 10 s.

### NFR-13.03 — Security
Certificate pinning on API endpoint. Dual-board HSM-backed code signing. In-app binary integrity. Play Integrity API / App Attest check with visible warning (not a hard block) on non-compliant devices. Zero PII in transit for anonymous lookups. Participation token design is anonymous: no voter identity is derivable from the token.

### NFR-13.04 — Privacy (DPA 2019)
Lawful basis: Article 88 the Commission public interest mandate + voter explicit consent for participation token. No biometric data. No voter-specific PII stored in the app beyond session. Participation tokens are cryptographically unlinkable to voter identity at the server.

### NFR-13.05 — Scalability
Backend auto-scales to 30M simultaneous active sessions during peak (Form 34C publication moment). Static-first CDN for read paths; write paths (grievance filing) rate-limited.

### NFR-13.06 — Accessibility
WCAG 2.2 AA on all web and in-app surfaces. Screen-reader tested on NVDA, JAWS, VoiceOver, TalkBack. Contrast ratio >= 4.5:1 for normal text, >= 3:1 for large text.

---

## User Stories

1. As a registered voter, I want to enter my station code and see the official vote tally, so that I can confirm the result I witnessed at my polling station is the published result.
2. As a registered voter, I want to verify the Form 34A hash against the Merkle root, so that I can confirm the form has not been altered after publication.
3. As a registered voter on a feature phone, I want to text a short-code and receive the tally by return SMS, so that I can verify results without a smartphone or data plan.
4. As a party agent, I want to monitor all stations in my constituency in real time, so that I can detect transmission delays without being physically present at every station.
5. As a data journalist, I want to download the full constituency dataset in CSV format, so that I can run my own statistical analysis.
6. As a voter with visual impairment, I want audio-guided navigation in my home language, so that I can use the app independently without sighted assistance.
7. As a citizen sceptical of the result, I want to run `make reproduce` on the reproducibility bundle, so that I can re-compute the national tally from published inputs and compare it myself.
8. As a voter who witnessed a problem, I want to file a grievance anonymously with my station code, so that the issue is recorded without revealing my identity.
9. As a voter queuing, I want to see the estimated wait time at my station, so that I can decide whether to wait or return later.
10. As a diaspora Kenyan, I want to check the result from my home constituency on the web portal, so that I can follow the election in my time zone.
11. As an IAB member, I want to cross-check the Merkle root displayed in the app against my own independent source, so that I can confirm the app is serving authentic data.
12. As a first-time voter, I want to complete a practice ballot-marking exercise in the app, so that I know how to mark my ballot correctly before election day.

---

## Data Model Additions

```
VerificationQuery:
  id: str  # UUID7
  station_code: str
  query_type: enum[TALLY, HASH, PARTICIPATION_TOKEN, QUEUE]
  channel: enum[APP_ANDROID, APP_IOS, WEB, WHATSAPP, SMS_USSD]
  session_token: str  # anonymous session, rotates per session
  timestamp_utc: datetime
  response_latency_ms: int
  result_status: enum[FOUND, NOT_YET_PUBLISHED, NOT_FOUND, MISMATCH]

ParticipationTokenQuery:
  id: str  # UUID7
  token_hash: str  # SHA-256 of participation token — token itself not stored
  station_code: str
  result: enum[FOUND_IN_ROLL, NOT_FOUND, STATION_NOT_YET_CLOSED]
  timestamp_utc: datetime

GrievanceSubmission:
  id: str  # UUID7
  station_code: str | None
  category: enum[KIT_FAILURE, FORM_DISPUTE, QUEUE, INTIMIDATION, OTHER]
  description: str
  photo_hash: str | None
  channel: enum[APP, WEB, WHATSAPP, SMS]
  anonymous: bool
  participation_token_hash: str | None
  timestamp_utc: datetime
  ticket_id: str  # routed to M23
```

---

## API Surface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/stations/{code}/result` | None (public) | 100/s per IP | Returns tally JSON + hash + Merkle path |
| GET | `/api/v1/stations/{code}/form34a.pdf` | None (public) | 20/s per IP | Signed PDF |
| GET | `/api/v1/stations/{code}/queue` | None (public) | 60/s per IP | Current queue status from M19 |
| POST | `/api/v1/participation/verify` | None (public) | 10/s per IP | Body: `{token_hash, station_code}` |
| GET | `/api/v1/merkle/root` | None (public) | 1000/s | Current published root + timestamp |
| GET | `/api/v1/merkle/path/{form_hash}` | None (public) | 50/s per IP | Merkle inclusion proof |
| POST | `/api/v1/grievances` | None (public) | 5/s per IP | Submit grievance; returns ticket_id |
| GET | `/api/v1/grievances/{ticket_id}` | Ticket token | 20/s | Status of submitted grievance |
| GET | `/api/v1/datasets/{constituency_code}` | None (public) | 5/s per IP | CSV + JSON bulk download |
| GET | `/api/v1/integrity/hash` | None (public) | 1000/s | Published app binary hash |
| POST | `/webhooks/whatsapp` | HMAC-SHA256 | 500/s | WA Business API inbound |
| POST | `/webhooks/ussd` | API key | 200/s | Aggregator USSD callback |

All endpoints: OpenAPI 3.1 spec at `/api/v1/openapi.json`. TLS 1.3. CORS: allow all origins (public read API). No authentication on public read paths. SSE stream at `/api/v1/stations/{code}/result/stream` for live tally updates.

---

## Integration Points

**Upstream:**
- M10 (RTS Back-End) — Form 34A JSON and PDF feed via internal event bus
- M11 (Public Results Portal) — Merkle root, hash publication
- M14 (Reproducibility Bundle) — participation token roll
- M19 (Queue Transparency) — per-station queue metrics
- M17 (Speech Integrity) — safety incident feed for Safety tab
- M23 (Grievance channels) — ticket creation and status
- M16 (Election Command Centre) — safety panic button routing

**Downstream (consumers):**
- Independent observers (ELOG, Carter Center, EU EOM) via public API
- Media consortium via bulk dataset endpoint
- IAB via authenticated read-only feed
- Supreme Court IT Registrar via archived snapshots

**Third-party:**
- WhatsApp Business API (Meta) — inbound/outbound WA messages
- SMS aggregator (Safaricom / Airtel) — outbound SMS delivery for USSD fallback
- Communications Authority of Kenya (CA) — non-premium USSD short-code assignment (e.g. *the Commission# or *400#); CA type-approval for the USSD application; SLA: CA short-code assignment requires formal written application 90 days before go-live; the Commission to submit application at T-270 minimum; type-approval clearance typically 30 days; CA also coordinates inter-operator USSD routing; dependency tracked in M13 rollout milestone at T-240
- Google Play Store + Apple App Store — distribution
- IPFS node — Merkle root cross-publication

---

## UI/UX — Key Screens

1. **Home** — voter's station card (name, location, queue), quick-verify CTA, election day countdown / live status indicator
2. **Verify Result** — station code entry, instant result display, hash verification badge (green checkmark / red alert), Merkle path expandable
3. **Participation Token** — token entry, confirmation that token is found in the station roll
4. **Queue Status** — colour-coded wait-time gauge, trend chart, SMS alert opt-in
5. **Safety** — station status map, alternate station finder, panic button
6. **Grievance** — category picker, description, optional photo attachment, anonymous toggle, ticket ID confirmation
7. **Learn** — interactive ballot tutorial (11 languages), "what happens after polls close" timeline
8. **Data** — constituency dataset download, reproducibility bundle instructions
9. **Settings** — language selector (11 options), notification preferences, high-contrast toggle, font size

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Verifiable and transparent election method | Constitution Art. 86(a) |
| the Commission public accountability | Constitution Art. 88(4) |
| Voter right to information | Access to Information Act 2016 s.4 |
| Anonymous participation (ballot secrecy) | Constitution Art. 86(d); Elections Act s.53 |
| Data minimisation for anonymous lookups | DPA 2019 s.26 |
| Accessibility for voters with disability | Constitution Art. 54; Art. 82 |
| Grievance channel requirement | Elections Act s.78; the Commission Act s.12 |
| No biometric data in citizen-facing app | DPA 2019 s.47 (biometric as sensitive category) |

---

## Success Metrics and SLA

| Metric | Target | Measurement | Sign-off |
|---|---|---|---|
| Unique verifications on E-Day + 7 days | > 5 million | API log count | the Commission Chair |
| App store downloads by E-Day | > 2 million | Play/App Store analytics | Datacraft |
| WhatsApp + SMS queries on E-Day | > 2 million combined | Aggregator logs | Datacraft |
| Portal availability during E-Day through E+30 | >= 99.99% | Uptime robot (3 replica checks) | the Commission ICT |
| Hash verification VERIFIED rate | 100% for all published forms | Automated test | IAB |
| Grievance acknowledgement SLA | 100% within 2 hours | Ticket system | M23 owner |
| Form 34A available on app within X of upload | <= 30 min | Timestamp diff | the Commission ICT |
| p99 tally lookup latency | < 500 ms | APM (OpenTelemetry) | Datacraft SRE |

---

## Test Strategy

- **Unit:** Business logic (hash verification, Merkle path validation, token anonymisation) >= 90% coverage
- **Integration:** M10 feed consumption, M19 queue feed, M23 grievance routing
- **E2E:** Playwright tests simulating voter journey on Android emulator and iOS simulator in CI
- **Performance/load:** k6 load test simulating 10M concurrent API calls at Form 34C publication event; 3-replica failover test
- **Security:** OWASP Mobile Top 10 pen test by independent firm at T-90; binary integrity check tested with tampered APK; certificate pinning bypass attempt
- **Accessibility:** axe-core automated scan in CI; manual screen-reader test on TalkBack/VoiceOver; WCAG 2.2 AA audit by independent accessibility firm at T-60
- **Language/localisation:** Translation review by native speakers; RTL support test if Somali display requires
- **Chaos:** Primary API replica killed; app must fall back to secondary within 5 seconds

---

## Rollout

| Phase | Date | Scope |
|---|---|---|
| Alpha (internal Datacraft) | 2026-11-01 | Core verify + hash check, EN only |
| Beta (the Commission + IAB) | 2027-01-15 | All tabs; EN + SW; 10 testers |
| Public pilot (Nairobi + 2 counties) | 2027-03-01 | 100K users; all 11 languages |
| Dry-run election | 2027-06-10 (T-60) | Full national; all channels live |
| General availability | 2027-07-11 (T-30) | App stores; SMS; WhatsApp |
| Election Day | 2027-08-10 | Full ops; 24x7 SOC |
| Post-election | 2027-08-11 to 2027-09-10 | Petition-window availability |

---

## Dependencies

- **M10** (RTS Back-End) — real-time Form 34A feed
- **M11** (Public Results Portal) — Merkle root publication
- **M14** (Reproducibility Bundle) — participation token issuance
- **M16** (Election Command Centre) — safety alert routing
- **M17** (Speech Integrity) — safety incident feed
- **M19** (Queue Transparency) — queue metrics
- **M23** (Grievance Channels) — ticket API
- **P06** (HSM) — Merkle root signing; binary code signing
- **P04** (IAM) — IAB authenticated read feed
- **M29** (Mobile Citizen Verify) — shares the same Rust client-side verification logic; the Rust verifier crate must be extracted into a shared library `libcommission-verifier` (Rust FFI + Android JNI + iOS XCFramework) per MOBILE-INTEGRATION-NOTES.md §8 before either M13 or M29 ships, to avoid divergent verification behaviour
- External: WhatsApp Business API; CA non-premium short-code; Google Play / Apple App Store

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Clone apps in Play/App Store spreading disinformation | Weekly automated clone-scan; CA/the Commission pre-agreed expedited takedown; official install links on DNSSEC-protected domain |
| R2 | DDoS at Form 34C publication | 3-replica CDN-first architecture; upstream scrubbing; Cloudflare or equivalent |
| R3 | WhatsApp Business API quota exceeded at peak | Pre-negotiated higher tier quota; SMS as fallback for all WA users |
| R4 | Merkle root cross-publication channel failure | 5 independent channels; app verified against all 5; single-channel failure is non-critical |
| R5 | Accessibility gap in a minority language | Native-speaker review pre-launch; in-app feedback channel for language errors; 72-hour correction SLA |
