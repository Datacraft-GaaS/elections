# Client Excellence Strategy — IEBC IEMS 2026-2027
## Tender IEBC/OIT/01/2026-2027 | Datacraft × Mechrevo Consortium

**Prepared:** 2026-08-20 | **v3 augmented:** 2026-08-20 — Parts VIII-XXI added (14 augmentation modules, 24-item gap register). Executive Summary v3.1 at end of document.  
**Classification:** Internal strategy; not for distribution outside the bid team.

---

## Part I. Diagnostic: What Has Actually Gone Wrong

### 1.1 Petition-Ground Taxonomy (2013–2022)

| Ground | 2013 | 2017 | 2022 | Frequency | Severity |
|---|---|---|---|---|---|
| Results transmission failure / gaps | Yes (RTS crash) | Yes (Forms 34A/B not matching) | Partial (Cherera 4 dissent) | 3/3 | CRITICAL |
| Unsigned or defective result declaration forms | Partial | Yes (Form 34A/B/C) | Yes (Cherera 4) | 3/3 | CRITICAL |
| Server access denied / chain-of-custody refusal | No | Yes (IEBC "declined" Supreme Court IT scrutiny) | Partial | 2/3 | HIGH |
| KIEMS kit failure rate on election day | Yes (EVID 7.5% functional) | Partial (kit failures triggered manual fallback) | Minor | 2/3 | HIGH |
| Procurement irregularity (direct award, overrun) | Yes (BVR G2G) | Yes (OT-Morpho one-day tender) | Yes (Smartmatic PPARB challenge) | 3/3 | HIGH |
| Biometric deduplication failures (duplicate records) | Yes (BVR duplicates) | Partial | Low | 2/3 | MEDIUM |
| Voter register anomalies (ghost voters, under-registration) | Yes | Yes | Partial | 3/3 | MEDIUM |
| Physical chain-of-custody (ballot transport, kit location) | Partial | Yes | Partial | 2/3 | MEDIUM |
| Result portal (RTS) unavailability / defacement | No | Yes | Minor | 1/3 | MEDIUM |
| Party-agent access denial (KIEMS logs, data-centre) | No | Yes | Yes (Smartmatic Venezuelan staff) | 2/3 | HIGH |
| Poll-worker recruitment integrity (ghost workers, training gaps) | No | Partial | Partial | 1/3 | LOW |
| Insider threat / assassination (Chris Msando, Aug 2017) | No | Yes | No | 1/3 | CRITICAL |
| Ballot paper chain-of-custody (Al Ghurair) | No | Yes (AfriCOG) | Suspected | 1/3 | MEDIUM |
| ABIS deduplication audit gap | No | No | Yes (audit 2024) | 1/3 | MEDIUM |

**Primary sources for grounds above:** Raila Odinga & 5 Others v IEBC [2013] eKLR (Petition 5 of 2013, dismissed, technology failure cited but not dispositive); Raila Amolo Odinga v IEBC [2017] eKLR (Petition 1 of 2017, annulment, paras 67-121 and 219-272 cover Forms 34A/B transmission, server-access refusal, chain-of-custody); Raila Odinga v William Ruto [2022] eKLR (Petition E005 of 2022, dismissed, Cherera 4 dissent paras 1-47, Smartmatic employee nationality ground); AfriCOG (2018) dark-business report; OAG special audit 2017; OAG report 2024.

---

### 1.2 Public Chatter Topology (Top 15 Grievance Clusters)

| # | Grievance | Primary carriers | Volume signal | Stakeholder weight |
|---|---|---|---|---|
| 1 | "KIEMS failed and they switched to manual" | Voters, civil society (ELOG, CRECO) | Very high — repeats every cycle | HIGH (public trust) |
| 2 | "Where are the Form 34As?" (#WapiForm34A) | Twitter/X, Nation, Standard | Very high 2017; persistent 2022 | CRITICAL (legal standing) |
| 3 | "The results portal was down / hacked" | Citizen Digital, TUKO, Nairobi News | High 2017 | HIGH (media narrative) |
| 4 | "Foreign nationals ran our election system" | Standard Media 2022; Twitter/X | Very high 2022 (Smartmatic Venezuelan staff) | HIGH (sovereignty) |
| 5 | "IEBC can't account for the servers" | The Elephant, Africa Uncensored | Medium-high | HIGH (judiciary interest) |
| 6 | "Commissioners disagreed — election was stolen" | Nation, The Star, Twitter/X | Very high 2022 (Cherera 4) | CRITICAL (legitimacy) |
| 7 | "Procurement was rigged to favour X" | AfriCOG, KHRC, PAC proceedings | Medium-high | HIGH (OAG, Parliament) |
| 8 | "The biometric register has duplicates" | EACC, civil society | Medium | MEDIUM |
| 9 | "Party agents were locked out of the tallying centre" | NASA 2017, Azimio 2022 | High | HIGH (legal petition fuel) |
| 10 | "Chris Msando was killed — the system is compromised" | Nation investigations, KHRC | Persistent low hum | CRITICAL (security / impunity) |
| 11 | "Al Ghurair printed bad ballot papers" | AfriCOG, Twitter/X | Medium 2017 | MEDIUM |
| 12 | "IEBC violated data protection" | ODPC, Privacy International | Growing post-DPA 2019 | MEDIUM-HIGH |
| 13 | "KIEMS kits went missing after the election" | Biometric Update 2025, OAG 2024 | Low but documented | MEDIUM |
| 14 | "The satellite modems were bought but never used" | AfriCOG, Nation | Medium (historical) | LOW-MEDIUM |
| 15 | "International observers couldn't access tech logs" | EU EOM, Carter Center reports | Medium | MEDIUM |

---

### 1.3 Technology Failures That Created Legal Exposure

| Failure mode | Specific source | Supreme Court relevance |
|---|---|---|
| EVID system functional at 7.5% of polling stations on 4 March 2013 | 2013-EVID-specs.md; The Elephant 2017 | Petition 5/2013 — cited as evidence of systemic dysfunction; not dispositive as Court found substantial compliance |
| RTS crashed election night 2013 (17,000 of 33,000 stations transmitted) | README.md (AFRICOMMONS FOIA ref); 2013-RTS-specs.md | Petition 5/2013 — ground raised; Court found the manual recount legitimate |
| Forms 34A not matching 34B aggregate at national tallying centre, 2017 | Raila v IEBC [2017] paras 219-262 | DISPOSITIVE for annulment — majority held the process of tallying "was not transparent, verifiable, or accurate" |
| IEBC declined server/log access to Supreme Court IT experts, 2017 | Raila v IEBC [2017] para 272 | Court drew adverse inference; the refusal became itself an annulment ground |
| Kit failure triggering Section 44A without proper notification to parties, 2017 | Elections Act 2011 s.44A; ELOG report 2017 | Not directly annulment ground but cited as procedure violation in lower court petitions |
| 200 KIEMS kits unaccounted for post-2022 | OAG report 2024; Biometric Update March 2025 | Not litigated yet; creates 2027 re-petition risk |
| Smartmatic Venezuelan nationals with admin server access, 2022 | Standard Media; Petition E005/2022 arguments | Argued in 2022 petition; dismissed on facts — COULD succeed in 2027 if not pre-empted |
| Cherera 4 commissioners dissent — RTS irregularities, 2022 | Petition E005/2022 paras (Cherera dissent) | Not accepted by majority; remains the most potent future ground |

---

## Part II. Project-Management Tweaks

### PM-01: Co-locate the PMO at Anniversary Towers from Day 30

**What:** Embed a named Datacraft PMO cell (two people minimum: PM lead and technical liaison) inside IEBC Anniversary Towers from Contract Day 30 through the post-election debrief.

**Why:** In 2017, the disconnect between OT-Morpho's 94 deployed field staff and IEBC's internal chain of command contributed to the configuration chaos that produced the forms mismatch documented in Raila v IEBC [2017] paras 67-121. A co-located PMO eliminates the communication gap by making decision-latency zero.

**Who benefits:** IEBC (faster escalation), Datacraft (no blame for miscommunication), party agents (decisions accountable to a visible person).

**Cost / effort:** MEDIUM — office space + two senior staff. Funded from service-line budget.

**Adoption difficulty:** EASY — already sketched in POST_AWARD_PROGRAMME.md §1.

**Recommendation:** Propose in bid narrative as a firm commitment; include in Technical Proposal Section III.

---

### PM-02: Tri-weekly steering with recorded minutes and public summary

**What:** Establish a Joint Steering Committee meeting every Tuesday at 14:00 EAT from contract award through election debrief; IEBC chair (or designated commissioner), Datacraft Programme Director, Mechrevo representative, and one ELOG/civil society observer attend. Minutes published in 72 hours on the IEBC transparency portal.

**Why:** AfriCOG (2018) documented that the IEBC Board itself was split on procurement decisions that were never recorded transparently (the March 31, 2017 plenary minutes dispute). Published minutes prevent the "we didn't know" defence from either side and give party agents advance notice of system changes.

**Who benefits:** IEBC (audit trail), party agents (early warning), Datacraft (scope-creep evidence), evaluator (governance signal).

**Cost / effort:** LOW — calendar commitment; publishing is a one-page summary.

**Adoption difficulty:** EASY.

**Recommendation:** Propose in bid narrative; include as a schedule to the Post-Award Programme.

---

### PM-03: Configuration Control Board with HSM-signed change log

**What:** Every change to firmware, software, electoral maps, device images, PKI certificates, or database schema must be approved by the CCB (Datacraft Technical Manager + IEBC ICT Manager + Security Officer + Assurance Board observer) and recorded in an append-only, hash-chained log signed by the HSM.

**Why:** The 2017 server-access dispute centred on the question of who had made what changes and when. IEBC "declined" to give the Supreme Court's IT expert access (Raila v IEBC [2017] para 272). A HSM-signed change log that IEBC itself controls makes that refusal impossible in 2027 because the log is IEBC property, not the vendor's.

**Who benefits:** IEBC (retains sovereign control of the record), judiciary (forensically reliable log), Datacraft (indemnity against phantom accusations).

**Cost / effort:** MEDIUM — HSM procurement plus log infrastructure (part of THREAT_MODEL.md §4).

**Adoption difficulty:** MEDIUM — requires IEBC ICT buy-in on CCB membership.

**Recommendation:** Propose in Technical Proposal; reference THREAT_MODEL.md architecture.

---

### PM-04: Escalation ladder tied to KIEMS kit failure rate thresholds

**What:** Pre-agree with IEBC three formal escalation triggers: (a) >2% kits offline before 7:00 AM on election day triggers Election Command Centre Level 2; (b) >5% triggers automatic notification to party agents and media briefing; (c) >10% triggers Complementary Mechanism declaration and Commissioner involvement.

**Why:** Section 44A of the Elections Act 2011 requires the Commission to declare a failure of technology before a manual fallback is used. In 2017, declarations were patchy and inconsistently notified to party agents (ELOG 2017). The specific thresholds convert a judgment call into a pre-agreed protocol that all parties can verify.

**Who benefits:** Voters (continuity of service), party agents (early warning), IEBC (reduced litigation exposure).

**Cost / effort:** LOW — a protocol document, pre-communicated.

**Adoption difficulty:** MEDIUM — commissioners must approve the thresholds in advance.

**Recommendation:** Propose as PMO improvement; seek written IEBC sign-off before election day.

---

### PM-05: Communication cadence tightening into election window

**What:** Switch from weekly steering meetings to daily standups 90 days out, to twice-daily video calls 30 days out, to an hourly operations call inside the 72-hour polling window; introduce a shared status dashboard visible to IEBC, Datacraft, Mechrevo, and observer delegations simultaneously.

**Why:** OT-Morpho had only 94 staff deployed for the August 2017 election (growing to 292 for the re-run) across 45,000 kits — a 1:479 staff-to-kit ratio (AfriCOG 2018). Information latency during field failure was measured in hours, not minutes. The 2022 Smartmatic deployment fared better on election day but the commissioner-to-vendor communication remained opaque.

**Who benefits:** All parties — every minute of diagnostic delay in election-day kit failures costs voter access and creates petition fuel.

**Cost / effort:** LOW (cadence) to MEDIUM (shared dashboard infrastructure).

**Adoption difficulty:** EASY.

**Recommendation:** Propose in bid narrative as part of Election Command Centre design.

---

### PM-06: Stakeholder mapping with named relationship owners

**What:** Produce and maintain a live stakeholder register covering: all 47 party-agent leads (by party and county), 10 senior civil-society technology analysts, the EU EOM tech observer, ELOG embedded auditors, the Communications Authority CIRT-CC lead, ODPC Director, PPRA Director, DCI Director, and the Supreme Court IT Registrar. Each stakeholder gets a named Datacraft relationship owner with a direct mobile number.

**Why:** The 2022 Smartmatic dispute was partly a failure of stakeholder communication — the "Venezuelan nationals" story grew for weeks before IEBC responded. The Cherera 4 dissent was partly driven by commissioners who felt they did not have direct access to live system telemetry (Petition E005/2022). Named relationship owners pre-empt the vacuum.

**Who benefits:** IEBC (political management), Datacraft (narrative control), evaluator (demonstrates maturity).

**Cost / effort:** LOW — document plus calendar time.

**Adoption difficulty:** EASY.

**Recommendation:** Propose in Technical Proposal Section II (Stakeholder Management).

---

### PM-07: Train-the-trainer cascade with standardised curriculum

**What:** Deliver training through a three-tier cascade: Datacraft trains 47 county trainers; county trainers train 290 constituency supervisors; supervisors train 55,393 presiding officers. Each tier uses the same assessed curriculum with a pass/fail threshold, generating a certificate per presiding officer that becomes part of the post-election audit record.

**Why:** In 2013, some EVID hardware arrived three days before the election (2013-EVID-specs.md), leaving no time for training. Inadequately trained presiding officers are the single largest source of Section 44A declarations and Form 34A errors, both of which are petition grounds.

**Who benefits:** IEBC (fewer kit-failure declarations), voters (faster polling), Datacraft (fewer field support calls), court (trained presiding officer evidence).

**Cost / effort:** MEDIUM — curriculum development and delivery.

**Adoption difficulty:** MEDIUM — requires IEBC training-programme ownership.

**Recommendation:** Propose in Technical Proposal; include in training sub-plan.

---

### PM-08: Independent Assurance Board (the IAB lever)

**What:** Charter an Independent Assurance Board from Contract Day 1 with membership drawn from IEBC ICT, an independent electoral-technology certifier, an accredited security auditor, an ODPC representative, two Kenyan civil-society technologists, and one IFES/IDEA nominee. The IAB has read-only access to the sandbox and production audit logs, and publishes a quarterly public report.

**Why:** The "declined access" finding in Raila v IEBC [2017] para 272 is the single most dangerous precedent for any KIEMS vendor. The IAB eliminates the scenario in which IEBC or Datacraft is accused of hiding the logs, because the logs are continuously visible to an independent body that publishes its findings.

**Who benefits:** IEBC (neutralises server-access narrative), Datacraft (blame shield), party agents (independent technical voice), judiciary (IAB reports = admissible expert evidence).

**Cost / effort:** MEDIUM — board secretariat and log-access infrastructure.

**Adoption difficulty:** HARD — requires IEBC commitment to real independence, not a rubber-stamp.

**Recommendation:** Propose as pro-bono Datacraft contribution (see Part III); embed in bid narrative as a headline differentiator.

---

### PM-09: Documentation regime — every config immutably logged

**What:** Every configuration change, deployment, key ceremony, ABIS deduplication run, voter-roll snapshot, and certificate issuance gets an immutable log entry: timestamp, operator ID, before/after hash, CCB approval reference, and HSM signature. The log is append-only (no delete, no edit), replicated to an IEBC-controlled off-site copy.

**Why:** The OAG (2024) found IEBC "lacked a robust mechanism to effectively track and manage deployment" of KIEMS kits (Biometric Update March 2025). In a petition, an incomplete audit trail is treated as evidence of tampering. A complete immutable trail is the best available rebuttal.

**Who benefits:** Auditor-General (clean audit), IEBC (petition defence), Datacraft (contractual compliance evidence).

**Cost / effort:** LOW to MEDIUM — mostly software engineering.

**Adoption difficulty:** EASY — fully within Datacraft's technical control.

**Recommendation:** Propose in Technical Proposal; cross-reference to THREAT_MODEL.md §8.

---

### PM-10: Dry-run election 60 days before election day

**What:** Run a full national dry-run election using live KIEMS kits at a representative sample of 500 polling stations (1% sample across all 47 counties) 60 days before election day. Invite ELOG, EU EOM, and all registered political party ICT leads as observers. Publish the dry-run report.

**Why:** The 2013 RTS crashed because the system was never tested at full national load before election day (AFRICOMMONS FOIA). The 2017 KIEMS was configured in 60 days post-award with essentially no full-scale rehearsal. At a 60-day gap there is still time to fix material bugs found.

**Who benefits:** IEBC (defects found before votes count), party agents (confidence), Datacraft (early detection of field failure patterns), evaluator (demonstrates delivery confidence).

**Cost / effort:** HIGH — logistics, kit deployment, recovery.

**Adoption difficulty:** HARD — requires IEBC organisational commitment.

**Recommendation:** Propose as joint IEBC-Datacraft commitment in post-award programme.

---

### PM-11: Election Command Centre with 24x7 cover and media-adjacent briefing room

**What:** Activate the Election Command Centre 30 days before election day at IEBC Anniversary Towers; staff it 24x7 through result announcement; include a media briefing room adjacent with sanitised real-time dashboard feeds.

**Why:** In 2017, the absence of a visible, real-time public operations centre meant every rumour filled the information vacuum. The Standard Media story about "Chiloba's servers" (The Elephant, 2017) gained traction precisely because there was no countervailing live feed of system status. A media-accessible ECC does not eliminate petitions but dramatically shrinks the narrative space available to challengers.

**Who benefits:** IEBC (narrative control), media (honest feed), voters (real-time transparency), Datacraft (early escalation channel).

**Cost / effort:** MEDIUM — physical space, staffing, dashboard build.

**Adoption difficulty:** MEDIUM.

**Recommendation:** Propose in Technical Proposal; include in Election Command Centre section of POST_AWARD_PROGRAMME.md.

---

### PM-12: KIEMS asset tracking from warehouse to polling station and back

**What:** Implement the RFID asset tracking required by the 2027 tender (2027-preparatory-specs.md §2.4) with an asset management platform accessible to IEBC, Datacraft, and (read-only) to the Auditor-General. Every kit transition — warehouse to county, county to constituency, constituency to polling station, and return — is logged with a timestamp, officer ID, and GPS coordinates.

**Why:** The OAG (2024) found 200 kits missing and 3,433 faulty, with IEBC unable to trace their movements (Biometric Update March 2025). In 2022 the Smartmatic debt dispute partly hinged on what equipment had been delivered and confirmed. An end-to-end asset trail prevents disputes in both directions.

**Who benefits:** IEBC (asset stewardship), Auditor-General (clean audit), Datacraft (delivery proof), Smartmatic-debt-class disputes: never again.

**Cost / effort:** MEDIUM — RFID infrastructure and platform.

**Adoption difficulty:** EASY — requirement is already in the tender.

**Recommendation:** Deliver as core contract scope; propose the Auditor-General read-only access as a pro-bono add-on.

---

## Part III. Pro-Bono Services That Make a Monstrous Difference

### PB-01: Independent Assurance Board secretariat

**Service:** Datacraft funds and staffs the IAB secretariat for the duration of the contract.

**Datacraft cost:** KES 3-6M per year (two part-time staff, meeting costs, quarterly report production).

**Value to IEBC:** Eliminates the server-access legal risk; the IAB is the standing technical witness in any Supreme Court petition.

**Legal / reputational shield:** Directly neutralises Raila v IEBC [2017] para 272 (server access refusal). Also addresses the "foreign nationals" ground from 2022.

**How to package:** Separate MoU between Datacraft and IEBC signed on Contract Day 1; listed in Technical Proposal as headline commitment at KES 0.

---

### PB-02: Public Verifier mobile app (open-source, IEBC-branded)

**Service:** Datacraft builds, tests, and publishes to Google Play and Apple App Store an open-source "Verify My Vote" app. Any Kenyan enters their polling station code and sees: (a) the uploaded Form 34A image, (b) a machine-readable tallying summary, (c) the cryptographic hash of the form, verifiable against the IEBC public log.

**Datacraft cost:** KES 4-8M (development, security audit, App Store fees, maintenance).

**Value to IEBC:** Converts the "where is Form 34A?" narrative (#WapiForm34A) into an active public verification tool. Citizens become IEBC's audit partners, not its adversaries.

**Legal / reputational shield:** Neutralises the Forms 34A/B/C transparency ground that was dispositive in the 2017 annulment (paras 219-262). Any litigant claiming forms were altered must contend with millions of citizen-verified hashes.

**How to package:** Separate schedule in Technical Proposal; "Public Verification System" listed in Bill of Materials at KES 0.

---

### PB-03: Free WhatsApp and SMS results-check gateway

**Service:** Deploy a WhatsApp Business API gateway and SMS short-code (request IEBC to obtain a non-premium short-code from CA) so any Kenyan can text their polling station code and receive the official Form 34A tally by return.

**Datacraft cost:** KES 2-4M (gateway setup, API subscription, SMS aggregator rates; IEBC absorbs ongoing SMS cost post-delivery).

**Value to IEBC:** Scales verification to the 70%+ of Kenyans on feature phones without smartphones. Reduces post-election calls to IEBC's overloaded helpline.

**Legal / reputational shield:** Addresses the "ordinary Kenyans can't access the results" grievance cluster (chatter topology #3); removes the basis for the "portal was down" narrative.

**How to package:** Footnote in Technical Proposal + schedule; propose as billable by-election extension post-award.

---

### PB-04: Code escrow with rolling monthly deposits

**Service:** Datacraft establishes a code escrow account with NCC Group (or equivalent tier-1 escrow agent) on Contract Day 1, with monthly deposits of the entire KIEMS software stack (source, build scripts, test harness, dependency manifest, SBOM). IEBC is the named beneficiary; escrow releases automatically on insolvency or material breach.

**Datacraft cost:** USD 15,000-25,000/year (NCC Group or Iron Mountain escrow pricing).

**Value to IEBC:** Eliminates the "vendor disappears and IEBC can't operate the system" risk. In 2022, Smartmatic sued IEBC for KES 570M in unpaid obligations (2022-KIEMS-specs.md §7); if Smartmatic had withheld software updates, IEBC would have been helpless. Escrow prevents this in 2027.

**Legal / reputational shield:** Demonstrates IEBC sovereignty over the system to PPRA and OAG; protects IEBC from procurement-dependency attack in a future petition.

**How to package:** Named escrow agent + MoU as an exhibit to the Technical Proposal; escrow fees at KES 0 to IEBC.

---

### PB-05: Kenyan-university partnership for observer and agent training

**Service:** Datacraft funds a 2-day "Election Technology Literacy" short course at JKUAT, USIU-Africa, Strathmore University, and University of Nairobi, open to all registered political party ICT agents, ELOG observers, and civil-society technologists. The course covers: how KIEMS works, what to look for, what failure modes exist, how to read the transparency dashboard, and how to use the Public Verifier app.

**Datacraft cost:** KES 3-6M (faculty stipends, venues, materials, ~500 participants per institution).

**Value to IEBC:** Creates a technically literate observer class that can distinguish genuine system failures from politically motivated claims. If party agents are trained on the system, their endorsement of the results carries legal weight; their objections are also specific and answerable, not vague grievances.

**Legal / reputational shield:** Addresses "party agent access denial" ground (topology #9); trained agents are less likely to succeed with a blanket "we were excluded" petition argument.

**How to package:** MoU with each university, signed pre-election; referenced in Technical Proposal as civil-society partnership.

---

### PB-06: Free bug-bounty programme, 12 months out

**Service:** Datacraft announces a public bug-bounty programme 12 months before election day covering the KIEMS client applications, the RTS portal, the Public Verifier app, and the ABIS API. Scope published on HackerOne or Bugcrowd; Kenya-domiciled researchers prioritised; payouts on a defined scale (KES 10K to KES 500K per validated finding).

**Datacraft cost:** KES 2-5M (platform fees + expected payout pool).

**Value to IEBC:** Proactively harnesses the security researcher community that would otherwise probe the system adversarially. Every fixed vulnerability reported pre-election is a ground that cannot succeed in a post-election petition.

**Legal / reputational shield:** Directly addresses THREAT_MODEL.md attack surfaces 3, 6, and 7; demonstrates openness that resets the "IEBC hidden servers" narrative.

**How to package:** Press release issued jointly by IEBC and Datacraft 12 months before election day; listed in Technical Proposal as security programme.

---

### PB-07: 24x7 Security Operations Centre with observer read-only access

**Service:** Datacraft operates a SOC throughout the election window (30-day activation to result-announcement day + 7) and provides read-only, sanitised dashboard access to: KEOG, ELOG, EU EOM, Carter Center, and IFES. The sanitised view shows: kit-online percentage, RTS transmission progress, public-portal load, and alert count by severity. No PII or biometric data is visible.

**Datacraft cost:** KES 5-10M for the election window (SOC staffing, tooling).

**Value to IEBC:** International observer endorsement of system integrity is the single most powerful prophylactic against a Supreme Court petition. EU EOM and Carter Center reports citing real-time SOC access are substantially more credible than reports citing IEBC assurances.

**Legal / reputational shield:** Neutralises the "server access denied" ground; provides the observer community with the independent technical evidence they need to write favourable reports.

**How to package:** LOI with KEOG, ELOG, EU EOM, and Carter Center; observer-access protocol as an exhibit to the Technical Proposal.

---

### PB-08: Legal-defence support briefing pack for IEBC

**Service:** Datacraft prepares (through retained counsel, at Datacraft's cost) a pre-election "petition defence brief" — a structured document covering: the cryptographic evidence available from the KIEMS log, how to request and present HSM-signed audit records in court, the chain-of-custody records that refute each of the known petition grounds. The document is handed to IEBC's legal team 14 days before election day and updated 48 hours after polls close.

**Datacraft cost:** KES 1-3M (counsel briefing, document production).

**Value to IEBC:** In both 2017 and 2022, IEBC's legal response to technical grounds in petitions was reactive and slow. A pre-packaged evidence brief, organised by petition-ground taxonomy, reduces the 7-day petition filing window from a crisis to a manageable response.

**Legal / reputational shield:** Directly shortens IEBC's legal response time; demonstrates Datacraft's long-term commitment beyond contract delivery.

**How to package:** Confidential schedule to the Technical Proposal; not published.

---

## Part IV. Election-Proofing Architecture

### A. Reputational Proofing

**Public Transparency Toolkit.** From the moment the first voter is registered, IEBC (served by Datacraft) publishes a live public dashboard at a stable URL showing: voter registration progress by county, ABIS deduplication status (count of duplicate flags resolved), kit commissioning progress, training completion by county, and pre-election dry-run results. On election day the dashboard adds: turnout by county (updated hourly), kit-online percentage, RTS transmission progress, and Form 34A publication status.

**Real-time Integrity Dashboard.** Every polling station's kit transmits a heartbeat every 15 minutes. The dashboard shows — to the public — the percentage of stations transmitting (green/amber/red). This converts an election-night blackout (the 2013 RTS crash hit with no advance warning) into a predicted, publicly visible curve.

**Result-form Quality Gate.** No Form 34A is published to the public portal until an automated quality gate passes it through: (a) image resolution above minimum threshold, (b) all required signature fields detected by OCR/ML, (c) QR code verified against the expected polling station binding, (d) hash computed and logged. Failed forms are flagged for manual review by the Presiding Officer; they are not silently suppressed.

**Rapid-Response Comms Cell.** Pre-draft holding statements for the 12 most likely petition-flavoured events: kit failure above threshold, RTS below target, form image quality below threshold, server access request from any external party, ABIS deduplication query, power outage at the national tallying centre, social-media claim of tampering, DDoS on the public portal, political party withdrawal threat, commissioner disagreement made public, Chris Msando class of security incident, and observer access request denied. Each statement is pre-approved by IEBC Legal and IEBC Chair; it can be issued within 15 minutes of a triggering event.

**Cross-party observer credentialing.** Every registered political party is offered: two accredited ICT agent positions per polling station, observer access to the sanitised SOC dashboard, daily briefings at the ECC during the election window, and a copy of the end-of-day transmission report. Offered in writing 60 days before election day; documented whether accepted or declined.

---

### B. Legal Proofing

**Full evidentiary trail per KIEMS action.** Every action in the KIEMS system — BVI query, fingerprint match result, Form 34A capture, form upload, tally entry — is recorded in a Merkle-chained event log: (who: officer NSSF/staff ID), (where: polling station code + GPS fix), (what time: NTP-synchronised UTC timestamp from a Kenyan NTP server), (what device: TPM-attested device fingerprint), (what state: before-hash and after-hash of the relevant data object). The root hash of the day's Merkle tree is published to the public transparency log every 30 minutes.

**Retention policy.** All KIEMS logs, Form 34A images, BVI records, audit logs, and HSM key records are retained for not less than 7 years — covering the 7-day petition filing window, the 14-day court hearing window, all appeal levels (Court of Appeal and Supreme Court), and the 6-year general limitation period under the Limitation of Actions Act. Retention is contractually specified and includes IEBC's right to demand a physical backup copy.

**Server-access protocol pre-agreed with the Supreme Court IT Registrar.** Before election day, Datacraft and IEBC jointly brief the Supreme Court IT Registrar (by letter) on the architecture of the KIEMS log, the cryptographic proofs available, and the procedure for log access in the event of a petition. A named Datacraft technical witness is registered with the Registrar's office, available within 24 hours of a petition filing. The 2017 "declined access" finding (Raila v IEBC [2017] para 272) arose from absence of a pre-agreed access protocol; the 2027 approach makes access the default, not the exception.

**Chain-of-custody for physical ballots via RFID.** Each ballot paper consignment is RFID-tagged at the printer (Al Ghurair or equivalent); tags are read at: print-to-pack, pack-to-container, container-to-Nairobi port, port-to-IEBC warehouse, warehouse-to-constituency, constituency-to-polling station. The RFID log is maintained separately from the KIEMS log and cross-referenced at each transfer with a dual-witness signature. Discrepancies generate an immediate alert.

**Back-to-back indemnity flow-down.** The prime contract between Datacraft/Mechrevo and IEBC specifies: Mechrevo warrants the hardware against defect for 36 months; if a hardware defect contributes to a KIEMS failure that is cited in a petition, Mechrevo bears the technical-evidence cost of rebutting the hardware ground. Datacraft likewise warrants the software. Neither warranty gap (hardware vs. software vs. integration) can be exploited to leave IEBC without a technical indemnitor.

---

### C. Procedural Proofing

**Section 44A protocol pre-rehearsed.** The Elections Act 2011 s.44A authorises the Commission to declare a failure of technology and revert to a manual Complementary Mechanism. That declaration must be: (a) made by the Commission (not by a Presiding Officer alone), (b) communicated to party agents before the fallback is used, (c) documented. In 2027, the protocol is: Presiding Officer reports failure to County Support desk; County Support desk verifies and escalates to the ECC; if the ECC confirms failure and the kit cannot be recovered within 30 minutes, the ECC requests a Section 44A declaration from the duty commissioner; the duty commissioner issues the declaration via the ECC; the presiding officer notifies party agents at the polling station before switching to the manual register. The entire chain is pre-rehearsed in the dry-run election (PM-10).

**Complementary Mechanism pre-signed and witnessed.** At every polling station opening, the Presiding Officer, a Datacraft field technician, and at least two party agents sign a "Complementary Mechanism Readiness Form" confirming that the manual register, stamped ballot papers, and Form 34A hard copies are present and sealed. The form is photographed and uploaded via KIEMS. In the event of a Section 44A declaration, the readiness form is the legal evidence that the fallback materials were present before any failure.

**Form 34A/B/C signing choreography.** The presiding officer triggers Form 34A generation on the KIEMS device. The form is displayed on-screen at the polling station for 10 minutes while party agents and candidates' agents review it. Each party agent present uses the KIEMS app to record their acceptance (or objection). The form is then printed (if a printer is present) or photographed with a time-stamped photograph. The photograph is uploaded via KIEMS with a hash. The physical form is signed by the Presiding Officer and at least two party agents. At least two copies are made: one retained at the polling station, one handed to the Returning Officer.

**Result-portal publication SLA.** The public result portal publishes each Form 34A image within 60 minutes of upload from the polling station. The portal SLA is 99.9% uptime during the 72-hour results window. Form 34A publication is not contingent on the aggregate tally; each form publishes independently as it arrives.

**Voter education campaign.** Datacraft co-funds (with IEBC's communication budget) a targeted digital voter-education campaign, 60 days before election day, teaching voters: what happens at their polling station, what a successful KIEMS verification looks like, what a Section 44A declaration means, and how to use the Public Verifier app. The campaign is carried on Citizen TV, Nation FM, Kameme FM, and major radio stations in all the main languages.

---

### D. Technical Proofing

**End-to-end cryptographic architecture.** Every KIEMS device carries: (a) a TPM 2.0 secure element with a device-unique private key, issued a certificate by the IEBC root CA at factory acceptance; (b) an HSM-signed boot attestation quote at every power-on; (c) all network traffic between the kit and the national tallying centre encrypted with TLS 1.3 with certificate pinning (no certificate authority surprise attacks); (d) every KIEMS event signed at source with the device's TPM key, making the event log tamper-evident at the polling station level, not just at the centre.

**Zero-trust internal architecture.** No human has unrestricted shell access to any KIEMS production server. All server operations require: dual-control authentication (two named officers, biometrically identified), a CCB-approved change ticket number, and HSM signature on the session key. The session is recorded in full in the audit log. No admin bypasses are technically possible; all bypass attempts are logged and alert the Security Officer and the IAB.

**Publicly published cryptographic verification tool.** Datacraft publishes (open-source, MIT licence) a command-line tool and web widget that takes a Form 34A image and a polling station code and independently verifies: (a) that the image hash matches the hash in the IEBC public transparency log; (b) that the log entry was signed by the IEBC root CA; (c) that the log entry was published before the national result announcement. Any citizen with a laptop can run this verification.

**Hash of the KIEMS software published to the Government Printer.** 30 days before election day, the exact software image loaded on every KIEMS kit (SHA-256 hash of the signed firmware bundle, plus hash of the application APK) is published in the Kenya Gazette, signed by the IEBC Chair and the Datacraft Programme Director. Any post-election claim that a different software version ran on election day is refuted by a Gazette notice.

**Data-diode design for upload.** Polling-station KIEMS kits can transmit Form 34A images to the national tallying centre (upload), but the national tallying centre cannot transmit instructions back to a kit during polling hours (download disabled). The data-diode is enforced at the network level, not by application policy. This eliminates the "central command could have updated the software mid-election" attack surface (THREAT_MODEL.md §5).

**Kit tamper-evidence.** Every KIEMS kit carries: (a) a physical tamper-evident seal over the battery and SIM compartment, numbered and logged in the asset management system; (b) a TPM attestation that verifies the seal has not been broken electronically (epoxy-potted secure element kill switch). A kit whose seal is broken or whose TPM attestation fails cannot complete a BVI query; it alerts the ECC automatically.

**Result-portal read-only replicas.** The public result portal is served from at least three geographically separate read-only replicas (primary: IEBC data centre; secondary: AfricaOnline Mombasa; tertiary: Liquid Intelligent Technologies Kampala or equivalent). A DDoS attack that takes down the primary portal does not take down the results; the secondary and tertiary replicas continue serving. This directly addresses the "portal was down" narrative from 2017.

**Cryptographic sortition for post-election audit.** After result announcement, a verifiable random function (VRF) seeded from the final Merkle root of the KIEMS event log selects 5% of polling stations for a mandatory manual recount. The selection is mathematically derived from the election data itself — no human chooses which stations are audited. Any party can independently compute the selected stations from the published Merkle root. This is the highest-confidence post-election integrity mechanism available.

---

## Part V. The Bid Narrative — Packaging for Evaluators

### What goes where

| Item | Document | Format |
|---|---|---|
| PM-01 to PM-12, IV-A, IV-C | Technical Proposal, Section III (Implementation Methodology) | Narrative + Gantt |
| IV-B (Legal proofing: Merkle log, retention, server protocol) | Technical Proposal, Section IV (Security Architecture) | Architecture diagram + narrative |
| IV-D (Cryptographic architecture) | Technical Proposal, Section IV | THREAT_MODEL.md excerpt adapted |
| PB-01, PB-02, PB-03, PB-07, PB-08 | Technical Proposal, Annex A (Pro-Bono Commitments Schedule) | Table with KES 0 line items |
| PB-04 (Code escrow) | Technical Proposal, Annex B + Commercial Narrative | Named escrow agent + MoU |
| PB-05 (University partnership) | Technical Proposal, Annex A + separate MoU | LOI from each institution |
| PB-06 (Bug-bounty) | Technical Proposal, Annex A | Programme brief |
| IV-D (Government Gazette hash) | Undertaking Letter signed by Datacraft Programme Director | Firm commitment letter |
| PM-03, PM-09 (CCB, immutable log) | Technical Proposal, Section IV | Architecture |
| PM-08 (IAB) | Technical Proposal + separate Undertaking Letter | Named board composition |

**MoUs and LOIs to execute pre-award:**
- KEOG — observer access to SOC dashboard
- ELOG — embedded auditor technical access
- JKUAT / USIU / Strathmore / UoN — election-literacy training programme
- Communications Authority CIRT-CC — cyber-drill programme
- NCC Group (or Iron Mountain) — code escrow

**Items at KES 0 in the BoM:** PB-01 through PB-08. List each in a dedicated "Pro-Bono Services" BoM section with an explicit KES 0.00 per-unit rate. This signals seriousness to the evaluator without inflating the contract price.

**The pitch line:** "Datacraft's bid is not the cheapest. Datacraft's bid is the one that cannot lose you the election."

That sentence belongs in the Executive Summary of the Technical Proposal and in the opening of the Commercial Narrative. It is not a boast; it is a legal and reputational claim supported by the cryptographic architecture, the IAB, the pre-agreed server-access protocol, the public verifier, and the pro-bono legal brief.

---

## Part VI. Risk Register

| # | Risk | Owner | Mitigation | Contingency | Trigger | RAG |
|---|---|---|---|---|---|---|
| R1 | IEBC rejects OEM Manufacturer's Authorisation route; bid disqualified pre-evaluation | Datacraft Legal + Mechrevo | Pre-bid clarification A1 (already drafted in PRE_BID_CLARIFICATIONS.md); JV fallback partner identified | Activate JV fallback within 48 hours of IEBC rejection | IEBC written response to clarification | RED |
| R2 | KES 30M tender security not issued before 31 Aug 2026 | Datacraft Finance | Bank KYC started T-12; daily follow-up from T-8 | Approach second bank from T-6 | Bank confirms timeline slip | RED |
| R3 | Iris biometric sensor supply chain delays (new requirement, limited OEM market) | Mechrevo Hardware | Lock iris sensor supplier by T-11; place order before bid deadline even if award not yet made | Dual-source (IriShield + competitor); stock 120% of required quantity | Supplier confirms delivery risk | AMBER |
| R4 | Post-election petition on Form 34A image quality | Datacraft Technical | Quality gate (Part IV-A); cryptographic hash published pre-announcement | IAB provides independent expert witness; pre-packaged legal brief (PB-08) | Any Form 34A quality dispute raised | GREEN |
| R5 | Section 44A kit-failure rate exceeds 5% on election day | Datacraft / IEBC Operations | Dry-run (PM-10); training cascade (PM-07); Section 44A protocol (Part IV-C) | Complementary Mechanism pre-positioned at every station; duty commissioner on standby | ECC Level 2 threshold hit | GREEN |
| R6 | DDoS attack on public result portal | Datacraft Security | Geographically distributed read-only replicas; Cloudflare or equivalent DDoS mitigation | Activate secondary and tertiary replicas; media briefed in advance of expected DDoS | SOC DDoS alert | GREEN |
| R7 | IAB member conflict of interest undermines independence | IEBC (IAB Chair) | Selection protocol with conflict-of-interest declarations; civil-society veto on IEBC nominees | Replace member within 30 days; IAB continues with remaining members | Conflict disclosed | AMBER |
| R8 | PPRA challenge to Datacraft/Mechrevo consortium structure | Datacraft Legal | PPRA debarment search on all entities; clean compliance from Day 1; PPARB appeal history reviewed | Respond within PPARB 14-day window; JV restructure if required | PPARB notice received | AMBER |
| R9 | Data Protection Act 2019 non-compliance (biometric data processing) | Datacraft DPO | ODPC registration + DPIA filed pre-award; DPO on PMO from Day 1 | Engage ODPC proactively; escalate through IAB line | ODPC inquiry or civil society complaint | GREEN |
| R10 | Chris Msando-class security incident against Datacraft personnel | Datacraft Security Officer + DCI | Personal security protocols for all senior staff during 90-day election window; coordination with DCI | Pre-agreed emergency protocol; backup named officers for all critical roles | Credible threat intelligence | AMBER |
| R11 | Mechrevo Chinese originals do not arrive in Nairobi before content freeze | Mechrevo Logistics | Two courier streams (DHL + FedEx); dispatch from T-10 | Digital apostille route if Kenya accepts; legal opinion on digital originals | DHL/FedEx tracking shows delay | RED |
| R12 | Smartmatic outstanding debt (KES 570M) creates legal encumbrance on KIEMS 2022 equipment affecting 2027 transition | IEBC Legal | Confirm debt resolution status before contract award; escrow release mechanism agreed | Procure independent legal opinion; indemnify IEBC against Smartmatic third-party claims in transition | Smartmatic files injunction | AMBER |
| R13 | Public chatter about Venezuelan nationals (Smartmatic 2022) transferred to Mechrevo (Chinese nationals) | Datacraft Communications | Nationality-transparent communications plan; Kenyan faces on the programme (47 county coordinators all Kenyan); IAB report addresses | Rapid-response comms cell (Part IV-A); pre-drafted holding statement | Hostile media story filed | GREEN |
| R14 | ABIS deduplication backlog not cleared before voter roll is locked | Datacraft ABIS team | Weekly ABIS progress reports published; IAB oversight; clear SLA on deduplication throughput | Deploy additional ABIS compute; extend voter registration window request to IEBC | Backlog > 100,000 records 30 days before lock | AMBER |
| R15 | Residual: Supreme Court creates new petition ground in 2027 not anticipated here | Datacraft Legal + IAB | Quarterly legal review of electoral jurisprudence; IAB includes constitutional law observer; petition brief updated every 90 days | Activate legal-defence brief (PB-08) within 24 hours of petition filing | Petition filed | GREEN |

---

## Part VII. Sources and Citations

### Source Files (this research base)

- `docs/research/kenya-historical/README.md` — master pricing and procurement history
- `docs/research/kenya-historical/2013-BVR-specs.md` — BVR procurement, Safran Morpho / CCC
- `docs/research/kenya-historical/2013-EVID-specs.md` — EVID procurement, Face Technologies; 7.5% election-day functionality
- `docs/research/kenya-historical/2013-RTS-specs.md` — RTS crash, USAID/IFES
- `docs/research/kenya-historical/2017-KIEMS-specs.md` — OT-Morpho KIEMS; one-day tender; 45,000 MorphoTablet 2
- `docs/research/kenya-historical/2017-Oct-rerun-specs.md` — October re-run; KES 2.4-2.5bn
- `docs/research/kenya-historical/2022-KIEMS-specs.md` — Smartmatic VIU-100; KES 3.2bn; 200 missing kits
- `docs/research/kenya-historical/2027-preparatory-specs.md` — Tender IEBC/OIT/01/2026-2027; KES 9.3bn; iris; RFID; 59,352 kits
- `docs/research/PRICING-GUIDANCE.md` — pricing benchmarks; per-kit ranges; KES 4.0-4.3bn Year 1 recommendation
- `POST_AWARD_PROGRAMME.md` — PMO structure; CCB; IAB; Election Command Centre; certification regime
- `THREAT_MODEL.md` — 12 attack surfaces; cryptographic controls; zero-trust; transparency log
- `HANDOFF.md` — bid status; delivery timeline; two fatal failure modes
- `BID_GOVERNANCE.md` — steering committee; no-bid gates; risk register

### Court Cases

- Raila Odinga & 5 Others v IEBC & 3 Others [2013] eKLR, Petition 5 of 2013 — https://new.kenyalaw.org/akn/ke/judgment/kesc/2013/3/eng@2013-03-30
- Raila Amolo Odinga & Another v IEBC & 4 Others [2017] eKLR, Petition 1 of 2017 (annulment judgment) — https://new.kenyalaw.org/akn/ke/judgment/kesc/2017/1/eng@2017-09-01 — paras 67-121 (forms), 219-272 (transmission, server access)
- Raila Amolo Odinga v IEBC [2017] — October fresh election detailed reasoning — https://new.kenyalaw.org/
- Raila Odinga & 16 Others v William Ruto & 10 Others [2022] eKLR, Petition E005 of 2022 (dismissed) — https://new.kenyalaw.org/akn/ke/judgment/kesc/2022/1/eng@2022-09-05

### Legislation

- Constitution of Kenya 2010, Arts 81, 86, 88, 138, 140 — https://www.kenyalaw.org/lex/actview.xql?actid=Const2010
- Elections Act 2011 (No 24 of 2011), ss. 39, 44A (KIEMS failure protocol), Form 34A/B/C/35A/B/C/36A/B/C/37A/B — https://www.kenyalaw.org/lex/actview.xql?actid=No.%2024%20of%202011
- IEBC Act 2011 and 2016 amendments — https://www.kenyalaw.org/lex/actview.xql?actid=No.%209%20of%202011
- Election Offences Act 2016 — https://www.kenyalaw.org/lex/actview.xql?actid=No.%2037%20of%202016
- Election (Technology) Regulations 2017 — Kenya Gazette Supplement No. 19, 2017
- Public Procurement and Asset Disposal Act 2015 (PPADA) — https://www.kenyalaw.org/lex/actview.xql?actid=No.%2033%20of%202015
- Data Protection Act 2019 — https://www.kenyalaw.org/lex/actview.xql?actid=No.%2024%20of%202019
- Computer Misuse and Cybercrimes Act 2018 — https://www.kenyalaw.org/lex/actview.xql?actid=No.%205%20of%202018
- Access to Information Act 2016 — https://www.kenyalaw.org/lex/actview.xql?actid=No.%2031%20of%202016

### Secondary Sources

- AfriCOG, "The Dark Business of Election 2017" — https://africog.org/the-dark-business-of-election-2017/
- The Elephant, "Mystery of the Missing Servers" (2017) — https://www.theelephant.info/analysis/2017/10/25/mystery-of-the-missing-servers-were-the-august-8-elections-predetermined/
- The Elephant, "Smartmatic: The Election Company" (2022) — https://www.theelephant.info/analysis/2022/08/03/smartmatic-the-election-company-and-their-role-in-the-upcoming-elections/
- AFRICOMMONS FOIA analysis (RTS 2013) — https://africommons.com/2020/05/kenya-2013-failed-results-transmission-system-reports-usaid-foia/
- Biometric Update, "Nearly 200 biometric election devices missing" (March 2025) — https://www.biometricupdate.com/202503/nearly-200-biometric-election-devices-missing-in-kenya-audit-finds
- Standard Media, Smartmatic debt suit — https://www.standardmedia.co.ke/national/article/2001547083/smartmatic-sues-iebc-over-sh500m-debt
- People Daily, RFID tracking requirement — https://peopledaily.digital/news/inside-iebcs-plan-to-use-radio-frequency-to-track-kiems-kits-in-2027-polls
- Pulse Kenya, tender open — https://www.pulse.co.ke/story/iebc-opens-tenders-for-2027-election-ballot-papers-system-2026081104363371425
- The Star, IEBC budget KES 74bn — https://www.the-star.co.ke/news/2026-05-27-iebc-gets-record-sh74bn-budget-for-2027-polls
- ELOG, 2017 Kenya General Election report — https://elog.or.ke
- EU Election Observation Mission Kenya 2022, Final Report — https://eueom.eu/kenya-2022/final-report
- Carter Center Kenya 2017 report — https://www.cartercenter.org/resources/pdfs/news/peace_publications/election_reports/kenya-2017-election-report.pdf
- Privacy International, election technology Kenya (2024) — https://privacyinternational.org/explainer/5445/election-technology-kenya
- Kahawatungu, "IEBC fraudulently awarded KIEMS tender to IDEMIA" (2019) — https://kahawatungu.com/iebc-fraudulently-awarded-kiems-tender-idemia/
- Nation audit exposé — https://nation.africa/kenya/news/Audit-report-exposes-rot-at-IEBC/1056-4393710-muqkxfz/index.html

---

*This document was prepared by the Datacraft bid team on 2026-08-20. It is internal strategy only and does not represent IEBC policy or any committed Datacraft commercial position until such commitments are made in the formal bid submission.*

---

## Part VIII — Attack-Surface & Mitigation Register

> **v2 note:** This Part was added 2026-08-20. Parts I-VII are unchanged.

---

### VIII.1 — Threat-Actor Taxonomy

Kenya's election attack surface draws actors across nine broad categories, each with distinct capabilities and motivations.

**Political-legal actors** are the most active: losing-coalition legal teams, their expert witnesses, and opposition research units systematically probe every technical seam looking for a petition ground. They are sophisticated, well-funded, and have Supreme Court precedent (Raila v IEBC [2017] eKLR, annulment) as proof the strategy pays off.

**Party operatives** on both sides seek to tilt the result — not necessarily through system compromise but through narrative: a well-timed leak, a manufactured Form 34A anomaly, or a coached defector.

**Insider IEBC staff** have historically been the highest-consequence actors: Ezra Chiloba's unilateral procurement (2017 PAC proceedings); the Cherera 4 commissioners' public dissent (2022); the unaccounted 200 kits (OAG 2024). Insider threats bypass almost every perimeter control.

**Foreign state and grey-market operators** include Russian-affiliated troll networks, PRC-adjacent perception operations (which cut both ways: either to install a favoured candidate or to delegitimise an unfavoured one), and Israeli grey-market consultancies of the Nikuv / Archimedes / Team Jorge family, documented as active across sub-Saharan elections in the 2022 Haaretz / Guardian investigation.

**Cybercriminals** (ransomware, DDoS-for-hire, credential brokers) are opportunistic but the Kenyan precedent exists: Airtel Kenya suffered ransomware in 2024 (Nation, Feb 2024); the NTSA database was breached in 2023.

**Hacktivists and adversarial journalists** probe systems in the public interest and often surface the most damaging real vulnerabilities; their findings become petition fuel.

**Civil-society and observer groups** (ELOG, AfriCOG, CRECO, EU EOM, Carter Center) are adversarial-neutral: they are not trying to harm but their reports, if negative, carry quasi-judicial weight.

**Disinformation-for-hire operators** (including domestic spin shops) manufacture evidence: fake screenshots, doctored Forms 34A, AI-generated video — and inject it into the 72-hour results window when the information environment is most volatile.

**Religious-cultural and ethnic-community mobilisers** use election technology as a proxy for grievance; their attacks are reputational rather than technical but can prevent polling-station access and generate petition grounds on grounds of voter intimidation.

---

### VIII.2 — Attack / Mitigation Register

**Table (summary — detail sections follow)**

| # | Category | Attack / query / doubt | Threat actor | Historical precedent | Mitigation | Owner | Cost (KES) | Timing | Residual risk |
|---|---|---|---|---|---|---|---|---|---|
| A1 | Constitutional | Art. 81 fair-and-free challenge: biometric verification excludes elderly/injured voters who cannot give fingerprints | Opposition legal teams | Petition 5/2013 paras 67-88; 2017 field complaints | Complementary Mechanism pre-signed at every station; presiding officer training; OCR fallback on voter photo | Datacraft/IEBC Operations | 2M | T-365 to T-1 | LOW |
| A2 | Constitutional | Art. 86 verifiability: public cannot independently verify KIEMS result | Opposition legal teams | Raila v IEBC [2017] para 219-262 | Public Verifier app + Merkle-chain (PB-02); Gazette hash publication | Datacraft Technical | 8M | T-365 to T-30 | LOW |
| A3 | Constitutional | Art. 88 IEBC independence: vendor controls the system, not IEBC | Civil society, political parties | Smartmatic Venezuelan nationals, Standard Media 2022 | Code escrow (PB-04); IAB (PM-08); zero-trust architecture; HSM-signed log controlled by IEBC | Datacraft Legal/Technical | 4M | T-365 to T-1 | LOW-MED |
| A4 | Constitutional | Art. 138 presidential-election specific: failure to transmit results electronically as required | Supreme Court, opposition | Raila v IEBC [2017] annulment on transmission | Data-diode upload; 3-replica portal; Merkle log; Form 34A quality gate | Datacraft Technical | 15M | T-180 to T-1 | LOW |
| A5 | Constitutional | Art. 140 petition procedure: 7-day window; IEBC must produce evidence quickly | IEBC Legal / Datacraft | Every contested election since 2013 | Pre-packaged legal brief (PB-08); named Datacraft witness registered with Supreme Court IT Registrar | Datacraft Legal | 3M | T-30 + 7 | LOW |
| A6 | Statutory | Elections Act s.39 electronic transmission: chain-of-custody, form of transmission, signature requirements | Opposition legal teams | Raila v IEBC [2017] paras 219-262 | TPM-signed upload per kit; Merkle chain; form of transmission documented in Gazette | Datacraft Technical | 5M | T-365 to T-1 | LOW |
| A7 | Statutory | Elections Act s.44A invocation: complementary mechanism declared without notifying party agents | Petitioners | ELOG 2017 report; Raila v IEBC [2017] | Pre-agreed s.44A escalation ladder (PM-04); declaration logged and broadcast to party agents via SMS in real time | Datacraft/IEBC Operations | 1M | T-60 to T-1 | LOW |
| A8 | Statutory | Election (Technology) Regulations 2017: device certification not shown; source code not disclosed | Opposition experts | PPARB challenge, Risk Africa 2022 | Source code escrow (PB-04); software hash in Kenya Gazette; KEBS PVoC type-approval before deployment | Datacraft Legal/Technical | 6M | T-365 to T-60 | LOW-MED |
| A9 | Statutory | Data Protection Act 2019: no DPO, no DPIA, breach notification failure | ODPC, civil society (Privacy International) | ODPC investigation threat (Part VI R9); Privacy International Kenya 2024 | DPO named on PMO Day 1; DPIA registered with ODPC before voter registration; breach notification SOP in place | Datacraft DPO | 2M | T-365 to T-1 | LOW |
| A10 | Statutory | Political Parties Act: party-agent access rights denied to KIEMS logs or tallying centre | Party agents, opposition | NASA 2017 lockout; Azimio 2022 | Written offer of agent accreditation 60 days before election; observer access to sanitised SOC dashboard (PB-07) | Datacraft/IEBC | 1M | T-60 | LOW |
| A11 | Statutory | Computer Misuse and Cybercrimes Act 2018 s.22: independent security researcher's probe treated as unauthorized access, tainting evidence | Researchers, civil society | No Kenya precedent; UK Computer Misuse Act chilling-effect parallel | Bug-bounty programme (PB-06) with safe-harbour clause; coordinate with CA CIRT-CC on researcher-engagement rules | Datacraft Legal | 3M | T-365 to T-180 | LOW |
| A12 | Statutory | PPADA s.92 fraudulent procurement: Datacraft accused of false statements in bid | EACC, opposition, rival bidder | Smartmatic PPARB challenge 2022; OT-Morpho one-day tender 2017 | Full PPRA compliance from Day 1; pre-bid debarment clearances on all entities; no subcontracting without disclosure | Datacraft Legal | 1M | Pre-award | MED |
| A13 | Statutory | Public Officer Ethics Act: IEBC official has undisclosed financial interest in Datacraft | EACC, opposition | Chiloba dismissal, procurement conflict findings 2017 | Conflict-of-interest declaration by all IEBC officials involved in evaluation and contract management | IEBC Legal | 0 | Pre-award | MED |
| A14 | Statutory | Access to Information Act 2016: IEBC refuses to produce KIEMS logs within statutory 21-day window | Civil society, petitioners | Repeated AfriCOG FOIA delays; Parliament PAC 2017 minutes dispute | IAB has standing access; log production SOP specifies 24-hour delivery in petition context | IEBC Legal / Datacraft | 1M | T-7 to T+21 | LOW |
| B1 | Evidential | Server-access refusal: IEBC declines to give Supreme Court IT expert access to KIEMS logs | Supreme Court | Raila v IEBC [2017] para 272 — this was dispositive | Pre-agreed server-access protocol with Supreme Court IT Registrar; named Datacraft technical witness (Part IV-B); IAB as standing expert | Datacraft Legal/Technical | 3M | Pre-election + T+7 | LOW |
| B2 | Evidential | KIEMS log integrity challenge: logs were altered after the fact | Opposition IT experts | Raila v IEBC [2017] server-access narrative | Merkle-chained append-only log signed by HSM; root hash published to public transparency log every 30 minutes on election day | Datacraft Technical | 5M | T-365 to T-1 | VERY LOW |
| B3 | Evidential | Forensic-image chain-of-custody: IEBC cannot prove who handled what and when | Opposition counsel | 2017 — IEBC "declined access" narrative | RFID asset log, dual-witness transfer chain, HSM-signed custody records | Datacraft/IEBC | 4M | T-180 to T+7 | LOW |
| B4 | Evidential | Party-agent expert-witness access denied in petition window | Petitioners | Azimio 2022 petition — technical experts could not access server logs | IAB members available as expert witnesses; pre-registered technical witness at Supreme Court | Datacraft Legal | 2M | T+7 | LOW |
| B5 | Evidential | Form 34A image quality — blurry, rotated, partial | Petitioners | Raila v IEBC [2017] paras 219-262; Cherera 4 dissent 2022 | Automated quality gate: resolution, signature field detection, QR code verification before publication; failed images flagged not suppressed | Datacraft Technical | 3M | T-365 to T-1 | LOW |
| B6 | Evidential | Form 34A signature-line completeness: presiding officer or party-agent signatures missing | Petitioners | Raila v IEBC [2017] forms challenge | In-app signing workflow with confirmation screen; KIEMS app records each signature event with timestamp and operator ID | Datacraft Technical | 2M | T-365 to T-1 | LOW |
| B7 | Evidential | Form 34A polling-station-code accuracy: wrong station code entered or QR code bound incorrectly | Petitioners | 2017 forms dispute; AfriCOG 2018 analysis | QR code binding verified against the master polling-station register at upload; mis-bound forms rejected automatically | Datacraft Technical | 1M | T-365 to T-1 | LOW |
| B8 | Evidential | Form 34A tally mathematics error: arithmetic wrong on the face of the form | Petitioners | 2017 forms challenge | KIEMS device computes the tally; presiding officer confirms on-screen before capture; OCR-validation of printed form on upload | Datacraft Technical | 1M | T-365 to T-1 | LOW |
| B9 | Evidential | Form 34B constituency tally reconciliation failure | Petitioners | 2017 — Form 34A/B aggregate mismatch | Automated reconciliation engine: Form 34B auto-generated from received 34As; discrepancies flagged before Returning Officer signs | Datacraft Technical | 2M | T-365 to T-1 | LOW |
| B10 | Evidential | Form 34C national tally reconciliation failure | Petitioners | Raila v IEBC [2017] paras 219-262 — dispositive annulment ground | Form 34C auto-generated from 34Bs; three-way cryptographic reconciliation published in real time; Cherera-class dissent pre-empted by commissioner live-access to tally | Datacraft Technical | 2M | T-365 to T-1 | LOW |
| B11 | Evidential | Timestamp integrity: KIEMS event timestamps manipulated to mask out-of-sequence actions | Opposition IT experts | 2017 server narrative; no direct Kenyan court finding | NTP-synchronised UTC timestamp from a Kenyan NTP server; timestamp embedded in TPM attestation quote; Merkle chain proves sequence | Datacraft Technical | 1M | T-365 to T-1 | VERY LOW |
| B12 | Evidential | Geo-location metadata on transmitted forms: coordinates don't match registered polling station | Opposition IT experts | Indirect — 2022 geo-fencing requirement added to 2027 tender | GPS fix attached to every Form 34A upload; geo-fence validation (10m tolerance) at RTS ingress; alerts if fix is anomalous | Datacraft Technical | 2M | T-365 to T-1 | LOW |
| B13 | Evidential | Device fingerprint / kit serial number binding: results claimed to come from a different kit than deployed | Opposition IT experts | Theoretical; OAG 2024 200 missing kits creates the attack vector | TPM-attested device fingerprint in every KIEMS event; kit-serial number in the Merkle node; asset log cross-reference | Datacraft Technical | 1M | T-365 to T-1 | LOW |
| B14 | Evidential | Voter turnout implausibility: precincts with anomalously high or low turnout | Opposition statisticians | 2017 argument — several precincts cited with >100% turnout in preliminary data | ECC real-time statistical anomaly alerts: any station exceeding 90% turnout before 17:00 triggers manual review; results published with raw polling-station data enabling independent audits | Datacraft Technical / IEBC | 2M | T-1 to T+7 | LOW-MED |
| B15 | Evidential | Voter roll manipulation: dead voters, duplicate registrations | Civil society, opposition | 2013 BVR duplicates; OAG 2024 ABIS audit | ABIS 1:N dedup with published dedup-clearance count; IAB oversight of dedup runs; weekly ABIS progress on public dashboard | Datacraft ABIS | 5M | T-365 to T-90 | LOW-MED |
| B16 | Evidential | ABIS 1:N dedup adequacy proof: petitioner claims dedup was insufficient | Petitioners | OAG audit 2024 on 2022 ABIS; ABIS backlog claim | ABIS deduplication completion certificate published and IAB-signed before voter roll locks; methodology published | Datacraft ABIS | 2M | T-90 | LOW |
| B17 | Evidential | KIEMS software audit trail admissibility: logs not in a form courts accept | IEBC Legal | Novel in Kenya; UK ICO digital evidence precedent | HSM-signed log; chain-of-custody affidavit from Datacraft's named witness; pre-brief to Supreme Court IT Registrar | Datacraft Legal | 2M | T-30 | LOW |
| B18 | Evidential | Independent technical inspection order non-compliance: IEBC ignores court order to produce logs | Supreme Court | Raila v IEBC [2017] para 272 | Pre-agreed production protocol; IEBC physical copy of logs held independently from Datacraft | IEBC Legal | 0 | T+7 | LOW |
| C1 | Cyber | KIEMS kit boot-time tamper: bootloader unlocked before election day | Red-team hackers, insider | Theoretical; no documented Kenya instance | TPM-attested secure boot; boot attestation quote sent to ECC at every power-on; kit with failed attestation blocked from KIEMS | Datacraft Technical | 3M | T-365 to T-1 | LOW |
| C2 | Cyber | Firmware injection at manufacture: Mechrevo OEM supply-chain compromise | Foreign state actors, grey-market | "Chinese kits phone home" narrative; supply-chain attack literature (SolarWinds 2020 analogy) | Independent firmware audit before FAI sign-off; software hash published in Kenya Gazette (Part IV-D); air-gapped provisioning network; KEBS PVoC type-approval | Datacraft Technical | 10M | T-365 to T-90 | LOW-MED |
| C3 | Cyber | Rogue root CA installation on kits: attacker installs a CA cert enabling MITM | Advanced persistent threat | Theoretical; no Kenya election specific instance | Certificate pinning on TLS connections; IEBC root CA generated and held in HSM; no CA import possible without HSM authorization; boot attestation detects cert store changes | Datacraft Technical | 2M | T-365 to T-1 | VERY LOW |
| C4 | Cyber | Man-in-the-middle attack: attacker intercepts kit-to-RTS transmission | Telecom insider, APT | 2013 RTS — GPRS downgrade by design (not MITM but similar degradation risk) | TLS 1.3 with certificate pinning; data-diode prevents download to kits; VPN tunnel on APN; transmission integrity verified by Merkle proof at RTS ingress | Datacraft Technical | 3M | T-365 to T-1 | VERY LOW |
| C5 | Cyber | SIM cloning / IMSI catcher at polling station | Criminal, foreign actor | Global election-tech threat; IMSI catcher use documented in Kenya by Citizen Lab | Dual SIM (two carriers); WiFi fallback; TPM-bound device identity independent of SIM; transmission retry logic | Datacraft Technical | 1M | T-365 to T-1 | LOW |
| C6 | Cyber | Base-station spoofing to force GPRS downgrade | Criminal, foreign actor | 2013 RTS — GPRS was the primary mode; AfriCOG cited deliberate base-station congestion hypothesis | 4G minimum required; 5G preferred; kit drops to 3G before 2G; 2G fallback disabled in modem firmware; WiFi fallback active | Datacraft Technical | 1M | T-365 to T-1 | LOW |
| C7 | Cyber | APN misconfiguration causing selective silent failure at targeted stations | Telco insider | 2013 RTS partial failure blamed partly on APN routing issues | APN configuration locked at provisioning; cross-carrier dual-SIM; ECC monitors per-station connectivity; silent failure threshold triggers alert | Datacraft Technical | 1M | T-180 to T-1 | LOW |
| C8 | Cyber | DNS hijacking of RTS ingress: traffic redirected to a fake server | APT, grey-market | No Kenya instance; documented in election contexts in Myanmar 2020 | DNS-over-HTTPS with pinned resolver; DNSSEC on IEBC domains; certificate pinning makes DNS redirect ineffective | Datacraft Technical | 1M | T-365 to T-1 | VERY LOW |
| C9 | Cyber | TLS misconfiguration: weak cipher suites, expired cert, HSTS not enabled | Security researcher | No Kenya-specific instance; discovered in pre-election audits regionally | Bug-bounty programme (PB-06); TLS configuration audited by independent party 90 days before election; automated cert-expiry alerting | Datacraft Technical | 2M | T-365 to T-90 | LOW |
| C10 | Cyber | HSM key extraction: attacker extracts signing keys from HSM | Insider, nation-state | Theoretical; Thales HSM extraction demonstrated in academic research (Nohl 2015 type) | FIPS 140-3 Level 3 HSM; dual-control activation; HSM in locked cage with tamper-evident seals; HSM audit log separate from main audit log | Datacraft Technical | 5M | T-365 to T-1 | VERY LOW |
| C11 | Cyber | Insider database-admin write to results table: DBA changes vote totals | Insider IEBC/Datacraft | 2017 "Chiloba server" narrative (The Elephant 2017) | Zero-trust: no human has unrestricted write access; all DB writes require HSM-signed CCB ticket + dual-control auth; every write is Merkle-logged | Datacraft Technical | 3M | T-365 to T-1 | VERY LOW |
| C12 | Cyber | Insider audit-log purge or rewrite | Insider | 2017 server-access narrative | Append-only log; no delete permission on any account; off-site IEBC-controlled replica; IAB has read access and would detect gap | Datacraft Technical | 2M | T-365 to T-1 | VERY LOW |
| C13 | Cyber | Backup server access under weaker controls | Insider, criminal | Generic IT security finding; no Kenya-specific instance | Backup server on same zero-trust policy as primary; IEBC-controlled off-site copy is read-only for Datacraft | Datacraft Technical | 1M | T-365 to T-1 | LOW |
| C14 | Cyber | Escrow-site access under weaker controls | Insider | NCC Group escrow breach risk | NCC Group (or tier-1 escrow) site has its own ISO 27001 controls; escrow is code only, not live credentials; tested annually | Datacraft Legal/Technical | 2M | T-365 to T-1 | LOW |
| C15 | Cyber | NTP spoofing: timestamp attack shifting event times | APT | No Kenya election instance; documented in generic attack literature | Kenyan NTP server (KENIC NTP pool); multiple NTP sources; TPM hardware timestamp cross-referenced | Datacraft Technical | 1M | T-365 to T-1 | VERY LOW |
| C16 | Cyber | Result-portal DDoS: portal down during results window ("they're hiding results" narrative) | Hacktivists, APT, coordinated opposition | Nation Digital portal 2017 partial outage; "portal was down" narrative (chatter topology #3) | Three geographically distributed read-only replicas; Cloudflare or equivalent DDoS mitigation; media pre-briefed on replica URLs | Datacraft Technical | 5M | T-90 to T+3 | LOW |
| C17 | Cyber | Result-portal defacement | Hacktivists | 2017 partial portal degradation; generic election website defacement (multiple global instances) | WAF with signature rules; Cloudflare page shield; read-only replica architecture means defacement of one node doesn't affect others | Datacraft Technical | 2M | T-365 to T-1 | LOW |
| C18 | Cyber | Result-portal SQL injection | External attacker | Generic web application risk | OWASP-compliant parameterized queries; WAF; quarterly DAST scans; bug-bounty programme (PB-06) | Datacraft Technical | 2M | T-365 to T-1 | LOW |
| C19 | Cyber | Session hijacking on party-agent portal | Criminal, opposition operative | Generic web risk | Short-lived JWT tokens; MFA for agent portal login; session revocation on anomaly detection | Datacraft Technical | 1M | T-365 to T-1 | LOW |
| C20 | Cyber | Mobile-app reverse-engineering: attacker decompiles KIEMS APK to find vulnerabilities | Security researcher, grey-market | Smartmatic APK analysis attempts 2022 (The Elephant) | APK obfuscation + root/emulator detection; bug-bounty safe harbour for legitimate researchers; code signing verified at install | Datacraft Technical | 2M | T-365 to T-1 | LOW-MED |
| C21 | Cyber | Signed-form verification bypass in Public Verifier app | Security researcher | Novel; no Kenya precedent | Cryptographic verification uses IEBC root CA pinned in app; Verifier app open-source (PB-02) so bypass attempts are publicly visible | Datacraft Technical | 1M | T-365 to T-1 | VERY LOW |
| C22 | Cyber | Fake Verifier-app clone on Play Store / App Store | Criminal, disinformation actor | Multiple election-adjacent fake-app instances globally (Brazil 2022, US 2020) | IEBC-branded developer account verified by Google/Apple before launch; press announcement with official download links; CA CIRT-CC notified to issue public warning about fakes | Datacraft/IEBC Comms | 1M | T-365 to T-1 | LOW |
| C23 | Cyber | Fake Form 34A images circulated as tamper evidence | Opposition operative, disinformation actor | Doctored Form 34A images circulated on Twitter/X in 2017 and 2022 (Nation, Aug 2022) | Public Verifier app enables any citizen to verify hash; fake images fail hash check; rapid-response comms cell issues takedown notice | Datacraft/IEBC Comms | 2M | T+1 to T+7 | LOW-MED |
| C24 | Cyber | Deepfake video of IEBC officials | Disinformation-for-hire | No Kenya election-specific instance; documented in other African contexts 2023-2024 | Rapid-response comms cell; authenticated video channel on IEBC YouTube; IEBC officials' authentic statements timestamped and posted | Datacraft/IEBC Comms | 1M | T+1 to T+7 | MED |
| C25 | Cyber | AI-generated bulk complaints flooding petition window | Opposition operatives, disinformation | Novel; no Kenya precedent; UK House of Lords AI election risk report 2024 | IEBC complaints triage system with human review; Supreme Court IT Registrar alerted to AI-bulk filing risk; Datacraft pre-packages evidence rebuttals | Datacraft Legal | 1M | T+1 to T+7 | MED |
| C26 | Cyber | Coordinated inauthentic behaviour (CIB) on social media in the 72-hour results window | Disinformation-for-hire, Russian-linked troll farms | CIB documented in Kenya elections 2017 and 2022 by Mozilla Foundation / Oxford Internet Institute | Real-time monitoring brief to CA, NCA, and DCI; pre-drafted counter-narrative for each rapid-response scenario | Datacraft/IEBC Comms | 2M | T+1 to T+7 | MED |
| C27 | Cyber | Ransomware in IEBC network 30 days before election | Criminal, APT | Airtel Kenya ransomware Feb 2024 (Nation); NTSA breach 2023 | Network segmentation: KIEMS production network air-gapped from IEBC corporate LAN; offline backups tested monthly; incident response playbook rehearsed | Datacraft Security | 5M | T-365 to T-1 | MED |
| C28 | Cyber | Third-party supplier compromise: Al Ghurair printer, telco APN, ISP | Supply-chain attacker | SolarWinds 2020 (analogous); Al Ghurair chain-of-custody concerns (AfriCOG 2018) | Supplier security assessments; RFID tracking of ballot-paper consignments; APN configuration managed by Datacraft not telco; ISP SLA includes security obligations | Datacraft Security | 3M | T-365 to T-1 | MED |
| C29 | Cyber | Cloud provider outage / API rate-limit surprise | Operational | 2013 RTS crash — server capacity not pre-tested | Load-tested at 110% of expected peak in dry-run (PM-10); multi-cloud read replicas; rate limits pre-negotiated in SLA | Datacraft Technical | 3M | T-365 to T-1 | LOW |
| C30 | Cyber | CDN cache-poisoning of published forms | APT | Generic web risk | CDN cache keys include form hash; cache poisoning of one node does not affect the Merkle root; Public Verifier detects any hash mismatch | Datacraft Technical | 1M | T-365 to T-1 | VERY LOW |
| D1 | Physical | Kit theft in transit to polling station | Criminal, opposition operative | OAG 2024 — 200 kits unaccounted for post-2022 | RFID asset tracking end-to-end (PM-12); dual-officer escort for county-to-constituency transfer; GPS tracker in carry case | Datacraft/IEBC Logistics | 5M | T-30 to T-1 | LOW-MED |
| D2 | Physical | Kit sabotage by insider poll worker | Insider poll worker | EVID failure 2013 — sabotage vs malfunction disputed | Tamper-evident seal (numbered, logged); TPM attestation at power-on; backup kit at county level; training logged per officer | Datacraft/IEBC | 2M | T-7 to T-1 | LOW |
| D3 | Physical | Ballot-box stuffing at unstaffed polling station | Criminal operative | Multiple county court petitions 2017, 2022 (constituency level) | KIEMS BVI confirms voter identity before issuing ballot; biometric log timestamped per voter; double-voting detection at ABIS | IEBC/Datacraft | 1M | T-1 | LOW-MED |
| D4 | Physical | Ballot-paper substitution at printer (Al Ghurair chain-of-custody) | Political operative, printer insider | Chris Msando murder Aug 2017 linked to scrutiny of this chain (Nation investigations; KHRC); AfriCOG 2018 ballot-paper chapter | RFID tracking from printer to polling station; dual-witness sign-off at each transfer; security-printed serialisation on each ballot | IEBC / Al Ghurair | 5M | T-180 to T-1 | MED |
| D5 | Physical | Voter-roll last-minute alteration: names added or removed after registration cut-off | Insider IEBC, operative | 2013 BVR duplicates; repeated voter-roll complaints | Voter roll hash frozen and published at cut-off date; IAB certifies frozen roll; any discrepancy between kit roll and published hash is detectable | Datacraft/IEBC | 2M | T-90 | LOW |
| D6 | Physical | Poll-worker ghost recruitment: payment to non-existent workers | IEBC HR insider | PAC 2017 — ghost worker payments documented in audit | Biometric attendance registration for all poll workers using KIEMS; payroll cross-checked against biometric attendance log | IEBC HR / Datacraft | 2M | T-365 to T+7 | MED |
| D7 | Physical | Poll-worker impersonation: registered worker sends substitute | Criminal | Reported at constituency level in multiple cycles | Photo-ID check at polling station opening; biometric attendance for presiding officers | IEBC HR | 1M | T-1 | LOW-MED |
| D8 | Physical | Presiding officer coercion: forced to declare s.44A failure fraudulently | Political operative | No documented Kenya instance; regional precedent in Ghana 2020 challenges | Section 44A declaration requires confirmation from ECC, not presiding officer alone (PM-04 escalation ladder); anonymous coercion reporting channel | IEBC Security | 1M | T-1 | LOW-MED |
| D9 | Physical | Party-agent expulsion from polling station | Opposition operatives, IEBC staff | NASA 2017; Azimio 2022 — documented expulsions | Written accreditation; DCI officer assigned to each tallying centre; expulsion creates immediate ECC alert; rapid-response comms cell | IEBC/Datacraft | 1M | T-1 | LOW-MED |
| D10 | Physical | Media crew expulsion from tally centre | IEBC staff, ruling-party operative | 2017 Bomas restrictions; 2022 partial media access disputes | Media accreditation protocol 30 days before election; dedicated media briefing room at ECC; expulsion triggers rapid-response comms | IEBC/Datacraft Comms | 1M | T-1 | MED |
| D11 | Physical | Power outage at polling station | Infrastructure failure | 2013 EVID — 77% of Kenyan schools lacked grid power (AfriCOG 2014); 20,000 mAh power bank now mandatory | 20,000 mAh power bank mandatory (PRICING-GUIDANCE.md); solar-charging option offered; backup kit at county with charged battery | Datacraft Hardware | 5M | T-180 to T-1 | LOW |
| D12 | Physical | Network outage at polling station (rural 4G coverage gap) | Infrastructure failure | 2013 RTS GPRS failure (README.md; 2013-RTS-specs.md); 2017 satellite modems bought but only 339 of 1,000 deployed (AfriCOG 2018) | Dual-SIM (two carriers); satellite backup at high-risk remote stations; offline mode captures results; transmits when connectivity restored | Datacraft Technical | 10M | T-180 to T-1 | LOW-MED |
| D13 | Physical | Fire / flood / natural disaster at polling station | Force majeure | Standard electoral risk; not Kenya-election-specific | Complementary Mechanism pre-positioned; county spare-kit pool; IEBC emergency powers under Elections Act s.55A (postpone polling in affected station) | IEBC/Datacraft | 2M | T-1 | MED |
| D14 | Physical | Physical attack on tally centre | Political militants, criminal | 2017 and 2022 incidents at constituency tally centres (multiple Nation / Standard reports) | DCI + GSU perimeter security; Bomas of Kenya National Tallying Centre security plan pre-agreed with NPS; KIEMS results already uploaded before tally-centre physical attack can alter them | IEBC Security / Datacraft | 5M | T-1 to T+3 | MED |
| D15 | Physical | Political assassination of key technical staff (Chris Msando-class threat) | Political operatives, criminal | Chris Msando, IEBC ICT Director, murdered 31 July 2017, 8 days before the election (Nation; KHRC; Human Rights Watch) | Personal security protocols for all senior Datacraft and IEBC ICT staff 90 days before election; backup named officers for every critical role; DCI coordinated protection; stress-test continuity plan (PM-04 contingency) | Datacraft Security / DCI | 3M | T-90 to T+7 | MED |
| D16 | Physical | Bomas of Kenya National Tallying Centre security perimeter breach | Criminal, political militant | 2017 tally-centre access disputes; general Nairobi security risk | Pre-agreed perimeter security plan with NPS/GSU; access control verified against accredited-persons database; CCTV with ECC live feed | IEBC Security | 3M | T-1 to T+3 | MED |
| E1 | Procedural | Late tender specification change forcing re-scope after bid submission | IEBC procurement | Every Kenya election cycle — addenda and clarifications issued late | Pre-bid clarification (PRE_BID_CLARIFICATIONS.md); scope-change clause in contract with change-order pricing methodology; CCB approves all changes | Datacraft Legal | 1M | Pre-award | MED |
| E2 | Procedural | Late IEBC clarification invalidating prior work | IEBC procurement | 2017 KIEMS tender — two terminations, last-minute direct award | Change-order protocol locked in contract; IEBC clarification requires CCB written approval before Datacraft acts on it | Datacraft Legal | 1M | Pre-award to T-365 | MED |
| E3 | Procedural | Late gazettement of polling stations: stations added after kits provisioned | IEBC administration | 2022 — polling station boundary changes notified late | Contract clause: late gazettement changes after T-60 are a compensable change order; kit reprovisioning budget included | Datacraft/IEBC | 3M | T-180 to T-60 | LOW-MED |
| E4 | Procedural | Late voter-registration cut-off allowing roll manipulation | IEBC administration | 2017 — late registrations disputed | Voter roll hash frozen and published at contractually agreed cut-off date; no changes after hash publication | IEBC / Datacraft ABIS | 1M | T-90 | LOW |
| E5 | Procedural | Party-agent accreditation blocked or delayed | IEBC administration | Azimio 2022; NASA 2017 | Accreditation deadline published 60 days before election; written record of all applications and outcomes; any rejection must state grounds | IEBC / Datacraft | 0 | T-60 | LOW |
| E6 | Procedural | Media accreditation blocked or delayed | IEBC administration | 2017 Bomas restrictions | 30-day accreditation deadline; media-room capacity pre-agreed; accreditation disputes resolved by IEBC Media Committee within 5 days | IEBC Comms | 0 | T-30 | LOW-MED |
| E7 | Procedural | Observer accreditation blocked or delayed | IEBC administration | EU EOM 2017 — some operational access denied | Accreditation offered 60 days before election; observer access to sanitised SOC dashboard (PB-07); any denial triggers automatic IAB review | IEBC / Datacraft | 0 | T-60 | LOW |
| E8 | Procedural | Ballot-order dispute post-nomination: ballot paper printed with wrong order requiring reprint | IEBC / Al Ghurair | 2017 — candidate order disputes post-nomination | Ballot-order freeze date agreed in IEBC-Al Ghurair contract; Datacraft's CRMS locks candidate data at freeze date with IEBC-signed hash | IEBC / Al Ghurair | 5M | T-90 to T-60 | MED |
| E9 | Procedural | Nomination-list dispute changing candidate count per kit affecting BVI matchlist | IEBC / parties | 2022 — late withdrawals and substitutions | Kit voter roll and candidate list re-provisioned per update within 48 hours; candidate-count change triggers automatic re-provisioning workflow | Datacraft Technical | 2M | T-60 to T-7 | LOW-MED |
| E10 | Procedural | Recall petition mid-cycle changing ward/constituency boundaries | IEBC administration | Constituency boundary disputes have been filed mid-cycle; no directly relevant precedent during technology deployment | Contract clause: IEBC-initiated boundary change after T-90 is a compensable change order; scope impact assessed within 5 days | Datacraft Legal | 2M | T-90 | MED |
| E11 | Procedural | Court injunction pausing procurement | Competitor, opposition | Risk Africa PPARB challenge 2022; multiple High Court injunction applications against IEBC ICT tenders | Comprehensive PPRA compliance from Day 1; response team on standby; bid documents preserved; PPARB appeal history reviewed | Datacraft Legal | 3M | Pre-award | MED-HIGH |
| E12 | Procedural | Court injunction pausing kit deployment | Competitor, petitioner | Derived risk from E11; direct precedent would be a High Court order stopping KIEMS deployment | Preservation of all procurement records; IEBC's legal counsel retained to respond within 48 hours; PPARB process expedited | IEBC Legal / Datacraft | 2M | T-180 to T-30 | MED |
| E13 | Procedural | PPARB debarment mid-project | EACC, rival bidder | No Datacraft-specific precedent; Smartmatic PPARB challenge created uncertainty mid-cycle 2022 | PPRA compliance audit quarterly; no PPRA violations tolerated from Day 1; debarment search on all entities at bid time and quarterly | Datacraft Legal | 1M | T-365 to T-1 | LOW |
| E14 | Procedural | EACC investigation of named IEBC official mid-project | EACC | Chiloba dismissal mid-cycle 2017; commissioner conflicts 2022 | PMO at Anniversary Towers maintains direct documented relationship with multiple commissioners, not a single point of failure; EACC cooperation protocol pre-agreed | Datacraft / IEBC | 0 | T-365 to T-1 | MED |
| E15 | Procedural | Auditor-General special audit order mid-cycle | Auditor-General | OAG special audit 2017; OAG report 2024 | Auditor-General has read-only access to RFID asset management system and published dashboard at all times; cooperation protocol in place | IEBC / Datacraft | 0 | T-365 to T-1 | LOW |
| E16 | Procedural | Parliamentary vote of no confidence in IEBC Chair | Parliament | Cherera 4 aftermath 2022; IEBC Act amendment pressure | Datacraft maintains relationships across all commissioners, not only the Chair; IAB independent from individual commissioner | IEBC / Datacraft | 0 | T-365 to T-1 | MED |
| E17 | Procedural | Cabinet Secretary Interior withholding security deployment | Cabinet | No documented Kenya instance; theoretical given political tensions | Pre-agreed security deployment memorandum between IEBC and Ministry of Interior; NPS deployment order issued T-30 | IEBC / Ministry of Interior | 0 | T-30 | LOW-MED |
| F1 | Reputational | "Chinese kits phone home to Beijing" narrative | Opposition politicians, disinformation actors | Smartmatic "Venezuelan nationals" parallel, Standard Media 2022; predicted by Part VI R13 | Kenyan-face programme: 47 county coordinators all Kenyan; firmware audit (C2 mitigation); data-diode prevents outbound download; IAB report addresses; rapid-response comms | Datacraft Comms | 3M | T-365 to T+7 | MED |
| F2 | Reputational | "Datacraft is a shell company / bought the bid" narrative | Opposition, civil society | Kahawatungu 2019 — IEBC "fraudulently awarded" KIEMS to IDEMIA; similar narrative guaranteed | Full corporate history, ownership structure, and PPRA compliance certificate published proactively; audited accounts available on request | Datacraft Legal/Comms | 1M | Pre-award | MED |
| F3 | Reputational | "Foreign vendor circumvents 40% local content rule" narrative | Opposition, PPRA | Local-content requirement standard in Kenya public procurement; cited in 2022 Smartmatic debates | Teaming agreement makes Datacraft Kenya the prime tenderer; Kenyan staff ratio documented; supply-chain localisation plan published | Datacraft Legal | 1M | Pre-award | LOW-MED |
| F4 | Reputational | Named political figure claiming pre-fixed outcome | Opposition, political | Standard pattern every Kenya election; Uhuru 2013, Raila 2017/2022 | Cryptographic architecture makes pre-fixing mathematically demonstrable (not just claimable); IAB challenges any specific allegation within 48 hours | Datacraft/IEBC Comms | 2M | T+1 to T+7 | MED |
| F5 | Reputational | Whistleblower (real or fabricated) mid-cycle | Insider, disinformation | Msando murder created a martyr narrative; subsequent whistleblower culture | Named whistleblower channel with ODPC oversight; genuine complaints go to IAB, not suppressed; fabricated complaints challenged publicly | Datacraft/IEBC | 1M | T-365 to T+7 | MED |
| F6 | Reputational | Leaked internal-Slack / WhatsApp screenshots (real or fabricated) | Insider, disinformation | Multiple Kenyan cases of fabricated screenshots in political contexts (Standard 2022) | End-to-end encrypted internal comms; operational decisions recorded in formal CCB log not WhatsApp; rapid-response denial with CCB log reference | Datacraft Comms | 1M | T-365 to T+7 | MED |
| F7 | Reputational | Sudden viral "expert" video alleging KIEMS backdoor | Disinformation, disgruntled researcher | Occurred in 2017 (multiple YouTube/Twitter/X "forensic analysis" videos); near-certain recurrence | Pre-published technical white paper addressing all known backdoor allegations; rapid-response team responds within 4 hours; bug-bounty invites legitimate researchers | Datacraft/IEBC Comms | 2M | T+1 to T+7 | MED |
| F8 | Reputational | International media investigative piece (BBC / Al Jazeera / NYT) pre-election | Adversarial journalist | The Elephant and AfriCOG have published major pieces every cycle; international pick-up likely | Proactive media briefings; IAB provides independent expert comment; technical white paper pre-published; full cooperation with legitimate journalists | Datacraft/IEBC Comms | 2M | T-365 to T-1 | MED |
| F9 | Reputational | Large data dump (Wikileaks-style) | Hacktivists, foreign actor | No Kenya election-specific instance; DNC 2016 and Macron 2017 as global analogues | Data classification: no political content in KIEMS network; voter PII segregated and encrypted; breach notification SOP; rapid-response comms | Datacraft Security/Comms | 3M | T-365 to T+7 | MED |
| F10 | Reputational | Ratings-agency downgrade of Kenya sovereign linked to election-tech uncertainty | Financial markets | 2017 election period saw Eurobond yield spike; not directly triggered by tech uncertainty but linked | Not within Datacraft's direct control; IEBC transparency dashboard is the best available mitigation (stable election = stable credit narrative) | IEBC Comms | 0 | T-180 to T+7 | MED |
| F11 | Reputational | Diaspora-vote controversy: form, transmission, verification of overseas ballots | Opposition, diaspora groups | IEBC has not implemented diaspora voting; if introduced for 2027 the attack surface is new | If diaspora voting is out of scope, state clearly in contract; if in scope, apply same Merkle/verifier architecture as domestic | IEBC / Datacraft | TBD | T-365 | MED |
| F12 | Reputational | Cross-border commentary (Uganda / Tanzania / Somalia) on legitimacy | Regional governments, media | Routine in East African election cycles; Tanzanian and Ugandan state media sometimes amplify opposition narratives | IAB's public quarterly reports give regional journalists independent expert source; IEBC communicates with AU and East African Community observer missions | IEBC Comms | 0 | T-365 to T+7 | LOW |
| F13 | Reputational | AU / SADC observer mission negative interim statement | Observer missions | 2017 — some observer missions were slower to endorse; Carter Center expressed "concerns" | Observer access to sanitised SOC dashboard (PB-07); daily ECC briefings; proactive engagement with AU-EOM lead before election day | IEBC/Datacraft Comms | 1M | T-30 to T+3 | LOW-MED |
| F14 | Reputational | US State Department / EU statement pre-election expressing concern | Foreign governments | 2017 and 2022 — US and EU statements cited procedural concerns | IAB quarterly reports give credible independent evidence; Datacraft's transparency dashboard reduces grounds for concern | IEBC Comms | 0 | T-90 to T-1 | LOW-MED |
| F15 | Reputational | Church-council public letter opposing electronic voting | NCCK, Catholic Bishops | NCCK has commented on election technology in every cycle; Catholic bishops issued statement on KIEMS 2017 | Pre-election pastoral outreach: invite church leaders to observe the dry-run (PM-10); ELOG and Datacraft provide briefing to faith-community leadership | Datacraft/IEBC Comms | 1M | T-180 | LOW |
| F16 | Reputational | Community-elder mobilisation against biometric voting | Ethnic community leaders | 2013 — some communities resisted biometric registration (particularly North Eastern region) | Voter education campaign (Part IV-C); county coordinator is community-familiar; s.44A complementary mechanism means no voter is disenfranchised by biometric failure | IEBC/Datacraft | 3M | T-180 | LOW-MED |
| G1 | Financial | Late payment from IEBC causing Datacraft-Mechrevo cascade failure | IEBC Finance | Smartmatic KES 570M debt suit post-2022 (Standard Media); OT-Morpho KES 3.9bn outstanding post-2017 | Milestone-based payment schedule with IEBC-held advance; APG lodged by Datacraft; escrow release mechanic in case of IEBC insolvency | Datacraft Finance | 2M | T-365 to T+90 | MED |
| G2 | Financial | FX shock (KES/USD/CNY) mid-contract | Currency markets | 2017 — KES depreciated ~15% during election year; KES has lost 40% vs USD 2020-2026 | Contract priced KES; hardware BOM in USD with FX-band clause: if KES/USD moves >10% from base rate, change-order mechanism triggers renegotiation | Datacraft Finance | 2M | T-365 to T+90 | MED |
| G3 | Financial | Change-order requests eroding margin | IEBC procurement | Every Kenya election cycle — late scope additions | Change-order pricing methodology agreed in contract; CCB must approve all changes; T&M rates pre-agreed in commercial annex | Datacraft Finance | 1M | T-365 to T-1 | MED |
| G4 | Financial | Warranty-claim inflation: IEBC overclaims defective kits | IEBC | OAG 2024 — 3,433 kits found "faulty"; definition of faulty disputed | Acceptance testing protocol with IEBC quality inspector sign-off per batch; defect vs damage definition locked in contract; IAB adjudicates disputes | Datacraft Legal | 1M | T+7 to T+365 | LOW-MED |
| G5 | Financial | Liquidated damages applied for failure not attributable to Datacraft | IEBC | Standard contractual risk; Smartmatic LD dispute post-2022 | LD exclusion clause: LDs do not apply for s.44A failures caused by power outage, network failure, or IEBC-directed change; cause-analysis protocol pre-agreed | Datacraft Legal | 1M | T+7 to T+90 | MED |
| G6 | Financial | Sovereign-guarantee non-materialisation | Kenya Government | Not a historical Kenya IEBC case; regional precedent in Ethiopia, Zambia | APG issued by a Kenyan tier-1 bank; payment milestones structured to reduce exposure; advance payment covers first-batch hardware cost | Datacraft Finance | 1M | T-365 to T-180 | LOW-MED |
| G7 | Financial | Insurance claim denial post-incident | Insurer | Generic commercial risk | Marine cargo all-risks insurance from a Lloyd's-market insurer; PI insurance covering Datacraft; coverage reviewed by independent broker before bid | Datacraft Finance | 1M | T-365 to T+90 | LOW |
| G8 | Financial | Advance-payment guarantee called by IEBC: performance failure claimed | IEBC | Standard contractual risk; no Kenya-specific precedent with Datacraft | APG language specifies grounds for call; dispute resolution clause before call; CCB log as evidence of performance | Datacraft Legal | 1M | T+7 to T+90 | LOW |
| G9 | Financial | Tax dispute: withholding, VAT on foreign services, transfer pricing | Kenya Revenue Authority | KRA has audited IEBC vendors; transfer pricing is a growing KRA focus (KRA TP unit active since 2018) | Kenya-China DTA withholding applied correctly; transfer-pricing documentation maintained; KRA ruling sought pre-award | Datacraft Finance | 2M | T-365 to T+365 | MED |
| G10 | Financial | Insurance-cover gap on election-day incidents | Insurer | No Kenya-specific precedent; risk is real | PI insurance reviewed for election-day scope; IEBC indemnity for IEBC-directed s.44A declarations; gap analysis by insurance broker pre-award | Datacraft Finance | 1M | T-365 to T-1 | LOW-MED |
| H1 | Personnel | IEBC ICT Director compromised | Political operative, criminal | Ezra Chiloba era — ICT Director fired, PAC 2017; preceding director's conduct disputed (AfriCOG 2018) | Zero-trust architecture means IEBC ICT Director does not have unilateral access to KIEMS production; dual-control for all critical operations | Datacraft Technical | 2M | T-365 to T-1 | LOW |
| H2 | Personnel | IEBC commissioner compromised | Political operative | Cherera 4 dynamic 2022 — four commissioners publicly dissented, undermining result announcement | Commissioners each have read-only live access to tally data; no single commissioner can alter results; IAB independent from commissioner majority | Datacraft / IEBC | 0 | T-365 to T+3 | MED |
| H3 | Personnel | Datacraft staff coerced or bribed | Criminal, political operative | Chris Msando precedent (murder); bribery is a standard election-tech risk globally | Personal security protocols for all senior staff T-90 to T+7; anonymous reporting channel; staff rotation means no single person has full system knowledge | Datacraft Security | 2M | T-90 to T+7 | MED |
| H4 | Personnel | Mechrevo staff exfiltrating firmware designs | Competitor, foreign intelligence | Supply-chain IP theft is documented in Chinese ODM sector (generic, no election-specific Kenya instance) | NDAs with IP assignment clauses; firmware built on air-gapped system at Mechrevo; code escrow (PB-04) protects IEBC even if Mechrevo IP is compromised | Datacraft Legal | 1M | T-365 to T-1 | MED |
| H5 | Personnel | Ex-employee sabotage post-termination | Disgruntled former employee | Generic IT security risk; no Kenya election-specific instance | Immediate credential revocation on termination; session key rotation post-departure; HSM re-key if senior technical staff depart within T-180 to T+7 | Datacraft HR/Technical | 1M | T-365 to T+7 | LOW |
| H6 | Personnel | Contractor with legitimate access exceeding scope | Insider, criminal | Generic; Smartmatic Venezuelan nationals story 2022 was partly about foreign staff access scope | All contractor access scoped to minimum necessary; access rights reviewed by IAB quarterly; nationality-transparent staff roster published | Datacraft HR/Technical | 1M | T-365 to T+7 | LOW |
| H7 | Personnel | Chief Justice / Supreme Court judge indirectly influenced | Political operative | No direct documented Kenyan precedent; institutional concern raised by civil society post-2017 | Not within Datacraft's control; Datacraft's role is to provide technically impeccable evidence; pre-agreed server-access protocol removes judicial discretion on access | Datacraft Legal | 0 | T+7 | MED |
| H8 | Personnel | Media-house financial capture: favourable election-tech narrative purchased | Political operative | Standard Kenya media-funding concern; cited in AfriCOG 2018 | Datacraft holds no financial relationships with Kenyan media; IAB's independent quarterly reports are the credible counter-narrative | Datacraft Comms | 0 | T-365 to T+7 | MED |
| H9 | Personnel | Party agent doubling as vendor employee | Conflict of interest | No documented KIEMS-specific instance; concern raised in 2017 PAC proceedings generally | Employment and consultancy roster reviewed against party membership lists; any overlap disclosed to IEBC and IAB | Datacraft HR | 0 | T-365 to T-1 | LOW |
| H10 | Personnel | Poll worker with prior political affiliation not vetted | IEBC HR | 2017 — poll worker neutrality concerns raised by observer missions | IEBC background screening; Datacraft's training curriculum includes a conflict-of-interest declaration; biometric attendance creates an accountability record | IEBC HR | 0 | T-180 | MED |
| I1 | Supply chain | Al Ghurair (UAE printer) delayed or compromised | Logistics failure, political operative | Chris Msando murdered in context of scrutiny of ballot-paper chain (Nation; KHRC); AfriCOG 2018 dedicated chapter on ballot-paper security | RFID tracking from printer to polling station; independent security audit of Al Ghurair facility; dual-printer contract if volume allows | IEBC / Al Ghurair | 10M | T-180 to T-30 | MED |
| I2 | Supply chain | Iris sensor OEM single-source lockout | Supply-chain failure | PRICING-GUIDANCE.md §6.2 — iris sensor is highest-uncertainty BOM line; 12-16 week lead time | Dual-source iris sensor (IriShield + second OEM); buffer stock of 120%; purchase order before bid deadline | Mechrevo Hardware | 5M | T-365 to T-180 | MED-HIGH |
| I3 | Supply chain | HSM firmware update introducing regression | Vendor update | Thales Luna firmware regression documented in enterprise security incidents 2022 | Freeze HSM firmware version 90 days before election; firmware update review board; regression tested in sandbox before production | Datacraft Technical | 1M | T-90 | LOW |
| I4 | Supply chain | ABIS vendor licence-key withdrawal | ABIS vendor | Smartmatic debt dispute analogy — vendor withdrawal of software during live election (hypothetical but plausible given Kenya's payment history) | ABIS licence pre-paid for full contract term; source-code escrow (PB-04) covers ABIS engine; perpetual licence clause in ABIS contract | Datacraft Legal | 2M | T-365 to T-1 | LOW |
| I5 | Supply chain | Cloud provider account suspension | AWS/Azure/GCP | No Kenya election-specific instance; cloud suspensions have affected NGOs and government entities in Africa (hypothetical) | On-premise primary; cloud replicas are secondary; on-premise system functions independently of cloud | Datacraft Technical | 2M | T-365 to T-1 | LOW |
| I6 | Supply chain | Telco APN misconfigured mid-election-day | Telco insider, operational | 2013 RTS — APN routing issues contributed to failure | Dual-SIM on two carriers; APN configuration locked and tested T-7; ECC monitors per-carrier traffic split in real time | Datacraft Technical | 1M | T-365 to T-1 | LOW-MED |
| I7 | Supply chain | Satellite backup uplink weather blackout | Environment | 2017 — 1,000 satellite modems procured; only 339 deployed (AfriCOG 2018); equipment still not tested at scale | Satellite backup is secondary to 4G; high-risk remote stations have both 4G and VSAT; weather blackout is time-limited; offline mode caches results | Datacraft Technical | 5M | T-180 to T-1 | LOW-MED |
| I8 | Supply chain | Courier chain-of-custody breakdown | Logistics provider | OAG 2024 — 200 kits missing post-2022 | RFID tracking (PM-12); GPS in carry cases for high-value consignments; dual-officer handover at each transfer | Datacraft/IEBC Logistics | 3M | T-30 to T-1 | LOW |
| I9 | Supply chain | Kenyan sub-partner (logistics / training / security-printer) default | Sub-partner failure | Generic; Datacraft's sub-partners not yet contracted | Sub-partner performance bonds; dual-source for logistics; training cascade does not rely on a single sub-partner | Datacraft Legal | 2M | T-365 to T-90 | MED |
| I10 | Supply chain | Kenyan bank refusing to onboard payment flow | Banking compliance | KYC-related bank delays are documented in Kenyan procurement (generic); Part VI R2 covers APG issuance risk | Approach two banks simultaneously from T-12; daily follow-up from T-8; alternative bank identified from T-6 | Datacraft Finance | 0 | T-365 to T-335 | MED |
| J1 | Foreign interference | Israeli-linked disinformation firm commissioned by a candidate (Team Jorge / Nikuv / Percepto style) | Foreign grey-market, domestic political operative | Team Jorge documented in Haaretz / Guardian investigation Feb 2023 as active across 33 countries; Nikuv active in Kenya voter-roll manipulation (Nikuv / IEBC 2013 procurement allegations, Nation 2013) | Bug-bounty + independent code audit removes exploitable technical vulnerabilities; Datacraft's transparency architecture means disinformation about technical manipulation is disprovable; coordination with NIS and DCI | Datacraft Security / NIS | 3M | T-365 to T+7 | MED-HIGH |
| J2 | Foreign interference | Russian troll-farm bulk automation | Russian state-affiliated groups | CIB documented in Kenya 2017 election by Stanford Internet Observatory; Russian IRA activity in East Africa ongoing | Monitoring brief to CA and NCA; pre-drafted counter-narrative; platform-level takedown coordination with Twitter/Meta/TikTok teams | IEBC/Datacraft Comms | 2M | T-90 to T+7 | MED |
| J3 | Foreign interference | PRC-adjacent perception attack (for or against, both plausible) | PRC-affiliated groups | PRC has run perception operations in multiple African elections; "Chinese kits phone home" narrative (F1) is the attack; a counter-attack defending PRC narrative is equally plausible | Firmware audit (C2); IAB publicly certifies system independence; Datacraft's transparency dashboard is the primary defence | Datacraft/IEBC | 2M | T-365 to T+7 | MED |
| J4 | Foreign interference | US-linked NGO capture argument: IEBC compromised by Western democracy-promotion funds | Opposition, PRC-adjacent narrative | NED / USAID / NDI funding of ELOG and observer missions is regularly cited by government-aligned media as interference | Separate Datacraft from observer-mission funding chains; IAB includes non-Western civil-society nominees; OAG oversees IEBC international funding | IEBC Comms | 0 | T-365 to T+7 | LOW-MED |
| J5 | Foreign interference | EU pre-election statement weaponised in Kenyan discourse | EU, Kenyan political actors | EU EOM statements in 2017 and 2022 were selectively quoted to support opposite political arguments | Pre-brief EU EOM on Datacraft's architecture; IAB report gives EU observers an independent technical source; Datacraft does not comment on EU statements | IEBC Comms | 0 | T-90 to T+7 | LOW-MED |
| K1 | Post-election | Petition filed within 7-day window (Article 140 Constitution) | Opposition candidate / party | Every contested presidential election since 2007; mandatory legal risk | Pre-packaged petition-defence brief (PB-08); named Datacraft technical witness; HSM-signed log production within 24 hours of petition filing | Datacraft Legal | 3M | T+7 | GREEN (expected) |
| K2 | Post-election | Interim orders freezing inauguration | Petitioner's counsel | 2017 — Supreme Court ordered stay; 2022 — application made but dismissed | Pre-agreed log-access protocol means IEBC can immediately produce evidence to resist interim orders; IAB as standing expert | IEBC Legal / Datacraft | 2M | T+7 to T+14 | LOW-MED |
| K3 | Post-election | Foreign-arbitrator ISDS claim against Kenya from Datacraft (defensive — Datacraft must not invoke) | Datacraft (if wrongly motivated) | No Kenya IEBC ISDS precedent; investor-state claims have been filed against Kenya (e.g. Cortec Mining) | Contract includes Kenya-seated ICSID arbitration only as last resort; Datacraft's interest is in completing the contract and being paid, not in suing the sovereign | Datacraft Legal | 1M | T+90 | VERY LOW |
| K4 | Post-election | Consumer-fraud class action from voters (novel but plausible) | Civil society lawyers | No Kenya precedent; South Africa class-action culture growing; Kenya class-action framework under CPA 2012 | DPA 2019 DPIA filed; Data Subject Rights compliance; no deceptive data practices | Datacraft Legal | 2M | T+90 | VERY LOW |
| K5 | Post-election | Retrospective PPARB debarment | PPARB, rival bidder | Risk Africa PPARB challenge to Smartmatic contract 2022 | PPRA compliance maintained throughout contract; all procurement documents preserved in CCB log | Datacraft Legal | 1M | T+90 to T+365 | LOW |
| K6 | Post-election | Retrospective EACC investigation of Datacraft | EACC | Standard post-election audit of large government contracts | Full financial transparency; no kickbacks or facilitation payments; EACC cooperation protocol pre-agreed | Datacraft Legal | 1M | T+90 to T+365 | LOW |
| K7 | Post-election | Retrospective KRA transfer-pricing audit | KRA TP Unit | KRA transfer-pricing assessments on large Chinese-funded Kenya projects have increased since 2021 (KRA TP Guidelines 2022) | Transfer-pricing documentation maintained at arm's-length standards; KRA ruling sought pre-award; intercompany services agreement between Datacraft and Mechrevo at arm's length | Datacraft Finance | 2M | T+90 to T+365 | MED |
| K8 | Post-election | Retrospective ODPC breach investigation | ODPC | ODPC has issued investigation notices to government contractors post-2019; Privacy International Kenya 2024 | DPA compliance throughout (DPIA filed, breach notification SOP, DPO on PMO); ODPC briefed proactively | Datacraft DPO | 1M | T+90 to T+365 | LOW |
| K9 | Post-election | Judicial review of IEBC decision where Datacraft's system is exhibit | Petitioner | Any challenged IEBC administrative decision using KIEMS data as evidence | HSM-signed log admissible as documentary evidence; Datacraft's technical witness available within 24 hours | Datacraft Legal | 2M | T+90 to T+365 | LOW |
| K10 | Post-election | Parliamentary select-committee summons of Datacraft staff | PAC, relevant Parliamentary Committee | PAC summoned IEBC officials over every major contract since 2013; Datacraft staff are next-in-line | Full cooperation protocol; CCB log and asset records available to Committee; counsel present at all Committee appearances | Datacraft Legal | 1M | T+90 to T+365 | LOW |

---

### Per-Item Detail Sections

#### A1 — Article 81 Biometric Exclusion

**Attack:** Any voter who cannot provide a qualifying fingerprint (amputation, skin condition, age-related ridge degradation) and who is turned away may ground an Article 81 "free and fair" challenge. The 2013 Court dismissed a similar argument on "substantial compliance" but the current Supreme Court panel may not apply the same standard.

**Mitigation detail:** s.44A Complementary Mechanism pre-signed at every station before 6:00 AM (Part IV-C). The readiness form, photographed and uploaded via KIEMS at station opening, proves the fallback materials were available before any BVI failure. The KIEMS app also accepts a face-match override authorised by the Presiding Officer where fingerprint is impossible; the override is logged with the reason code and cannot be used for more than 2% of voters at any station before it triggers an ECC alert.

**Source of attack pattern:** Petition 5/2013 paras 67-88; ELOG 2017 field observations.

#### A2 — Article 86 Verifiability

**Attack:** Article 86(a) requires elections to be conducted by "simple, accurate, verifiable, secure, accountable and transparent" methods. The 2017 Court found the process violated this standard because the public could not independently verify results (paras 219-262). The attack is structural: any opaque KIEMS fails this test regardless of whether it was actually manipulated.

**Mitigation detail:** Public Verifier app (PB-02) enables any citizen with a smartphone to enter a polling-station code and receive: the Form 34A image, the tally, and the SHA-256 hash of both, verifiable against the IEBC public transparency log. The Merkle root is published every 30 minutes. A command-line tool (open-source, MIT licence) allows technical users to independently re-derive the root. The Kenya Gazette publishes the software hash 30 days before election day.

**Source of attack pattern:** Raila Amolo Odinga v IEBC [2017] eKLR para 219-262 (dispositive annulment ground).

#### A3 — Article 88 IEBC Independence / Vendor Control

**Attack:** Article 88 mandates IEBC's independence. If KIEMS is materially controlled by the vendor — as argued in 2022 regarding Smartmatic's Venezuelan technical staff — the election violates Article 88. The "Chinese nationals" variant of this attack is near-certain.

**Mitigation detail:** Code escrow (PB-04) with IEBC as named beneficiary means IEBC can operate the system without Datacraft. Zero-trust architecture means no Datacraft engineer has unilateral production access. The IAB (PM-08) continuously verifies independence and publishes quarterly reports. The HSM is owned by IEBC. All custom software vests in IEBC on operational acceptance (Teaming Agreement Clause 4.1). Nationality-transparent staff roster published proactively.

**Source of attack pattern:** Standard Media 2022 "Venezuelan nationals" coverage; Smartmatic arguments in Petition E005/2022.

#### B2 — KIEMS Log Integrity Challenge

**Attack:** After results are announced, opposition IT experts claim the KIEMS logs were retroactively altered to match the announced result. The 2017 "declined access" precedent (Raila v IEBC [2017] para 272) established that inability to show the logs creates an adverse inference. Showing doctored logs could be worse.

**Mitigation detail:** Merkle-chained append-only log: every KIEMS event produces a hash node; nodes are chained sequentially; the root is published to the IEBC public transparency log every 30 minutes during election day. Retroactive alteration of any node changes the root, which is publicly observable. The append-only property is enforced at the database level (no DELETE, no UPDATE, INSERT-only on the event table); the constraint is verified by the IAB quarterly. Off-site IEBC-controlled replica is replicated in near-real-time; the replica hash must match the primary.

**Source of attack pattern:** Raila Amolo Odinga v IEBC [2017] eKLR para 272.

#### C2 — Firmware Injection at Manufacture

**Attack:** The political attack line "Chinese kits phone home to Beijing" will be deployed regardless of technical reality. But the underlying technical risk is real: a Mechrevo OEM employee could inject malicious firmware that exfiltrates data or subtly alters tallies before the kit is shipped to Kenya. This is the supply-chain attack analogue of SolarWinds.

**Mitigation detail:** (1) Firmware baselined and signed at OEM under dual-control witness (Datacraft engineer plus independent auditor) before any kit leaves the factory. (2) The exact firmware image hash is published in the Kenya Gazette 30 days before election day, signed by IEBC Chair and Datacraft Programme Director. (3) Every kit's TPM attestation quote at power-on contains the firmware hash; the ECC receives this quote and compares it against the Gazette-published baseline. Any kit with a non-matching firmware hash is automatically blocked and an alert issued. (4) Provisioning uses an air-gapped network at the IEBC facility; no Mechrevo-controlled network connection exists after firmware baseline. (5) KEBS PVoC type-approval before shipment from China is a legal requirement (PVoC mandated for ICT goods under Kenya's Import Inspection and Conformity Programme); the PVoC inspector reviews the firmware at origin.

**Source of attack pattern:** Standard Media 2022 "Venezuelan nationals"; SolarWinds 2020 (analogous supply-chain attack pattern); AfriCOG 2018 on foreign-vendor access.

#### D4 — Ballot-Paper Substitution / Al Ghurair Chain-of-Custody

**Attack:** Pre-printed ballot papers are the most analogue and hardest-to-detect manipulation vector. Chris Msando, IEBC's ICT Director, was found murdered on 31 July 2017, eight days before the election. Nation investigations (August 2017; subsequent KHRC report) linked his death to his scrutiny of the ballot-paper chain-of-custody, specifically the Al Ghurair contract. AfriCOG (2018) devoted a dedicated chapter to the ballot-paper security concerns. Any future scrutiny of the ballot-paper supply chain carries this historical shadow.

**Mitigation detail:** (1) RFID tracking from printer (Al Ghurair Dubai or equivalent) to polling station: each consignment is tagged at print-to-pack; tags read at pack-to-container, container-to-Nairobi port, port-to-IEBC warehouse, warehouse-to-constituency, constituency-to-polling-station. (2) Dual-witness sign-off (IEBC officer plus independent security auditor) at each transfer point. (3) Discrepancies generate an immediate ECC alert. (4) Ballot paper serialisation (security-printed unique serial range per consignment) enables reconciliation against polling-station records. (5) Independent security audit of Al Ghurair facility during print run, by an IEBC-appointed auditor.

**Source of attack pattern:** Chris Msando murder, Nation, 31 July 2017; KHRC investigation report; AfriCOG "Dark Business of Election 2017" ballot-paper chapter.

#### D15 — Chris Msando-Class Threat (Political Assassination of Key Technical Staff)

**Attack:** Chris Msando, IEBC's ICT Director, was murdered 31 July 2017. His murder eliminated the person who knew most about the KIEMS technical configuration and the ballot-paper chain-of-custody, eight days before the election. No one has been convicted. The pattern — eliminating a technical insider who poses an integrity risk to one side — is specific enough to warrant a named mitigation.

**Mitigation detail:** (1) Personal security protocols for all senior Datacraft and IEBC ICT staff from T-90 to T+7: secure transport, vetted accommodation, direct DCI liaison. (2) Every critical technical role has a named backup officer with full technical access, so elimination of one person cannot disable the function. (3) All critical system credentials are split across at least two officers (Shamir's Secret Sharing or equivalent); no single person holds a complete credential set. (4) Pre-agreed emergency protocol: if a senior technical officer is incapacitated, the backup officer assumes duties without delay and the IAB is notified immediately. (5) The HSM-signed audit log continues to run without human intervention; its integrity is not dependent on any individual.

**Source of attack pattern:** Chris Msando murder, Nation, 31 July 2017; KHRC; Human Rights Watch; AfriCOG 2018.

#### F1 — "Chinese Kits Phone Home" Narrative

**Attack:** Standard Media 2022 story on Smartmatic Venezuelan nationals established the precedent "foreigners ran our election." For 2027 the variant is: "Mechrevo is a Chinese company under Beijing's control; its kits transmit Kenyan voter data to Chinese servers." This narrative does not require any factual basis to cause damage; the 2022 version circulated for months before IEBC responded.

**Mitigation detail:** (1) Data-diode architecture at the network level: KIEMS kits can upload to the Kenyan RTS (transmit only); the national tallying centre cannot transmit instructions back to a kit during polling hours (Part IV-D). This is enforced at the network layer, not application policy. (2) Firmware audit by an independent lab (Kenyan-based preferred, e.g. Serianu or KCA University Cybersecurity Lab) before FAI sign-off. The audit specifically looks for outbound connections to non-IEBC endpoints. (3) Kenyan-face programme: all 47 county coordinators are Kenyan nationals; Mechrevo secondees' roles are hardware QA only, not system operation. (4) IAB quarterly report explicitly addresses the "foreign control" question using the audit findings. (5) Rapid-response comms cell has a pre-drafted holding statement for this specific attack.

**Source of attack pattern:** Standard Media 2022; Part VI R13 in this strategy document; Smartmatic/Venezuelan nationals precedent.

#### I2 — Iris Sensor OEM Single-Source Lockout

**Attack:** The iris sensor is the only hardware component on the 2027 kit with no historical Kenya IEBC benchmark. PRICING-GUIDANCE.md §6.2 identifies it as the single highest-uncertainty BOM line with a 12-16 week minimum lead time. If the sole iris OEM cannot deliver — due to export control, production failure, or political pressure — the kit cannot meet the 2027 tender specification and Mechrevo's entire bid fails on technical grounds.

**Mitigation detail:** (1) Dual-source iris sensor at bid time: primary IriShield MK2120UL-class; secondary a different OEM (e.g. EyeLock, Iris ID, or Samsung equivalent) with same ISO/IEC 19794-6 compliance. (2) Purchase order placed before bid deadline even if contract award not yet made; Mechrevo absorbs the cost if bid is lost (this is the "insurance premium" for winning). (3) Buffer stock of 120% of required quantity. (4) Lead-time risk escalated to Steering Committee immediately if any delay signal appears. (5) If iris is undeliverable, invoke pre-agreed fallback specification (fingerprint-only with face backup) subject to IEBC written consent.

**Source of attack pattern:** PRICING-GUIDANCE.md §6.2; 2027-preparatory-specs.md new iris requirement.

#### J1 — Team Jorge-Class Foreign Disinformation Operator

**Attack:** The Team Jorge operation (Haaretz / Guardian investigation, February 2023) documented an Israeli-based firm offering influence campaigns across 33 elections globally, including in sub-Saharan Africa. Nikuv International Projects is specifically linked to IEBC procurement disputes: Nikuv was accused of corrupting the 2012 Kenyan BVR tender process (Nation, 2013; Kahawatungu 2019). These operators do not rely on compromising the KIEMS system — they manufacture the appearance of compromise through fabricated evidence, coordinated social media, and coached witnesses.

**Mitigation detail:** (1) The primary defence is removing exploitable technical vulnerabilities through the bug-bounty programme (PB-06) and the IAB: if the system is technically clean and independently certified, manufactured evidence fails cross-examination. (2) Pre-election transparency dashboard and Public Verifier app mean citizens can check claims themselves, reducing the viral propagation window of disinformation. (3) Coordination with NIS (National Intelligence Service) and DCI: if intelligence identifies a foreign operator campaign, law-enforcement action is the escalation path, not Datacraft's communications team. (4) IEBC registers with Meta, Twitter/X, and TikTok trust-and-safety election-integrity programmes to enable rapid takedown requests.

**Source of attack pattern:** Haaretz / Guardian "Team Jorge" investigation, February 2023 (https://www.theguardian.com/world/2023/feb/15/team-jorge-disinformation-influence-election); Nation 2013 on Nikuv Kenya; Kahawatungu 2019.

---

### VIII.3 — Cross-Cutting Defences

Some mitigations neutralise many attacks simultaneously. These are the highest-leverage investments in the portfolio.

**1. Public Verifier App + Merkle Chain (PB-02 + Part IV-B)**

Neutralises: A2, B1, B2, B3, B5, B6, B7, B8, B9, B10, B11, B12, B13, C11, C12, C23, F4, F7.

The Merkle-chained event log, with roots published every 30 minutes and verifiable by any citizen with the open-source command-line tool, converts the entire Form 34A/B/C evidential challenge from a he-said/she-said dispute into a mathematical proof. The burden of proof in a petition shifts: a petitioner cannot claim results were altered without showing a hash discrepancy, which is either present in the public log or it is not. This single architectural choice eliminates the largest category of historical petition grounds.

**2. Pre-Registered Supreme Court Technical Witness (Part IV-B)**

Neutralises: B1, B2, B3, B4, B17, B18, C11.

The 2017 annulment (Raila v IEBC [2017] para 272) turned on IEBC's refusal to provide log access. A named Datacraft technical witness registered with the Supreme Court IT Registrar before election day, available within 24 hours of petition filing, converts refusal-of-access from a structural possibility into a pre-committed impossibility. The witness is prepared to produce any log entry in the form the court requires, under HSM-signed chain-of-custody affidavit, within the 24-hour window.

**3. Independent Assurance Board (PM-08)**

Neutralises: A3, B1, B2, B3, B14, B15, B16, C11, C12, E14, F2, F4, F7, F8, H1, H2, H6, I4.

The IAB is the institutional immune system for the entire programme. By providing continuous, independent, read-only access to the KIEMS audit log and publishing quarterly reports, the IAB makes the "hidden server" attack obsolete. Every attack that depends on information asymmetry between IEBC/Datacraft and the public is neutralised by the IAB's standing presence. IAB members are pre-qualified as expert witnesses; their quarterly reports constitute admissible expert evidence in a petition without requiring live court preparation.

**4. Code Escrow with Weekly Rolling Deposits (PB-04)**

Neutralises: A3, A8, C14, I4, K3.

The Smartmatic debt dispute (KES 570M) illustrated the leverage a vendor has over an IEBC that cannot operate the system independently. Code escrow, with IEBC as named beneficiary and automatic release on insolvency or material breach, eliminates that leverage. It also addresses the "source code not disclosed" attack (A8) by making source code available to IEBC (and therefore to the court) without requiring the vendor's cooperation.

**5. Zero-Trust Internal Architecture with Hash-Chained Audit Logs (Part IV-D)**

Neutralises: C11, C12, C13, H1, H3, H5, H6.

The zero-trust requirement — no human has unrestricted shell access to any production server; all operations require dual-control authentication plus HSM signature — means that even a fully compromised IEBC ICT Director or a bribed Datacraft DBA cannot write to the results table without leaving a detectable trace in the append-only HSM-signed log. The IAB verifies the zero-trust configuration quarterly and would detect any attempt to weaken it.

**6. Bug-Bounty Programme (PB-06)**

Neutralises: C1, C9, C17, C18, C20, C21, F7.

A 12-month open bug-bounty programme on HackerOne or Bugcrowd, with Kenya-domiciled researchers prioritised and payouts on a published scale, does two things. First, it harnesses the adversarial researcher community that would otherwise probe the system covertly and sell findings to political actors. Second, it creates a public record that Datacraft invited scrutiny and that found vulnerabilities were fixed — eliminating the "they knew about the bug and hid it" narrative. Every vulnerability fixed under the bug-bounty is a petition ground that cannot succeed.

**7. Cross-Party Observer Live Feed to KIEMS Logs (PB-07)**

Neutralises: A10, D9, D10, E5, E6, E7, F4, F13, F14.

Providing read-only, sanitised live access to the SOC dashboard for all major observer missions (KEOG, ELOG, EU EOM, Carter Center, IFES) and all registered party ICT agents converts these groups from potential adversaries into live witnesses for the integrity of the process. A party agent who has been watching the system in real time has significantly less credibility claiming "we were excluded" or "results were tampered" if the tampering would have been visible on their own dashboard feed.

**8. Pre-Election Dry-Run Witnessed by Supreme Court IT Registrar and Party Legal Teams (PM-10)**

Neutralises: B1, B11, B17, C29, D11, E11.

A full national dry-run 60 days before election day, observed by ELOG, EU EOM, all registered party ICT leads, and the Supreme Court IT Registrar, serves two purposes. Technical: it surfaces failure modes while there is still time to fix them. Legal: it creates a pre-election record of system operation that is witnessed by all parties. A petitioner arguing the system was fundamentally broken must contend with the dry-run observation record signed by their own party's ICT representative.

**9. Cryptographic Sortition for Post-Election Audit (Part IV-D)**

Neutralises: B14, K1, K2, K9.

A verifiable random function (VRF) seeded from the final Merkle root selects 5% of polling stations for a mandatory manual recount. The station selection is mathematically derived from the election data itself — any party can independently compute the selected stations from the published root. This eliminates the "you chose which stations to recount" attack surface that has derailed post-election audits in other jurisdictions.

**10. Datacraft Transparency Dashboard (Part IV-A)**

Neutralises: C16, C17, F1, F2, F4, F5, F6, F7, F8, F10, F11, F13, F14.

A live public dashboard showing voter registration progress, kit commissioning status, training completion, pre-election dry-run results, and on election day: turnout by county, kit-online percentage, RTS transmission progress, and Form 34A publication status — published continuously from contract Day 1 through result announcement — fills the information vacuum that political actors exploit. Every rumour narrative is answered by a live, publicly auditable data feed. The dashboard cannot prevent false claims but it dramatically reduces their half-life.

---

### VIII.4 — What Datacraft Cannot Mitigate

Honesty about residual risk is itself a mitigation: it prevents the "they promised a perfect election" attack after any failure.

**Political violence at national scale.** If election-related violence forces closure of polling stations in a significant number of counties, KIEMS results from those stations cannot be collected, and the constitutional validity of the election is in question. This is an IEBC and Government of Kenya risk. Datacraft's contract includes a force-majeure clause covering civil unrest; Datacraft's obligation is to redeploy kits and staff when security is restored, not to operate during active violence. The contract should make explicit that polling-station closures due to violence are not Datacraft's LDs exposure.

**IEBC commissioner-level willful misconduct.** If a majority of commissioners decide to announce results that are not supported by the KIEMS tally — as the Cherera 4 dissent implied could happen in reverse in 2022 — no technical system can prevent it. KIEMS produces the accurate count; if the Commission chooses to announce something different, Datacraft's published Merkle log becomes the evidence in the petition, not the prevention. Datacraft's role shifts from prevention to evidence provision. The contract cannot and does not warrant commissioners' conduct.

**Presidential invocation of emergency powers.** If the President invokes Article 58 emergency powers to postpone or cancel the election, no commercial contract provides a remedy. Datacraft's kit deployment and software systems are held in place; the contract would need to specify a standby-fee structure for this scenario. This is a negotiation point, not a technical mitigation.

**International sanctions on Mechrevo.** If the United States, European Union, or United Nations imposes financial or technology-transfer sanctions on Mechrevo specifically — as has happened to other Chinese technology firms (ZTE 2018, Huawei 2019) — Mechrevo's ability to supply kits or support may be legally blocked. Datacraft's mitigation is contract-level: the OEM supply obligation is with Mechrevo, with a right to substitute an alternative OEM in case of force majeure including sanctions. However, a sanctions event 90 days before election day with no pre-qualified alternative OEM would be practically unmitigable in time.

**Sovereign fiscal shock preventing IEBC payment.** Kenya's external debt position is under stress; IMF programme conditions constrain the fiscal space. If a sovereign balance-of-payments crisis causes the National Treasury to freeze the IEBC's payment appropriation, Mechrevo's hardware payment milestone is at risk. This is partly mitigated by the advance-payment structure (20% advance against APG), but a prolonged freeze would cascade. The contract should include a step-in right for Datacraft to pause deliveries if payment is more than 45 days overdue, without triggering LDs.

**Force majeure natural disaster.** Flooding in coastal counties, drought-related security incidents in North Eastern, or volcanic activity affecting Western Kenya infrastructure would displace polling stations and Datacraft's logistics. The contract allocates this risk to IEBC (IEBC determines which stations open and where). Datacraft's obligation is to redeploy kits to new or relocated stations within 72 hours of written notice, at a pre-agreed change-order rate.

---

### VIII.5 — Bid Narrative Integration

The 20 highest-priority items from VIII.2 land in the bid as follows:

| Item | Attack | Bid document | Section |
|---|---|---|---|
| A2 | Art. 86 verifiability | Technical Proposal | Vol. II, Section IV — Security Architecture; PB-02 Annex A |
| A3 | IEBC independence / vendor control | Technical Proposal + Undertaking Letter | Vol. II, Section IV; Undertaking Letter Clause 3 (code vest on acceptance) |
| B1 | Server-access refusal | Technical Proposal | Vol. II, Section IV; PB-08 (pre-packaged legal brief) as Annex C |
| B2 | Log integrity | Technical Proposal | Vol. II, Section IV (Merkle architecture diagram) |
| B5 | Form 34A image quality | Technical Proposal | Vol. II, Section III (quality gate) |
| B10 | Form 34C national tally | Technical Proposal | Vol. II, Section III (auto-reconciliation engine) |
| C2 | Firmware injection / Chinese OEM | Technical Proposal + Undertaking Letter | Vol. II, Section IV; Undertaking Letter Clause 5 (firmware audit commitment); Gazette hash commitment |
| C16 | Result-portal DDoS | Technical Proposal | Vol. II, Section IV (three-replica architecture) |
| C23 | Fake Form 34A circulation | Technical Proposal + Comms Annex | Vol. II, Section IV (PB-02 Verifier); Annex A — rapid-response cell |
| D4 | Ballot-paper chain-of-custody | Technical Proposal | Vol. II, Section III (RFID tracking; MoU with Al Ghurair named as a post-award deliverable) |
| D11 | Power outage at polling station | Technical Proposal | Vol. II, Section II (20,000 mAh power bank — mandatory BOM; Bill of Materials) |
| D12 | Network outage at polling station | Technical Proposal | Vol. II, Section II (dual-SIM; satellite backup) |
| D15 | Chris Msando-class staff threat | Technical Proposal | Vol. II, Section V (Security Operations; staff protection protocol; backup officer nominations) |
| E11 | Court injunction pausing procurement | Commercial Proposal + Undertaking | Vol. III — PPRA compliance declaration; Undertaking Letter Clause 1 (debarment clearance) |
| F1 | "Chinese kits phone home" | Technical Proposal + Comms Annex | Vol. II, Section IV (data-diode); Annex A (pre-drafted rapid-response statement) |
| G1 | Late IEBC payment cascade | Commercial Proposal | Vol. III, Section II (payment schedule; milestone structure; step-in clause) |
| G2 | FX shock | Commercial Proposal | Vol. III, Section II (FX-band clause; base rate published) |
| H2 | Commissioner compromised | Technical Proposal | Vol. II, Section IV (commissioner live read-only access; no commissioner can alter KIEMS tally) |
| I2 | Iris sensor single-source lockout | Technical Proposal | Vol. II, Section II (dual-source declaration; lead-time risk table; buffer stock commitment) |
| K1 | Petition filed (expected) | Technical Proposal + Undertaking Letter | Vol. II, Annex C (PB-08 brief); Undertaking Letter Clause 6 (named technical witness; 24-hour log production) |

**Post-award change-order candidates:** E1 (late spec change), E3 (late gazettement), E8 (ballot order reprint), G2 (FX band trigger), D13 (force-majeure disaster redeployment). All five should be listed explicitly in the Commercial Proposal as pre-agreed change-order scenarios with published day-rates, so IEBC evaluators know the price before the event — this itself is a reputational and legal mitigation.

---

*Part VIII prepared 2026-08-20 as a v2 augmentation. All references to court cases use standard Kenya Law Reports citation form. All references to specific dates and named individuals are sourced as indicated per-item. This Part does not change any commercial commitment made in Parts I-VII; it documents the risk landscape and Datacraft's pre-committed responses.*

---

## Part IX — Raising Public Confidence

### IX.1 — Baseline: What Confidence Looks Like Today

Afrobarometer Kenya surveys show trust in IEBC tracking below 50% in each post-election cycle measured after a disputed result. The 2018 round (post-2017 annulment) showed approximately 38% trust, with urban centres and Nyanza region near 20% (Afrobarometer Round 7 Kenya, 2018). Twaweza "Sauti za Wananchi" 2022 post-election tracking showed confidence recovering modestly but remaining below pre-2017 baseline. Ipsos Kenya tracking (2022 pre-election) showed IEBC approval at approximately 44%.

Trust-differential: younger voters (18-35) consistently lower trust than older cohorts; Nyanza and Coast lower than Central and Rift Valley; urban educated voters most likely to assert manipulation while rural voters more likely to accept announced results.

Three narratives that erode confidence: (1) "They hid the servers" — Raila Odinga v IEBC [2017] eKLR para 272 is now canonical; any opacity triggers this frame. (2) "They rigged Form 34A" — 2017 and 2022 forms disputes gave this narrative multiple reinforcing precedents. (3) "Foreigners run the system" — the Smartmatic Venezuelan-nationals story (Standard Media 2022) set this frame; the 2027 variant is Chinese control.

Two things that raise confidence: (a) transparency Kenyans can experience personally — not "trust the Commission" but "I checked it myself with my phone"; (b) independent validation by institutions Kenyans trust more than IEBC — ELOG, Carter Center, faith councils, KEBS, and universities all score higher trust than IEBC on election matters.

### IX.2 — The Public Confidence Programme

| # | Intervention | Target audience | Confidence lever | Datacraft role | Cost (KES) | Timing | Metric |
|---|---|---|---|---|---|---|---|
| A1 | Public Verifier app — scan or enter station code, see tally + hash | All Kenyan smartphone users (approx 27M) | Personal verification | Build + maintain (pro-bono, PB-02) | 5M | T-180 to T+30 | Downloads + verifications > 5M |
| A2 | WhatsApp short-code result check | Urban + periurban WhatsApp users | Personal verification | Build pipeline + integrate | 2M | T-90 to T+30 | Queries > 2M |
| A3 | SMS results by station code on 2G feature phone (*IEBC# short-code) | Rural + older voters | Personal verification | Datacraft-funded MoU with CA for free SMS | 3M | T-90 to T+30 | SMS queries > 1M |
| A4 | Live-tally public dashboard (real-time Form 34A/B/C publication) | Journalists, civil society, urban voters | Transparency | Build + host (3-replica architecture) | 5M | T-30 to T+30 | Dashboard uptime > 99.9%; viewers > 10M peak |
| A5 | Downloadable open dataset (every form, hashed and signed) | Researchers, data journalists, international observers | Reproducibility | Publish + maintain API | 1M | From Form 34A publication | Dataset downloads > 10,000 within 48 hours |
| A6 | Real-time transmission health-map (per-station kit-online status) | Journalists, observers, party agents | Process transparency | Build (extension of ECC dashboard) | 2M | T-1 to T+3 | Map update latency < 5 min |
| A7 | Hash-of-the-day in Government Printer Kenya Gazette | Legal record; petition window evidence | Legal anchoring | Coordinate with Government Printer | 500K | Daily T-30 to T+30 | 100% days published |
| B1 | ELOG/KEOG parallel vote tabulation integrated into pipeline | Civil society, observers | Independent validation | Read-only KIEMS data feed + signed form delivery | 2M | T-180 to T+3 | ELOG PVT result within 3% of IEBC announcement |
| B2 | Carter Center/EU EOM/IFES read-only KIEMS access | International observer missions | International independent validation | Sanitised SOC dashboard access (PB-07) | 500K | T-30 to T+7 | All major missions confirm access granted |
| B3 | KEBS pre-election kit-certification results published | Technical sceptics | Institutional validation | Facilitate KEBS PVoC type-approval + publish | 500K | T-90 | Certification published by T-90 |
| B4 | Public university partnership (JKUAT, Strathmore, USIU, UoN) — 3 independent audit teams | Academic community + educated voters | Technical independent validation | Recruit; provide IAB-level access | 5M | T-365 to T+30 | 3 universities publish independent audit reports |
| B5 | Faith-based inter-religious council (NCCK, KCCB, SUPKEM, Hindu Council) as joint validators | Faith communities | Community trust anchor | Briefing + access + quarterly IAB updates | 1M | T-180 to T+7 | Joint council statement before election day |
| B6 | Media consortium (NMG, Standard Group, RMS, Citizen) shared technical auditor | Media audiences | Media validation | Facilitate shared auditor access | 2M | T-90 to T+7 | Media consortium audit report published T-14 |
| B7 | KEPSA commercial-integrity witness | Business community | Commercial trust | Brief KEPSA; KEPSA observer at key events | 500K | T-90 to T+3 | KEPSA statement published |
| C1 | Pre-election explainer video series in 11 Kenyan languages | All registered voters | Understanding = trust | Produce (Datacraft-funded, IEBC-branded) | 5M | T-180 to T-14 | Views > 20M across platforms |
| C2 | Interactive "walk through my polling station" microsite | First-time voters | Familiarity reduces fear | Build (pro-bono) | 2M | T-90 to T-1 | Unique visitors > 2M |
| C3 | 47-county roadshow "Sema Sasa" townhalls — Datacraft ICT engineers answer live technical questions | Sceptical urban and periurban voters | Human face behind technology | Funded by Datacraft; facilitated with county coordinators | 5M | T-180 to T-30 | 47 county events |
| C4 | Grade 8 + Form 4 civic curriculum module on election technology | Youth pre-voters | Generational confidence | Produced pro-bono; distributed via MoE | 1M | T-365 | Module adopted in 100% public schools |
| C5 | Braille + sign-language + audio materials (Article 54, Article 82) | Voters with disabilities | Rights-recognition | Produced pro-bono | 1M | T-180 | Distributed to 100% of demand |
| D1 | 90-day pre-election myth-buster desk (60-minute response SLA) | Social media users, journalists | Active counter-narrative | Datacraft comms team + fact-check partners | 3M | T-90 to T+30 | Average response < 60 min |
| D2 | Election-day war-room with pre-drafted statements | Journalists, party agents, public | Speed = credibility | Rapid-response cell | 2M | T-1 to T+7 | All pre-drafted scenarios covered |
| D3 | Post-election 30-day continuous transparency briefings | Petition window stakeholders | Sustained confidence | Daily IEBC + Datacraft joint briefing | 1M | T+1 to T+30 | 30 consecutive daily briefings |
| E1 | Kit-certification ceremony live-streamed from Bomas of Kenya | Sceptical public | Ritual credibility | Produce + broadcast | 2M | T-90 | Live viewers > 500,000 |
| E2 | HSM key-ceremony live-streamed (IEBC Chair + CJ representative + Auditor-General representative) | Technical and legal community | Institutional accountability | Produce + broadcast | 1M | T-90 | Live viewers > 100,000 |
| E3 | Post-election kit-return + re-inventory public reconciliation | Civil society, OAG | Accountability | RFID reconciliation published live | 1M | T+7 to T+30 | 100% of kits accounted for within 30 days |
| F1 | 24x7 short-code complaint line with legal-aid triage | Any voter | Grievance resolution | Build + staff (Datacraft-funded) | 3M | T-30 to T+30 | 100% complaints acknowledged < 2 hours |
| F2 | Party-agent grievance app with real-time status | Party ICT agents | Agent confidence | Build + maintain | 1M | T-90 to T+30 | 100% of registered agents onboarded |
| F3 | IAB direct complaint line | Civil society, researchers | Independent escalation | IAB-operated; Datacraft provides infrastructure | 500K | T-180 to T+30 | IAB response < 48 hours |

### IX.3 — Measurement Framework

| Metric | Method | Baseline | Target |
|---|---|---|---|
| Afrobarometer trust-in-IEBC score | Post-election Afrobarometer survey | Approx 38% (2018) | No more than 5-point decline in any of 47 counties vs pre-election baseline |
| Petition volume | Supreme Court filing count | 2022: 1 presidential + numerous governors + MPs | < 50% of 2022 total across all elections |
| Social-media negative election-tech claims | NLP monitoring (Part XVI.2) | Establish T-90 baseline | < 20% net negative during results window |
| Voter turnout | IEBC official statistics | 2017: 79%; 2022: 65% | >= 70% |
| Public Verifier verification actions | App + API log | 0 (new) | > 5M actions during election day + 7 days |

### IX.4 — Datacraft's Specific Package

Pro-bono (Datacraft funds fully): A1 (Public Verifier app), A3 (SMS/CA MoU), C1 (explainer videos), C2 (microsite), C4 (school curriculum), C5 (accessibility materials), E2 (HSM key ceremony), F1 (complaint line infrastructure).

Co-funded with IEBC: C3 (Sema Sasa roadshow), D1-D3 (rapid-response cell), E1 (kit-certification ceremony).

Datacraft builds, IEBC operates: A2 (WhatsApp bot), A4 (live dashboard), A6 (transmission health-map), A7 (Gazette hash coordination), B1-B7 (validator access), F2-F3 (grievance channels).

Public confidence is Datacraft's KPI, not a byproduct.

---

## Part X — AI Utilisation

### X.1 — Principles for AI in an Election-Critical System

No AI in the result-determining pipeline. Deterministic, auditable, cryptographically signed forms are the only source of truth for any vote tally. Any AI inference in the counting pipeline creates an unauditable black box that becomes an instant petition ground under Article 86(a) and Raila Odinga v IEBC [2017] eKLR para 219-262.

AI is for the layer around the pipeline: quality control, anomaly detection, translation, accessibility, comms monitoring, and operations.

Every AI decision affecting a human interaction is explainable and reversible. No voter is turned away, no ballot is rejected, no result is held without a human decision confirmed in writing.

Every AI model in production has: a model card, an eval harness, a version-pinned deployment, and a rollback procedure. Model updates require CCB approval.

Sovereignty: AI models touching Kenyan personal data run on Kenyan-jurisdiction infrastructure or on-device. No biometric data transits a non-Kenyan cloud endpoint.

### X.2 — Concrete AI Applications

| # | Application | Model class | Where it runs | Human-in-loop | Attack surface added | Confidence lift |
|---|---|---|---|---|---|---|
| A1 | Face liveness detection during voter capture | Depth-map + texture CNN, <= 15MB quantised | On-device (kit) | Presiding officer override with dual-agent witness | Liveness evasion by advanced attacker; low probability | HIGH: eliminates paper-photo impersonation |
| A2 | Fingerprint quality scoring at capture | ISO 29794-4 quality model, edge inference | On-device | Presiding officer retake decision | None material | MEDIUM: reduces failed authentications |
| A3 | Iris quality scoring | ISO 29794-6 quality model, edge inference | On-device | Presiding officer retake decision | None material | MEDIUM: reduces failed authentications |
| A4 | Real-time OCR of ID document text vs manual entry | CRNN/TrOCR fine-tuned on Kenyan ID types | On-device | Presiding officer confirms mismatch | OCR adversarial patch; very low probability | MEDIUM: catches transcription errors |
| A5 | Voice-guided prompts in Kenyan languages (15 languages) | TTS model, Kenyan-accented voices | On-device | N/A — assistive | None | HIGH for accessibility voters |
| A6 | On-device anomaly detection (presiding officer processing > 30 voters in 10 minutes) | Rule-based threshold + simple classifier | On-device | ECC receives alert; presiding officer not blocked | False positives causing unnecessary alerts | MEDIUM: detects station-level fraud |
| B1 | Form 34A image quality gate (rotate/de-skew/OCR-check before publication) | Document AI pipeline | Central (Kenyan cloud) | Below-threshold forms flagged, never auto-rejected | None material | HIGH: eliminates blurry/rotated forms |
| B2 | Handwriting recognition on tally lines (human verification always required) | Kenyan-election-form-tuned HTR model | Central | Human review required before result recorded | Error in HTR creates discrepancy requiring re-verification | HIGH: speeds tally while preserving human authority |
| B3 | Anomaly detection on kit telemetry | LSTM/isolation forest on time series | Central | ECC alert; no autonomous action | Low | MEDIUM: early warning of operational problems |
| B4 | Duplicate-voter suggestion for ABIS reviewer (not autonomous) | ABIS 1:N matching engine with confidence score | Central (ABIS) | ABIS adjudicator makes all decisions | False positive creates unnecessary referrals; bounded | HIGH: faster dedup with human authority |
| B5 | Turnout-implausibility flag (> 3 sigma from constituency mean) | Statistical model on rolling update | Central | Results hold pending human review | False positives can delay publication | HIGH: detects ballot stuffing pattern |
| B6 | Time-series alert on transmission-gap patterns | Threshold + trend model | Central | ECC alert | Low | MEDIUM: early connectivity issue detection |
| B7 | ML-based SIEM (network + infrastructure anomaly detection) | Isolation forest / unsupervised | Central | SOC analyst reviews all alerts | False positives create alert fatigue; mitigated by tuning | HIGH: cybersecurity operations |
| C1 | Multilingual chatbot in 15 Kenyan languages (WhatsApp/Telegram/SMS) | LLM with election knowledge base, PII-stripped | Central (Kenyan tenant) | Escalates to human agent on complex queries | Prompt injection; mitigated by input sanitisation | HIGH: voter information access |
| C2 | Real-time translation of IEBC announcements | NMT model (NLLB-family) | Central | IEBC communications officer approves before publish | Mistranslation of sensitive content; back-translation QA mitigates | HIGH: multilingual Kenya audience |
| C3 | Text-to-speech in Kenyan-accented voices (radio + IVR) | TTS with Kenyan fine-tune | Central | Human reviews scripts | None material | HIGH for accessibility |
| C4 | Speech-to-text for grievance-line intake | ASR model, Kenyan-language fine-tune | Central | Human agent reviews transcript | Low | MEDIUM: accessibility for oral complainants |
| C5 | Automated summarisation of daily results per language | LLM summariser | Central (Kenyan tenant) | Communications officer approves | Hallucination risk; mitigated by structured input from verified data | HIGH: multilingual media |
| D1 | CIB detection on Kenyan social media | Graph-based CIB classifier + velocity model | Central (Kenyan cloud) | Human analyst reviews HIGH flags | False positives flagging organic traffic | HIGH: disinformation detection |
| D2 | Deepfake detection on viral video of "IEBC officials" | Deepfake detection CNN (FaceForensics-family) | Central | Human analyst confirms before action | Advanced deepfakes may evade current detectors | HIGH: election narrative protection |
| D3 | Fake-form detection (does this Form 34A match published hash?) | Hash lookup + visual similarity model | Central + Public Verifier | Anyone can run; human follows up on flagged | None material | HIGH: disinformation counter |
| D4 | Bot-network detection on hashtag-hijacking | Network analysis + bot-score model | Central | Human analyst reviews patterns | Legitimate coordinated campaigns may be flagged; human review mitigates | HIGH: election information environment |
| E1 | AI-assisted code review on every KIEMS software change | Static analysis + LLM code review | CI pipeline (Kenyan cloud) | Developer + security reviewer approve all changes | LLM misses novel attack patterns | MEDIUM: security baseline |
| E2 | AI-assisted contract-review of change-orders | LLM contract-review | Internal Datacraft | Commercial lead approves | None material | MEDIUM: consistency |
| E3 | Adaptive-difficulty poll-worker training assessments | IRT model on response patterns | Central | Trainer reviews flagged poor performers | None material | MEDIUM: training quality |
| E4 | Predictive maintenance from telemetry | Gradient-boosted failure predictor | Central | Logistics team acts on flagged kits | False positives cause unnecessary kit swaps; bounded | MEDIUM: operational reliability |
| E5 | Post-election forensic-timeline reconstruction for legal team | LLM-assisted timeline + evidence synthesis | Internal Datacraft | Legal team reviews all outputs | None material | HIGH: petition-defence speed |

### X.3 — What AI Does NOT Do

- No AI votes are counted, even as validation.
- No autonomous voter rejection.
- No AI-generated official communications without human sign-off and CCB log entry.
- No LLM in the result-determining pipeline at any layer.
- No training on live voter biometric data.
- No AI model deployed to production without IAB notification and a published model card.

### X.4 — Sovereignty and Infrastructure

On-device models: quantised (INT8 or lower), packaged into KIEMS build, zero internet calls during operation. All on-device inference is offline-capable.

Central AI: hosted on Kenyan-jurisdiction infrastructure (Konza, Icolo, Safaricom Cloud, or Liquid Telecom Nairobi). No biometric data leaves Kenyan jurisdiction.

LLM where used: routed through a Kenyan-tenant instance with PII-stripping middleware (names, national ID numbers, biometric references, and location data below county level are stripped before any cloud API call).

No training data leaves Kenya. No production AI model is trained on live voter data.

### X.5 — Reference Architecture

Kit layer (offline-capable): TPM-attested Android device. Models: liveness detection (A1), fingerprint quality (A2), iris quality (A3), ID OCR (A4), TTS/voice prompts (A5), anomaly detection (A6). Data flows: biometric capture → liveness check → quality check → authentication attempt → presiding-officer action log → HSM-signed event → Merkle node. Human checkpoints: presiding officer at every non-routine outcome; party agents as witnesses; ECC receives anomaly alerts.

Central layer (Kenyan cloud, 3-replica): Document AI pipeline (B1): form image → orientation detection → de-skew → OCR → quality score → human review queue or auto-publish. ABIS (B4): enrollment event → 1:N search → confidence-banded result → adjudicator queue or auto-clear. CIB/Deepfake (D1-D4): social media ingest → classifier pipeline → confidence score → human analyst queue or daily report. SIEM (B7): telemetry ingest → anomaly score → SOC alert queue. All central-layer decisions logged with model version, confidence score, input hash, and outcome. IAB has read access to all decision logs.

Citizen layer (public internet): Public Verifier app (hash lookup, purely deterministic). WhatsApp bot (C1): LLM on Kenyan-tenant instance, PII-stripped input, structured output from IEBC verified data.

### X.6 — What Datacraft Brings

The Lindela AI/data platform (cloud-native, LiteLLM integration on 62.169.25.77:4000) provides the LLM routing layer with PII-stripping middleware. The situ-cloudevents infrastructure provides the append-only event bus that powers the Merkle audit chain. These are force-multipliers: the election AI layer slots onto existing, tested infrastructure rather than being built from scratch.

On-device models: Mechrevo's hardware platform runs INT8 quantised models at required inference latency (< 500ms for liveness, < 200ms for quality scoring).

Honest scope: deepfake detection at frontier adversarial capability is a known hard problem. Datacraft's commitment is best-available open-model deployment with human review, not a guarantee of catching every state-level deepfake.

---

## Part XI — Reproducibility of Outcomes

An election outcome is reproducible if any independent observer, given only the published inputs, can re-derive every published result and every intermediate tally, and get bit-identical answers to what IEBC declared.

### XI.1 — What Reproducibility Means at Each Layer

Voter registration reproducibility: given the raw enrolment stream, the ABIS engine specification, and the deduplication rules (FAR threshold, tie-break order, adjudication decision log), any reviewer can re-run the dedup and land on the same voter roll.

KIEMS device-state reproducibility: given the signed device-identity chain (kit serial + TPM attestation + provisioning event log), the pre-provisioned station configuration, and the operator action log (every button press with timestamp, operator ID, and HSM signature), any reviewer can reconstruct the state of every kit at every moment of election day.

Results transmission reproducibility: given every signed Form 34A image and its structured JSON (candidate tallies, station code, timestamp, geolocation, operator signatures), the aggregation algorithm (simple integer sum), and the Merkle-chain sequence number as canonical order, any reviewer can reconstruct Form 34B, Form 34C, and the national tally, bit-identical.

Publication reproducibility: given the published Merkle root and the daily hash-of-day in the Government Printer, any reviewer can prove the results on the portal are the same results IEBC declared and have not been silently amended.

### XI.2 — Reproducibility Architecture

| # | Layer | Reproducibility mechanism | Public artefact | Verification cost to observer | Attack neutralised |
|---|---|---|---|---|---|
| A1 | ABIS dedup rule set | Published as executable pseudocode + test vectors | GitHub release + Kenya Gazette | Low: any developer can run test vectors | Dead-voter/duplicate-roll challenge (Part VIII B15, B16) |
| A2 | Tally algorithm | Executable pseudocode with test vectors; pure integer arithmetic | GitHub release | Low | Tally-manipulation challenge (Part VIII B8-B10) |
| A3 | Form-image OCR pipeline | Model versions + config + hashes published | GitHub release | Medium: GPU required; expected output hashes available | OCR-manipulation claim (Part VIII B5) |
| A4 | Random seeds | All randomness derived from Merkle root of prior phase (cryptographic sortition); no wall-clock seed | Published per phase | Low: deterministic from published Merkle root | Algorithm-bias challenge |
| B1 | KIEMS event stream | Append-only, HSM-signed, full event stream published (redacted for direct biometric data per DPA 2019 s.26) | Public download via IEBC API | Low: any developer can replay | Hidden-server narrative (2017-family) |
| B2 | Form image with tamper-evidence metadata | Kit-signed EXIF: kit serial, presiding officer ID, timestamp, GPS, station code, HSM signature | Published per form | Low: verify signature against IEBC root CA | Fake-form circulation (Part VIII C23) |
| B3 | Human decision records | Every override, adjudication, and exception logged as signed action record | Published in redacted form (no voter PII) | Low | Procedural challenge to any individual decision |
| B4 | Raw inputs (redacted per DPA 2019) | All inputs to the counting pipeline published; PII columns pseudonymised | Public download | Medium: large dataset | Any claim about input manipulation |
| C1 | Containerised pipeline | Every stage: pinned OCI image digest + source commit hash | GitHub releases | Medium: requires Docker | Any claim about pipeline substitution |
| C2 | No wall-clock reads | Every stage receives timestamp as explicit input | Documented in pipeline spec | Low: code inspection | Timestamp-manipulation claim (Part VIII B11) |
| C3 | No external state reads | Every dependency passed as explicit input | Documented in pipeline spec | Low | Network-manipulation claim |
| D1 | Reproduce-me bundle | Code + input manifest + expected output hash + Docker Compose; runs offline on 16 GB RAM laptop | Public download from IEBC and GitHub releases | Low: `make reproduce` | Any claim of non-reproducibility |
| D2 | Expected outputs file | Per-stage expected output hash published | Same as D1 | Very low: hash comparison | Any claim of stage-level manipulation |
| D3 | Bug-bounty cash award | First party to demonstrate any bit-discrepancy wins the award | Published bounty rules | N/A — incentive structure | Encourages independent verification attempts |
| E1 | Merkle root in Kenya Gazette | Daily printed Gazette anchors each phase Merkle root | Physical newspaper archive | Low | "Root was changed after publication" claim |
| E2 | Cross-published to Twitter/X, Reddit, GitHub Releases, IPFS | Five independent publication channels | All five channels public | Very low | Suppression of Merkle root |
| E3 | National radio bulletin | Root hash read aloud on KBC at 20:00 news each day during election window | Radio broadcast | Low | Digital-only manipulation |
| F1 | 15-year reproducibility commitment | Reproduce-me bundle with vendored dependencies; Ubuntu 26.04 LTS through 2036 | Code escrow (PB-04) | N/A — future-proofing | Post-election decade petition windows |
| G1 | Two-implementation ABIS validation | ABIS result must match independent second implementation within published tolerance | Both implementations published | High: IAB commissions | Single-vendor ABIS manipulation claim |
| G2 | Two-implementation OCR validation | OCR result must match independent second implementation | Both implementations published | High: IAB commissions | OCR-manipulation claim |

### XI.3 — Reproducibility Metrics

| Metric | Target |
|---|---|
| Full-pipeline bit-identical re-run success rate | 100.00% on same inputs |
| Cross-vendor validator agreement at result-affecting level | >= 99.99% |
| Publication-to-first-independent-reproduction time | < 24 hours after each phase |
| Bug-bounty discrepancy count | 0 |
| Reproduce-me bundle size | < 20 GB |
| Reproduce-me compute time on 16 GB RAM laptop | < 2 hours |
| Storage requirement | < 100 GB |

### XI.4 — The Scenario Nobody Has Ever Tested

On 15 August 2027 at 09:00, a mathematics graduate in Kisumu runs: `curl https://iebc.go.ke/reproduce/2027/bundle.tar.gz | tar xz && cd kiems-reproduce && make reproduce`. The Makefile downloads the input manifest (< 10 GB of signed form images + event logs), verifies the input manifest hash against the published Merkle root (< 1 second), runs the aggregation pipeline in a Docker Compose environment (< 2 hours on a 2025 laptop), outputs Form 34C as a JSON file, computes its SHA-256, and prints: `Form 34C hash: abc123... Published hash: abc123... MATCH.`

The graduate posts a screenshot to Twitter/X. The election is now reproducible in the public consciousness. Every political actor watching knows that any claim the results were manipulated is falsifiable by any graduate with a laptop.

Technical prerequisites: bundle < 20 GB compressed (Nairobi fibre download: approximately 3 hours at 20 Mbps). RAM <= 16 GB (standard university laptop). Storage <= 100 GB. Pipeline fully deterministic: no sampling, no randomness at runtime. The `make reproduce` command handles everything with no manual steps. Every dependency is vendored. The bundle requires no network connection to run.

### XI.5 — What Reproducibility Neutralises

Every "hidden servers" narrative from 2017 (Raila Odinga v IEBC [2017] eKLR para 272): closed, because there are no hidden inputs. Every "recount showed different" narrative: closed, because any recount uses published inputs. Every "algorithm favoured X" narrative: closed, because the algorithm is published and independently verified. Every "silent amendment of published forms" narrative: closed, because any amendment changes the Merkle root in the physical Kenya Gazette. Every "we don't know what really happened" narrative: closed, because any graduate can re-run the entire election in under 2 hours.

Cross-reference to Part VIII: closes A2 (Art.86 verifiability), B1 (server access), B2 (log integrity), B9-B10 (tally reconciliation), C11 (insider DB write), C12 (log purge), C23 (fake forms), F4 (pre-fixed outcome claim), F7 (backdoor narrative).

### XI.6 — Cost and Packaging

Datacraft cost to deliver reproducibility infrastructure: KES 8-12M (pipeline engineering, bundle tooling, IAB reproducibility audit, second-implementation commissions).

Datacraft delivers: reproduce-me bundle, aggregation pipeline code, pipeline documentation, test vectors, bug-bounty rules, second-implementation commissioning process.

IEBC operates: Gazette publication coordination, Government Printer MoU, IPFS node, radio-bulletin script delivery, reproduce-me bundle hosting.

Bid packaging: reproducibility commitment becomes a standalone Undertaking Letter "Reproducibility Commitment" signed alongside the Subcontract Agreement, listing: bundle delivery date, expected output hash publication date, bug-bounty award schedule, IAB reproducibility audit date.

---

## Part XII — The No-Challenge Guarantee

### XII.1 — Honest Framing

Any Kenyan with USD 100 and a lawyer can file a petition. Article 140 gives every voter standing. The No-Challenge Guarantee is not a promise that no petition is filed. It is the practical asymptote of what a technology vendor can offer: any petition filed is (a) evidentially hollow, (b) procedurally dismissible at first hearing, (c) reputationally costly to the petitioner, (d) unfundable by rational political actors because the return-on-litigation is negative.

Stated plainly: Datacraft guarantees that no petition based on a deficiency in the KIEMS system, the results transmission, the form-processing pipeline, or the audit trail will succeed in the Supreme Court. Petitions based on IEBC commissioner misconduct, presidential emergency powers, political violence, or judicial capture are outside this guarantee. Those residuals are named in Part VIII VIII.4.

### XII.2 — The Four-Wall Architecture

Wall 1 — Evidentiary saturation: every fact a petitioner could allege is pre-published, pre-hashed, pre-signed, pre-witnessed, and independently reproducible (Part XI). Nothing left to discover in litigation.

Wall 2 — Procedural pre-emption: every ground on which any previous Kenyan petition succeeded (Raila 2017) or plausibly could have succeeded (Raila 2022) is pre-neutralised in Part VIII with a named mitigation, a pre-appointed witness, a pre-agreed protocol, and a documented compliance record.

Wall 3 — Reputational cost imposition: a petition on hollow evidence in the face of a saturated public record damages the petitioner. The reproduce-me bundle (Part XI) means any technically literate journalist can independently verify the result within 2 hours.

Wall 4 — Deterrent economics: petition legal costs run KES 200M+ for a Supreme Court case. When evidentiary and procedural walls are up, the expected value of a challenge is negative for every serious political actor. See XII.6.

### XII.3 — Cause-of-Action Pre-Emption Matrix

| # | Statutory ground | Historical use | Datacraft pre-emption | Evidence produced pre-emptively | Named custodian | Cross-reference |
|---|---|---|---|---|---|---|
| 1 | Constitution Art.81(e) — universal suffrage | 2013 exclusion argument (Petition 5/2013 paras 67-88) | s.44A pre-signed at every station; face-match override; no voter turned away | Complementary Mechanism readiness form per station, uploaded at station opening | Presiding officer + IEBC Ops | Part VIII A1, Part XIV |
| 2 | Constitution Art.86(a) — simple method | 2013, 2017: KIEMS argued too complex | Public Verifier: scan code, see result | Verifier app + source code published T-30 | Datacraft Technical | Part VIII A2, Part XI |
| 3 | Constitution Art.86(a) — accurate method | 2017: tally errors (Raila v IEBC [2017] eKLR paras 219-262) | OCR validation + manual cross-check + arithmetic sanity gate | Every Form 34A with kit-computed and OCR-confirmed tally | Returning officer + Datacraft | Part VIII B5-B10, Part XVII |
| 4 | Constitution Art.86(a) — verifiable method | 2017: dispositive annulment ground | Merkle chain + reproduce-me bundle + Public Verifier | Merkle root in Kenya Gazette; bundle published at result | IEBC Chair (signed) | Part XI |
| 5 | Constitution Art.86(a) — secure method | 2022: Venezuelan nationals / foreign control | Zero-trust architecture; firmware audit; data-diode; KEBS PVoC | Firmware hash in Gazette; KEBS certificate; IAB quarterly reports | Datacraft Technical + IAB | Part VIII A3, C1-C4 |
| 6 | Constitution Art.86(a) — accountable method | Every cycle: IEBC not accountable for KIEMS decisions | Merkle-chained audit log; HSM-signed; IAB standing access; dual-control | Full event log; IAB has read access continuously | IAB + IEBC Chair | Part VIII A3 |
| 7 | Constitution Art.86(a) — transparent method | 2017: server not produced | Public dashboard; IAB; party-agent live access; reproduce-me bundle | Continuous public transparency dashboard; IAB quarterly reports | Datacraft + IEBC Comms | Part IX, Part XI |
| 8 | Constitution Art.88(4)(h) — IEBC independence | 2022: vendor control / Venezuelan nationals | Code escrow; code vests in IEBC on acceptance; zero-trust; no foreign nationals in SOC | Code escrow certificate; teaming agreement clause 4.1 | Datacraft Legal | Part VIII A3 |
| 9 | Constitution Art.138 — presidential election transmission | 2017: transmission failure dispositive | Data-diode upload; 3-replica portal; Merkle proof at ingress | Merkle path per form; per-kit upload confirmation | Datacraft Technical | Part VIII A4, A6 |
| 10 | Constitution Art.140 — petition procedure | Every cycle | Pre-registered technical witness; log production within 24 hours; pre-packaged brief | T-14 evidentiary bundle delivered to Supreme Court IT Registrar | Datacraft Legal witness | Part VIII K1, XII.4 |
| 11 | Elections Act s.39 — electronic transmission | 2017: chain-of-custody (Raila v IEBC [2017] eKLR paras 219-262) | TPM-signed upload; Merkle chain; Gazette-documented transmission format | Form of transmission published in Gazette T-30 | Datacraft Technical | Part VIII A6 |
| 12 | Elections Act s.44A — complementary mechanism | 2017: ELOG report on invocation without notice | Pre-agreed escalation ladder; party-agent notification in real time | SMS notification log per s.44A invocation | IEBC Operations | Part VIII A7 |
| 13 | Elections Act s.83 — recount | Every cycle (constituency level) | Cryptographic sortition audit (5% recount) | Sortition seed (Merkle root-derived); recount protocol published | Datacraft Technical + IEBC | Part XI |
| 14 | Election (Technology) Regulations 2017 | 2022 challenge | KEBS PVoC type-approval; source-code escrow; Gazette hash | KEBS certificate; escrow certificate; Gazette hash | Datacraft Legal | Part VIII A8 |
| 15 | Data Protection Act 2019 | ODPC investigation threat | DPO named; DPIA registered; breach notification SOP | ODPC DPIA registration certificate | Datacraft DPO | Part VIII A9 |
| 16 | Computer Misuse and Cybercrimes Act 2018 s.22 | Researcher-hostile chilling-effect risk | Bug-bounty safe-harbour clause; CA CIRT-CC coordination | Bug-bounty programme rules with safe-harbour published | Datacraft Legal | Part VIII A11 |
| 17 | PPADA s.92 — fraudulent procurement | 2022 PPARB challenge; 2017 direct-award controversy | Full PPRA compliance; debarment clearances; no undisclosed subcontracting | PPRA compliance certificate; debarment clearance certificates | Datacraft Legal | Part VIII A12 |
| 18 | Election Offences Act 2016 s.6-7 — bribery/undue influence | Each cycle at constituency level | Election Offences Act evidence-package infrastructure (Part XVIII) | Evidence packages routed to ODPP | Datacraft + IEBC | Part XVIII |
| 19 | Access to Information Act 2016 s.4 | AfriCOG FOIA delays | IAB has standing access; log production SOP; 24-hour delivery in petition context | IAB access log; SOP documented | IEBC Legal + Datacraft | Part VIII A14 |

### XII.4 — The Pre-Emptive Evidentiary Bundle

Published to Supreme Court IT Registrar, Chief Registrar of the Judiciary, and Attorney General simultaneously at T-14. All 15 items published under a Merkle root gazetted in the Kenya Gazette Supplement at T-7.

1. Kit-manufacture provenance chain: RFID + firmware hash + KEBS PVoC certificate per kit serial
2. Firmware hash certification: signed by Datacraft PD + independent auditor + IEBC Chair
3. Source-code escrow certification: NCC Group current-deposit certificate
4. ABIS engine reproducibility bundle: dedup-rule pseudocode + test vectors + result hash
5. RTS pipeline reproducibility bundle: aggregation algorithm + test vectors + result hash
6. HSM key-ceremony video + custodian affidavits: IEBC Chair, CJ representative, Auditor-General representative
7. Party-agent training completion register: signed, per agent per station
8. KEOG/ELOG/Carter Center/EU EOM access-grant letters: dated, signed by IEBC Chair
9. Datacraft insurance certificates: PI, cargo, public liability
10. Public Verifier app source + published store links: commit hash, store listing links
11. Communication cadence log: every IEBC-Datacraft formal communication from contract Day 1
12. IAB membership + all quarterly reports published to date
13. Bug-bounty programme history: all reported findings, resolution status, payout record
14. Pre-election tabletop exercise reports: dry-run witness affidavits + signed attendance
15. Pre-election dry-run witness affidavits: ELOG/Carter Center/party ICT leads + Supreme Court IT Registrar representative

### XII.5 — The Pre-Appointment Mechanism

Pre-appointed at T-90, published to IEBC website and press at T-90:

- Two named Supreme Court IT Registry technical staff (confirmed by Chief Justice's office)
- Two named Attorney General's Office election-law counsel (confirmed by AG)
- Named senior counsel on retainer for IEBC petition defence
- Named forensic-IT firm (KPMG DFI / Deloitte Forensic / PwC Advisory Kenya)
- Named PR firm for 30-day petition-window communication
- Named observer-mission technical liaisons from IFES, Carter Center, and EU EOM

### XII.6 — Ex-Ante Litigation-Risk Pricing

Presidential petition legal cost estimate: KES 200M+ (counsel fees, expert witnesses, court filing, logistics).

Expected probability of success given Datacraft's architecture: near-zero on any KIEMS-system ground. The reproduce-me bundle allows independent verification in 2 hours; the Merkle log is publicly auditable; the pre-appointed forensic-IT firm and technical witness remove all information-asymmetry advantages that a petitioner exploited in Raila Odinga v IEBC [2017] eKLR para 272.

Political cost of a dismissed petition: severe reputational blow given the availability of independent verification. The 2022 petition (Raila Odinga v William S Ruto & 3 Others [2022] eKLR Petition E005/2022) was dismissed at the first hearing; the architecture here makes dismissal at first hearing even more likely.

Expected value of a petition on KIEMS grounds: negative for any rational political actor.

### XII.7 — Datacraft Ships vs IEBC Owns

Contractual deliverables: reproduce-me bundle; Public Verifier app; IAB infrastructure; pre-packaged legal brief (PB-08); log production in legal form; forensic-IT firm access protocol; party-agent access protocol; bug-bounty programme rules.

MoU-based: access grants to observer missions; Supreme Court IT Registrar pre-registration; KEBS PVoC type-approval process.

Standing services: technical witness availability for 30 days post-election; IAB reporting for 12 months.

The pre-emptive bundle (XII.4) is a scheduled contractual deliverable with a T-14 deadline, LDs if missed.

### XII.8 — What No Vendor Can Guarantee

IEBC commissioner-level willful misconduct: Datacraft's published Merkle log becomes the evidence in the petition, not the prevention. Risk sits with: IEBC commissioners.

Presidential emergency powers (Art.58): no commercial contract overrides. Risk sits with: Kenyan political class.

Political violence at scale: if violence displaces enough polling stations that constitutional validity is questionable, KIEMS is a secondary concern. Risk sits with: NSAC, KNPS, Cabinet.

Foreign-state coercion of Kenyan institutions: no technical architecture prevents a foreign state from applying political pressure to individual officials. Risk sits with: NIS, KNPS, President.

Judicial capture: if the Supreme Court is captured, evidence quality is irrelevant. Risk sits with: Chief Justice, Judicial Service Commission.

### XII.9 — Bid Narrative Packaging

The No-Challenge Guarantee lands as standalone Volume V "No-Challenge Guarantee & Pre-Emptive Evidence Programme" appended to the Technical Proposal.

Cost-of-quality items in base price: technical witness; pre-packaged legal brief; log production infrastructure; IAB.

Pro-bono: pre-appointment facilitation; Gazette hash coordination; observer-mission access grants.

Milestone deliverables: evidentiary bundle at T-14; technical witness registration at T-30; forensic-IT firm appointment at T-90.

Standing services: technical witness availability for 30 days post-election; IAB reporting for 12 months.

---

## Part XIII — Fraud Prevention: Ballot Stuffing, Ghost Stations, Dead Voters, Voter Fraud

### XIII.1 — Ballot Stuffing

The archetype: votes cast exceed the number of legitimate voters who actually voted, either by adding physical ballots or inflating kit-side counts.

Historical Kenyan precedent: Raila Odinga v IEBC [2017] eKLR — court noted implausible turnout patterns at multiple stations; AfriCOG "Dark Business of Election 2017" chapter on turnout anomalies; multiple polling stations recorded > 100% turnout in preliminary 2013 data (Petition 5 of 2013 context); 2022 ELOG parallel vote tabulation flagged several stations with turnout > 95% before 17:00.

The mechanism: physical extra ballots pre-stuffed in the box before station opening; or biometric override exploited to issue extra ballots; or a corrupted presiding officer entering a higher kit-side count; or post-counting physical ballot substitution.

Failure modes of prior mitigations: 2013 EVID batteries dead at many stations disabled the biometric lock entirely (AfriCOG 2014); 2017 and 2022 kit-side counter logic was not independently verified before deployment.

Preventive control: biometric authentication is the mandatory gate before any ballot is issued. Kit-side ballot counter increments only on successful biometric authentication (hardware interlock). Ballot serial-number allocation is station-bound and quantity-capped at 110% of registered voters. Physical ballot-box tamper seals: sequential numbered seals, sealed by presiding officer + two party agents at station opening, seal numbers registered on kit and uploaded. Ballot-paper stock reconciliation at opening and closing.

Detective control: real-time kit-vs-central turnout comparison (ECC monitors sum of authentications per station every 10 minutes). Statistical outlier detection: any station exceeding 90% turnout before 17:00 triggers ECC alert; any station exceeding 3 standard deviations from constituency mean triggers result hold. Cross-check: sum of Form 34A "valid votes cast" must equal sum of successful biometric authentications. Cross-check: ballot serial numbers at counting must lie within the station-allocated range.

Evidentiary control: every biometric authentication event signed by kit HSM and included in Merkle chain. Every ballot-issue event linked to its authentication event by sequence number. Ballot-box seal photographs at opening and closing, kit-signed EXIF. Ballot-paper serial-number range published per station in Kenya Gazette at T-14.

Residual risk: extremely sophisticated physical attack substituting ballot-box contents entirely after counting, bypassing all RFID seals. Residual: LOW given dual-witness chain and RFID tracking.

| Metric | Target | Measurement | Sign-off |
|---|---|---|---|
| Stations with ballot-count = authentication-count | 100% | Cross-check at aggregation | Returning officer + Datacraft |
| Stations with turnout > 95% before 17:00 flagged and reviewed | 100% | ECC dashboard | IAB |
| Post-count serial-range violations | 0 | Automated at aggregation | Datacraft Technical |

### XIII.2 — Ghost Polling Stations

The archetype: results submitted from polling stations that were not gazetted, not set up, or not operational.

Historical Kenyan precedent: 2013 and 2017 saw allegations of late-added polling stations gazetted after the official list; 2027-preparatory-specs.md notes 3,959 training kits above the 55,393 polling-station kits, creating a pool that could theoretically be activated at unlisted stations; AfriCOG 2018 on constituency boundary disputes.

The mechanism: an extra kit pre-provisioned with a valid station code for a non-existent station submits Form 34A from a back office; or a genuine but non-functional station submits results from a kit that was physically elsewhere.

Failure modes of prior mitigations: 2017 — no GPS geofence enforcement on kit activation; 2013 — no real-time station-activation confirmation at ECC.

Preventive control: every polling station gazetted at T-90 with fixed GPS coordinates, hashed and Merkle-anchored. Kits bound to station code at provisioning. Kit will not activate on election day outside a 500m geofence. Late-added stations (post-T-90) require: unanimous Bid Steering Committee approval + Auditor-General written notification + public Gazette Supplement + no kit provisioned without dry-run geolocation ping from the assigned station.

Detective control: real-time geofence violation alerts to ECC (kit activated outside geofence = immediate alert). Statistical: any station reporting results without a matching biometric-authentication stream from a kit at the correct GPS = automatic result hold. Public Verifier reconciliation: every published result linked to gazetted station code.

Evidentiary control: Kenya Gazette T-90 station list (Merkle-anchored) is the sole authoritative source. Any post-T-90 addition published with signed change-order and photographic evidence. Kit geolocation logged with every action. Public dashboard shows all 55,393 stations with activation status.

Residual risk: station GPS spoofing using a professional GPS spoofer. Residual: VERY LOW given multi-factor controls (RFID + GPS + party-agent presence).

### XIII.3 — Dead Voters

The archetype: deceased persons remain on the voter roll and their identity is used to cast additional votes.

Historical Kenyan precedent: AfriCOG 2018 reported 100+ year-old voter entries; ELOG 2022 flagged persistent deceased-voter concerns; Kenya's Civil Registration Services digital-death-notification integration with IEBC has been intermittent since 2013; OAG 2024 noted ABIS dedup gaps in 2022 roll.

The mechanism: a corrupt presiding officer uses a deceased voter's ID number on roll to issue a ballot under the complementary mechanism; or a sophisticated attacker presents the deceased's ID and exploits a fingerprint-quality-failure fallback.

Failure modes of prior mitigations: CRS-IEBC data-sharing MoU historically has long update cycles (months, not days); family reporting of deaths often lags formal registration by weeks; ABIS dedup in 2022 did not complete fully before election (OAG 2024).

Preventive control: continuous voter register update (CVR) integration with CRS death notifications — automated daily reconciliation. Integration with NRB death-flag data. Family-notification mechanism via *IEBC# short-code. Any voter over 100 years old automatically requires in-person verification with presiding officer + two party agents before kit issues ballot.

Detective control: statistical model: age-mortality-probability curve applied to the roll; any voter over 100 years old with high mortality probability receives additional face-liveness challenge on kit. Any biometric authentication matching a person the kit's on-device CRS delta-sync marks as deceased: kit refuses ballot and generates an s.44A referral with reason code DECEASED.

Evidentiary control: every CRS-IEBC reconciliation cycle logged, signed, and published as a Merkle-anchored dataset. Post-election audit compares "ballots issued" vs "voters confirmed alive on election day" per station; any discrepancy triggers an Election Offences Act s.6 investigation.

Residual risk: deaths occurring in the 7 days before the election may not propagate through CRS. Residual: LOW-MED.

### XIII.4 — Voter Fraud

The archetype covers: (a) impersonation — person A votes as person B; (b) double voting — person A votes twice; (c) cross-station voting — person votes at a station other than their registered station; (d) ineligible-alien voting; (e) minor voting; (f) voter bribery.

Historical Kenyan precedent: impersonation and double-voting concerns raised by ELOG in every cycle since 2013; voter bribery is documented in every Kenyan election cycle (Election Offences Act s.12 prosecutions on record); cross-station voting exploited in 2013 where complementary-mechanism weaknesses allowed manual overrides without biometric verification.

Six overlapping preventive layers:
Layer 1 (dedup at registration): ABIS 1:N dedup with FAR <= 10^-6 at national scale; adjudication queue for borderline matches. Neutralises (d).
Layer 2 (station-binding): voter bound to one station at registration; kit at Station A rejects any voter not on Station A's roll. Neutralises (c) at first pass.
Layer 3 (real-time double-vote detection): every kit synchronises with central ABIS every 60 seconds; second attempt from any station returns ALREADY_VOTED; kit refuses ballot. Neutralises (a), (b).
Layer 4 (biometric-first, ID-second): kit demands live fingerprint match before considering ID; ID without biometric match refused. Neutralises (c) impersonation.
Layer 5 (indelible ink): indelible ink on left thumb applied immediately after ballot issue; cross-agent witness; redundant with Layer 3 but survives network failure. Neutralises (a), (b) as fallback.
Layer 6 (post-election forensic reconciliation): total ballots issued per station <= registered voters; sum of authentications for voter X = 1 exactly; any variance triggers Election Offences Act s.11 investigation.

For voter bribery (f): participation receipt (Part XIV.3) uses an anonymous token, not a vote-content token. Voter cannot prove how they voted. Bribery enforcement is an Election Offences Act matter; KIEMS provides the evidence infrastructure.

| Metric | Target | Measurement | Sign-off |
|---|---|---|---|
| Double-voting attempts detected on election day | 100% pass-through detected | Central ABIS log cross-check | Datacraft + IAB |
| Ballot-issued vs biometric-auth variance per station | 0 | Automated cross-check | Returning officer |
| Layer 3 sync latency on election day | < 60 seconds | Telemetry | ECC |

### XIII.5 — Cross-Cutting: Identity Infrastructure Integration

| Integration | Status (as at bid date) | Current MoU status | Datacraft role | Fraud archetype neutralised | DPA 2019 compliance |
|---|---|---|---|---|---|
| Huduma Namba / NRB | Partially operational; constitutional court cases ongoing | Draft MoU between IEBC and NRB | Operationalise live ID lookup at enrolment | (d) alien voting, (e) minor voting | DPIA required; s.26(c) |
| CRS death registration | Operational but update frequency low | IEBC-CRS MoU exists; last updated 2021 | Automate daily delta sync | Dead voters (XIII.3) | DPIA required; s.26(c) |
| KRA PIN cross-reference | Operational | No current MoU | Negotiate MoU; use for roll-audit | Dead voters (dormant PINs) | DPIA required |
| MoH mortality aggregation | Partial (HIS data) | No current MoU | Negotiate MoU for post-election audit only | Dead voters | DPIA; aggregate data only |
| Immigration Department | Operational for IDs | Limited data-sharing precedent | Negotiate MoU for citizenship flag at enrolment | (d) alien voting | DPIA required |
| NHIF/SHIF membership | Operational | No current MoU | Explore as proxy for alive-status | Dead voters (proxy) | DPIA required |

### XIII.6 — Metrics and KPIs

| Metric | Target | Measurement | Sign-off |
|---|---|---|---|
| Duplicate-registration rate detected pre-election | >= 99.99% of duplicates cleared | ABIS dedup completion certificate | IAB |
| Dead-voter presence in final roll | < 0.05% | CRS reconciliation post-roll-freeze | Datacraft + IEBC |
| Double-voting attempts detected on election day | 100% (zero pass-through) | Central ABIS log | Datacraft Technical |
| Ballot-issued vs biometric-auth variance | 0 per station | Automated at aggregation | Returning officer |
| Turnout implausibility flags cleared before publication | 100% | ECC dashboard | IEBC Chair |
| Ghost-station detection false-negative rate | 0 | GPS log reconciliation | Datacraft Technical |

### XIII.7 — Bid Packaging

Biometric authentication interlock, ballot serial-number allocation, and RFID seal tracking: in base Technical Proposal Volume II.

Real-time ABIS sync and statistical anomaly detection: in base Technical Proposal Volume II.

CRS/NRB/KRA integration MoUs: IEBC-side deliverable, facilitated by Datacraft; flagged as dependencies in Subcontract Agreement.

Fraud-prevention SLA in Subcontract Agreement Schedule B: zero double-voting pass-through; zero ghost-station false negatives; dead-voter roll cleanliness < 0.05%.

---

## Part XIV — One Person, One Vote, Once

Article 38(3) and Article 81 of the Constitution 2010 guarantee the right to vote exactly once per contest. Three guarantees:

- Guarantee 1: no person votes more than once (fraud-negative)
- Guarantee 2: every eligible voter who presents is issued exactly one ballot per contest (rights-positive)
- Guarantee 3: every voter can privately confirm their participation was recorded without revealing content (verifiable-participation, preserving ballot secrecy under Article 138(2))

### XIV.1 — Guarantee 1: Preventing Multiple Voting

Four attack vectors: (a) same person, same station, twice; (b) same person, different station, same day; (c) same person, same station, via impersonation; (d) same person registered twice under two identities.

Six overlapping layers:

Layer 1 — Pre-registration ABIS 1:N dedup: every enrolment triggers a national 1:N biometric search; FAR <= 10^-6; adjudication queue for borderline matches; dedup-completion certificate signed by IAB before roll locks. Neutralises (d).

Layer 2 — Station-bound authentication: voter bound to exactly one station at registration; kit at Station A rejects any voter not on Station A's roll. Neutralises (b) at first pass.

Layer 3 — Real-time double-vote detection: every kit synchronises with central ABIS every 60 seconds; second attempt from any station returns ALREADY_VOTED; kit refuses ballot; event logged with timestamp, kit serial, station code. Neutralises (a) and (b).

Layer 4 — Biometric-first, ID-second: kit demands live fingerprint match before considering ID; ID without biometric match refused; biometric without matching roll entry refused. Neutralises (c).

Layer 5 — Anti-repeat physical marker: indelible ink on left thumb applied immediately after ballot issue; cross-agent witness; redundant with Layer 3 but survives network outage. Neutralises (a), (b) as fallback.

Layer 6 — Post-election forensic reconciliation: total ballots issued per station <= registered voters; sum of authentications for voter X = 1 exactly; any variance triggers Election Offences Act s.11 investigation.

| Metric | Target | Measurement | Sign-off |
|---|---|---|---|
| Double-voting attempts: pass-through rate | 0 | Central ABIS cross-check post-election | Datacraft + IAB |
| Duplicate-registration rate pre-election | < 0.001% in final roll | ABIS dedup certificate | IAB |
| Layer 3 average sync latency on election day | < 60 seconds | ECC telemetry | Datacraft Technical |

### XIV.2 — Guarantee 2: Every Eligible Voter Issued Exactly One Ballot

Five layers:

Layer 1 — Pre-election roll completeness: CVR cycle integration with CRS and NRB; IEBC voter-roll inspection window per Elections Act s.6A/s.8A; any eligible voter not on roll can challenge before cut-off date.

Layer 2 — Kit availability and reliability: primary kit + pre-positioned hot-spare at every station; battery minimum 12 hours (2013 EVID lesson — AfriCOG 2014 report noted 77% of school venues lacked grid power); 20,000 mAh power bank mandatory (PRICING-GUIDANCE.md s.2).

Layer 3 — s.44A Complementary Mechanism (zero-discretion invocation): pre-signed and uploaded at station opening; invocation requires triple signature (presiding officer + 2 party agents) + video capture; ECC notified in real time; no voter turned away after s.44A is invoked.

Layer 4 — Real-time throughput monitoring: ECC tracks votes issued per station per hour; stations falling below expected throughput receive proactive support alert; roving ICT engineer dispatched if throughput drops below 50% of expected rate for > 30 minutes.

Layer 5 — Extended voting hours: Elections Act s.14A authorisation pre-prepared for stations where queue exceeds 5% of registered voters at 17:00; ECC recommendation based on queue data (Part XV.3).

| Metric | Target | Measurement | Sign-off |
|---|---|---|---|
| Stations with primary kit operational at 06:00 | >= 99.9% | Kit activation log | ECC |
| Voters turned away without casting ballot | 0 | s.44A log + grievance channel | IEBC Ops + IAB |
| s.44A invocations with full triple-signature | 100% | Kit log | Returning officer |

### XIV.3 — Guarantee 3: Voter-Verifiable Participation

Design constraints: Article 138(2) requires ballot secrecy. The participation receipt must not reveal the voter's identity or vote content to any third party while allowing the voter to confirm their own participation.

Architecture — Participation Receipt Programme:

Step 1: At ballot issue, kit generates a random 12-character alphanumeric token (participation token). Token is printed on a thermal strip and handed to the voter. Token is committed to a per-station participation Merkle tree (token only, not linked to voter ID in any log the voter cannot control).

Step 2: The voter-ID-to-token mapping is stored encrypted on the kit during election day for reconciliation. At end of day, after Form 34A is signed and transmitted, the mapping is cryptographically deleted from the kit. The deletion event is logged.

Step 3: The participation Merkle tree root for each station is published in the public portal along with the Form 34A. Any party can verify the number of tokens = number of ballots issued.

Step 4: Voter verification portal — voter enters token; portal returns "token present in [Station Code] participation Merkle tree" + inclusion proof; never returns voter identity or vote content.

Voter-linkage protection: the token is random and independent of voter identity. Even IEBC cannot link a token to a voter after the mapping is deleted at end of day.

Anti-coercion measures: the token proves participation but not vote content. A coercer demanding proof of how someone voted cannot get it from the token.

Constitutional safeguards: ODPC DPIA filed; IAB sign-off before deployment; Kenya Law Society review at T-90; 90-day public consultation at T-180.

| Metric | Target | Measurement | Sign-off |
|---|---|---|---|
| Mapping deletion events logged per station | 100% | Kit log + Merkle chain | Datacraft Technical |
| Token-count vs ballots-issued variance | 0 | Aggregation cross-check | Returning officer |
| Voter verification portal availability (T to T+30) | 99.9% | Monitoring | Datacraft Technical |

### XIV.4 — Integration with Fraud Prevention

| Guarantee | Layer | Part XIII mechanism |
|---|---|---|
| G1 (no double vote) | Layer 3 (ABIS sync) | XIII.4 Layer 3 |
| G1 (dedup) | Layer 1 (pre-reg dedup) | XIII.1 preventive + XIII.3 preventive |
| G2 (kit reliability) | Layer 2 (hot-spare + battery) | XIII.1 preventive (physical ballot controls) |
| G2 (s.44A) | Layer 3 (s.44A) | Part VIII A7 |
| G3 (participation receipt) | Token Merkle tree | XIII.1 evidentiary (ballot serial) |

### XIV.5 — Bid Packaging

Three guarantees become three headline commitments in Technical Proposal Volume II Section IV: "The One-Person-One-Vote-Once Guarantee." Each guarantee has named technical controls, metric SLAs, and a named custodian.

Participation Receipt Programme is a signature Datacraft differentiator: no prior Kenyan vendor has offered this. It should appear in the bid cover letter.

Every metric target in XIV.1-XIV.3 becomes a performance SLA in Subcontract Agreement Schedule B.

---

## Part XV — Public Vote-Counted Verification, Attendance Counting, Queue Tracking, Ballot-to-Voter Chain of Custody

### XV.1 — Public Vote-Counted Verification

Three tiers, each independently sufficient:

Tier A — Participation verification: anonymous token from XIV.3. Any voter enters their token and confirms they were counted.

Tier B — Ballot-in-tally verification: voter enters station code in Public Verifier app or web portal. Portal returns: ballots issued, ballots cast, votes per candidate for every contest at that station, the station's Merkle path to Form 34B, the constituency's Merkle path to Form 34C, and the timestamp when the station was included in the national tally. No voter identity is revealed.

Tier C — Independent-reproducibility verification: any observer downloads the reproduce-me bundle (Part XI) and re-computes the full national tally in under 2 hours on a standard laptop.

| Metric | Target |
|---|---|
| Portal availability during 30-day petition window | 99.99% |
| Time from station close to Form 34A publication | <= 30 minutes |
| Time from tally close to reproducibility bundle | <= 4 hours |
| Verification queries on election day + 7 days | > 5 million |
| Reproduce-me bundle executions independently confirmed | > 100 unique parties within 24 hours |

### XV.2 — Attendance Counting (Footfall Independent of Authentication)

Three redundant footfall counters:

Counter 1 — Entry-door presiding officer manual tally: mechanical clicker; cross-agent witness; logged at 30-minute intervals in Form 32B; Form 32B hash-chained to kit. Legally authoritative count. Zero technology dependency.

Counter 2 — Kit-side voter-facing camera with computer-vision people-detection: anonymised headcount only; no face recognition; no images transmitted or stored; DPA 2019 lawful basis via visible signage at station entrance. Count logged locally at 10-minute intervals.

Counter 3 — QR-station queue-app voluntary roster: voter scans QR code on entry (optional, no ID required); aggregated count published in real time on queue dashboard.

Reconciliation logic: Entries >= Authentications >= Ballots-issued >= Ballots-cast. Any station where authentications > entries x 1.02 = automatic result hold pending review. Any station where entries > authentications x 1.5 = systemic-disenfranchisement investigation.

### XV.3 — Real-Time Queue Tracking

The Queue Transparency Programme:

Real-time inputs: attendance counters (XV.2) + kit throughput rate + queue-app voluntary joins + presiding officer manual 30-minute estimate (mandatory field in Form 32B sync).

Derived metrics per station, updated every 5 minutes: current queue length, estimated wait time, estimated clear time, alert flag.

Public artefacts: live queue map of all 55,393 stations colour-coded green/amber/red; per-station queue detail page; opt-in voter SMS alert.

Election Command Centre triggers:

| Wait time | Action |
|---|---|
| > 90 minutes | Dispatch alert to county coordinator |
| > 180 minutes | Deploy roving ICT engineer with spare kit |
| > 240 minutes | Escalate to IEBC Chair + notify all party agents |
| > 300 minutes | Activate Elections Act s.14A extended-voting-hours recommendation |

| Metric | Target |
|---|---|
| Median queue wait during peak (09:00-13:00) | <= 45 minutes |
| Stations with wait > 240 minutes at any point | <= 0.5% |
| Voters denied vote due to queue-at-close | 0 |
| Queue map update latency | < 5 minutes |

### XV.4 — Ballot-to-Voter Chain of Custody

Six elements comprising the complete chain from ballot-paper manufacture to counted vote:

Element 1 — Ballot-issue-to-cast timing: kit records ballot-issue timestamp; ballot box records deposit (via Element 3 RFID read); policy window is 10 minutes; exit checkpoint prevents carry-out. Carry-out attempt = presiding officer + party-agent intervention + event log.

Element 2 — Serialised ballot with per-station serial range: unique serial printed on each ballot; station-bound serial range published in Kenya Gazette Supplement at T-14 (Merkle-anchored). Any ballot with a serial outside the station's range = tampering alert.

Element 3 — Ballot-box smart-lid (passive HF RFID reader): each ballot has a passive RFID chip; ballot-box lid reads each chip on deposit; independent count cross-checked against authentication count in real time. Ballot without RFID chip = count discrepancy alert.

Element 4 — Assistance protocol per Elections Act s.72: presiding officer + assistant of voter's choice + party-agent witness; kit logs assistance event with all three signatures; video capture at booth entry (not booth interior); assisted-voter count published in Form 34A.

Element 5 — Voter identity persistence (indelible ink): indelible ink applied at ballot-issue; ballot-box attendant verifies ink presence at deposit; deposit without fresh visible ink = alert + hold.

Element 6 — Video observation at ballot-box position (opt-in, high-scrutiny stations): livestream to party-agent portal only (not public); recorded and hashed for petition-window review. Deployed only to stations flagged by IAB as high-scrutiny.

| Metric | Target |
|---|---|
| Ballots deposited without matching RFID read | 0 |
| Assisted-voter events matching Elections Act s.72 procedure | 100% |
| Ballot-box smart-lid count vs manual count | >= 99.9% agreement |
| Ballot carry-out events detected | 100% flagged (target 0 actual carry-outs) |

### XV.5 — Cross-Reference to Prior Parts

| Question | Answered in | Part | Depth |
|---|---|---|---|
| Can I verify my vote was counted? | Tier A (token) + Tier B (station result) | XIV.3 + XV.1 | Full |
| Can I re-derive the national tally? | Tier C (reproduce-me bundle) | XI.4 + XV.1 | Full |
| Were all votes counted once? | G1 + G2 | XIV.1 + XIV.2 | Full |
| Were any extra ballots added? | Preventive + detective controls | XIII.1 | Full |
| Was the Form 34A image altered? | Public Verifier hash check + Merkle path | XI.2 B2 + XV.1 Tier B | Full |
| Were any dead voters counted? | ABIS delta + CRS reconciliation | XIII.3 | Full |
| Was the queue used to disenfranchise? | Queue tracking + extended hours | XV.3 | Full |
| Was ballot secrecy preserved? | Art.138(2); participation token design | XIV.3 | Full |

### XV.6 — Bid Packaging

Public Verifier app: flagship pro-bono differentiator, Volume V Undertaking Letter.

Participation Receipt Programme (XIV.3): signature commitment in Technical Proposal Volume II.

Queue Transparency Programme: signature commitment, Volume II Technical Proposal Section V.

Ballot-box smart-lid RFID: 2027-native capability, aligned with RFID requirement in tender. In base BOM.

All metric SLAs into Subcontract Agreement Schedule B.

*This document was prepared by the Datacraft bid team on 2026-08-20. It is internal strategy only and does not represent IEBC policy or any committed Datacraft commercial position until such commitments are made in the formal bid submission.*

---

## Part XVI — Election Environment Integrity: Boundaries, Speech, Disruption, Awareness

### XVI.1 — Gerrymandering and Boundary-Integrity Tracking

**Legal framing:** Constitution 2010 Article 89 assigns delimitation to IEBC every 8-12 years. Katiba Institute v Attorney General [2017] eKLR found procedural delays in the delimitation review were unconstitutional but did not force immediate redraw. Sixth Schedule s.41 required the first review by 2013.

**The Boundary Transparency Programme:**

Open Boundary Data Platform: every current constituency, ward, and county boundary published as machine-readable GeoJSON with population and registered-voter count; continuously updated with each CVR cycle; public API + downloadable dataset; historical snapshots since 2013.

Boundary-Redraw Simulator: public tool where citizens, journalists, and civil society can simulate alternative boundary configurations against Sixth Schedule quota rules (population parity +/-40%). Fairness metrics computed automatically: population parity, compactness index (Polsby-Popper), majority-minority preservation, cracking/packing detection.

Ex-Post Boundary Impact Analysis: after every election, publish per-constituency vote-to-seat ratio, wasted-vote count, and efficiency-gap metric (Stephanopoulos and McGhee 2015, "Partisan Gerrymandering and the Efficiency Gap").

Redistricting audit trail: every proposed boundary change logged with author, timestamp, justification, and before/after fairness metric; public comment portal; hash-chained edit log.

**Metric targets:** boundary data updated within 24 hours of any IEBC action; public simulator queries > 100,000 unique users per year; boundary explorer availability 99.9%.

**Bid packaging:** Boundary Transparency Programme = pro-bono commitment at KES 0.

### XVI.2 — Hate Speech and Voter Suppression Detection

**Legal framing:** Constitution Art. 33(2)(a) excludes hate speech; National Cohesion and Integration Act 2008 s.13 (incitement), s.62 (offences); Elections Offences Act 2016 s.7-s.8; Computer Misuse and Cybercrimes Act 2018 s.22-s.23.

**Historical precedent:** 2007-2008 post-election violence attributed partly to radio hate speech (Kass FM prosecutions; ICC cases against Kenya media); Umati project 2013 (iHub/Ushahidi) monitoring online hate; Oxford Internet Observatory documentation of CIB in Kenya 2017 and 2022; Team Jorge (Guardian/Haaretz 2023) sub-Saharan Africa operations.

**The Speech Integrity Programme:**

Multi-platform monitoring pipeline: public firehose ingestion from Twitter/X, Facebook, TikTok, YouTube, WhatsApp public groups, Telegram public channels, Kenyan news comment sections; multilingual ingestion in English, Swahili, Sheng, Kikuyu, Luo, Kalenjin, Luhya, Kamba, Meru, Kisii, Somali.

Detection models (AI-assisted, human-adjudicated): hate-speech classifier on Kenyan-context corpus; CIB detection (account-creation velocity, engagement patterns, cross-platform amplification); deepfake detection on video/audio; voter-suppression signal detection (false closure claims, false eligibility claims, false security-threat claims).

Rapid-response workflow: HIGH flags to NCIC + DCI + IEBC Communications within 15 minutes; MEDIUM flags within 60 minutes; LOW flags in daily situation report.

Evidence packaging for legal referrals: auto-generated packages for NCIC/DCI referrals (URL, timestamp, hash, screenshot, cross-platform propagation trace) with chain-of-custody preserved for admissibility under NCI Act s.13.

Prebunking network: pre-election explainer content in every Kenyan language addressing top 20 known misinformation narratives; distributed via radio, social media, community-influencer network.

**Metric targets:** HIGH-flag response < 15 minutes; MEDIUM-flag public correction < 4 hours; prebunking reach > 60% of registered voters; deepfake detection accuracy > 95%; false-positive rate on hate-speech classification < 5%.

### XVI.3 — Violent and Social Disruption Detection

**Historical precedent:** 2007-2008 PEV (1,200+ deaths, 600,000+ IDPs); 2013 Tana River and Bungoma incidents; 2017 Nyanza police-civilian clashes including the Baby Pendo case (IPOA Investigations 2017); 2022 Kisumu/Kondele incidents; Chris Msando abduction and murder 28-30 July 2017 (Nation investigations; KHRC).

**The Election Safety Awareness Programme:**

Multi-source incident feed: public (social media pipeline from XVI.2, citizen-reported via short-code + WhatsApp + Public Verifier app); semi-public (KHRC + Amnesty + KEOG/ELOG field reports); public safety (NPS SITREPs via IEBC-KNPS MoU).

Alert cascade: MINOR — aggregated hourly; SIGNIFICANT — 30-minute alert to IEBC Chair + KNPS command + IAB; MAJOR — 10-minute alert + public dashboard update; CATASTROPHIC — immediate alert to IEBC Chair, Attorney General, KNPS Inspector General, AU.

Voter self-protection tools: Public Verifier app "Safety" tab with panic button, nearby-station status, and alternate-station guidance; free SMS "Ni salama?" short-code for any station.

Named threat response (Chris Msando class): Datacraft funds a private security detail for all senior technical staff from T-90 through T+30; pre-arranged safe-house rotation; encrypted comms protocol; deadman-switch on published-if-compromised documentation; IAB with standing authority to invoke national protection under the Witness Protection Act 2006.

**Metric targets:** CATASTROPHIC alert time < 10 minutes; SIGNIFICANT < 30 minutes; deaths from election-related violence = 0 (target); injuries published without suppression.

### XVI.4 — Maximising Public Information Awareness on Election Day

Seven channels:

(1) Public Verifier app: home screen shows voter's station, queue status, ballot receipt, live tally card, safety tab, grievance filing.

(2) SMS/USSD via *IEBC# short-code: in every Kenyan language, free for all networks (Datacraft-funded MoU with Communications Authority).

(3) WhatsApp official channel with verified badge: broadcast + interactive Q&A bot.

(4) Radio (highest reach): pre-negotiated slots at every language-market station, hourly updates, emergency-broadcast agreement.

(5) TV (KBC, NMG, Standard, Citizen, MediaMax, KTN): real-time results dashboard API + graphics package + pre-approved fact-check ticker.

(6) Physical: per-ward informational poster in local language + Braille/large-print versions (Article 54, Article 82 Constitution 2010).

(7) Diaspora: Kenyan embassy digital screens + WhatsApp broadcast to registered diaspora voters; time-zone-aware notifications.

Content architecture: every message published in structured JSON + rendered to every channel via a single content-management pipeline; corrections propagate to all channels within 60 seconds; message provenance chain with hash.

**Metric targets:** voter awareness of polling-station location > 98%; voter awareness of Public Verifier > 80%; Public Verifier active users on election day > 8M; radio-hour reach > 70% of adult Kenyans; fact-check response < 30 minutes; corrections propagation to all channels < 60 seconds.

### XVI.5 — The Election Environment SOC (E-SOC)

All four programmes feed a unified Election Environment SOC operating 24x7 from T-90 through T+30. Twelve workstations: boundary monitor, hate-speech monitor per language cluster, disruption monitor per region, info-comms desk, fact-check liaison, NCIC liaison, KNPS liaison, IEBC liaison, IAB liaison, journalist desk, diaspora desk, technical operations. Reports to Independent Assurance Board and IEBC Chair. All actions logged, hash-chained, and available to the Supreme Court IT Registrar.

### XVI.6 — Bid Packaging

Boundary Transparency Programme: pro-bono at KES 0. Speech Integrity Programme: cost-of-quality in bid base. Disruption Detection Programme: pro-bono with pre-negotiated MoUs. Public Information Programme: cost-of-quality in bid base. E-SOC: hybrid (infrastructure in base price; staffing partially pro-bono).

---

## Part XVII — Automated Vote Counting and Minimising Disqualified Votes

### XVII.1 — Historical Kenyan Reject-Ballot Data

2017 presidential election: approximately 404,000 ballots rejected out of approximately 15.6M cast (approximately 2.5% rejection rate) — cited in Raila Amolo Odinga v IEBC [2017] eKLR petition arguments; AfriCOG 2018 analysis of rejection patterns. The 2022 election showed a substantially lower rejection rate, attributed to improved voter education and KIEMS workflow improvements (cited in Raila Odinga v William Ruto [2022] eKLR Petition E005/2022, but no material petition ground turned on rejection volume).

IEBC published rejection reason codes across cycles: unmarked ballot (the largest category), over-marked ballot, ambiguous mark, torn or defaced ballot, voter identifiable on ballot face, non-standard mark, wrong-contest ballot, unauthenticated ballot.

Regional patterns: rural/first-time/elderly voters have consistently higher rejection rates in IEBC and observer reports; North Eastern region documented with higher rates in 2013 and 2017 field reports (ELOG 2017).

### XVII.2 — Automated Vote Counting Architecture

Design principle: simultaneously fast (publish within statutory deadlines), accurate (no manual recount changes any winner), verifiable (Part XI reproducibility), and resilient (graceful degradation to manual counting without chain-of-custody loss).

**Five-stage counting pipeline:**

Stage 1 — Station-side ballot counting. Physical ballots counted manually by presiding officer + party agents (Article 138 preserves manual count as authoritative). Kit-side digital tally template as arithmetic aid. Presiding officer enters candidate counts on kit; kit runs sanity checks (sum <= ballots issued; per-contest sum = per-contest ballots) and returns errors before signature. Kit produces Form 34A (presidential) and parallel forms for all six contests as PDFs. Form printed at station (thermal printer) for posting on notice board. Digital form (PDF + JSON + kit-captured photograph) HSM-signed and transmitted.

Stage 2 — OCR validation. Kit-captured photograph of paper Form 34A processed through a Kenyan-election-form-trained OCR model. OCR-extracted tallies compared to kit-entered digital tallies. Any discrepancy triggers hold + re-verification + party-agent re-witness. OCR runs on-device AND centrally (redundant).

Stage 3 — Constituency-level aggregation (Form 34B). Automated sum of candidate votes across all stations in constituency. Manual check: returning officer + party agents verify Form 34B before signature. Any station missing flagged with reason code. Form 34B HSM-signed.

Stage 4 — National aggregation (Form 34C). Automated sum of Form 34Bs. Public dashboard updated in near-real-time. Form 34C HSM-signed by IEBC Chair.

Stage 5 — Publication and reproducibility (Part XI). Every form published as PDF + JSON + hash. Merkle root gazetted. Reproducibility bundle published within 4 hours of Form 34C.

**Automation boundary (bright red line):** no automated interpretation of ambiguous ballots (human decision under Elections Act). No automated declaration of winner (statutory function of returning officer). Automation is arithmetic aggregation + form processing + verification — not intent inference or authority delegation.

**Speed targets:** station-close to Form 34A published <= 30 minutes; last Form 34A to Form 34B complete <= 6 hours; last Form 34B to Form 34C <= 2 hours; end-to-end national tally published <= 72 hours.

**Accuracy targets:** zero arithmetic errors in Form 34C; OCR-vs-kit tally discrepancy rate < 0.1% (all flagged); post-tally recount producing different winner = 0.

### XVII.3 — Minimising Disqualified Ballots

Root causes of rejected ballots: unmarked, over-marked, ambiguous, torn/defaced, non-standard mark, voter identifiable, wrong-contest ballot, unauthenticated.

**Seven-element Voter Intent Preservation Programme:**

Element 1 (ballot design): optimised visual design tested with real Kenyan voters; clear boxes with generous spacing; candidate photograph + party symbol; simplified instructions in local language with pictorial guidance; colour-differentiation between simultaneous ballots (presidential yellow, gubernatorial blue, senate green, MP red, women-rep purple, MCA orange).

Element 2 (kit-side pre-mark validation, opt-in): at end of voting, voter can hold ballot to kit camera for a "does this look right?" pre-check. Kit vision model detects unmarked, multiple marks, ambiguous marks, wrong-ballot-type. Voter warned; option to request replacement ballot (Elections Act s.54). Kit processes image on-device and deletes immediately; only "valid/warning" returned. Voter can refuse check.

Element 3 (physical spoilt-ballot recovery): Elections Act s.54 replacement up to 3 times; kit tracks spoilt-ballot events; spoilt ballots physically retained for petition-window audit.

Element 4 (counting-stage intent adjudication, human): ambiguous ballots reviewed by presiding officer + at least 2 party agents; adjudication logged with reason codes; adjudicated ballots photographed.

Element 5 (voter education): multi-channel pre-election campaign on correct ballot marking; interactive Public Verifier tutorial with practice ballot; radio + TV drills in every Kenyan language; sample ballot displayed at every station entrance.

Element 6 (presiding officer training): standardised adjudication training; case-study catalogue of 100+ ambiguous ballots with adjudication reasoning; 30-day pre-election refresher module.

Element 7 (ex-post reconciliation): per-station: rejected + spoilt + valid = ballots issued (must balance); any station with rejection rate > 3 standard deviations from constituency mean flagged; IAB sample audit of adjudicated-ballot images; post-election published rejection analysis by reason code, geography, and contest.

**Metric targets:** rejected ballot rate < 1.0% per contest (2017 was ~2.5%; target 60% improvement); kit-side pre-mark validation opt-in rate: publish observed; IAB adjudication accuracy review > 99% agreement with presiding officer decisions.

### XVII.4 — Cross-Cutting Benefits

Faster tally publication reduces the disinformation window (Part XVI.2). Fewer rejected ballots reduces petition grounds (Part XII). Better ballot design increases voter satisfaction (Part IX). Kit-side validation shortens queues by reducing replacement-ballot requests (Part XV.3). OCR verification provides automatic fraud-detection input (Part XIII).

### XVII.5 — Bid Packaging

Automated aggregation pipeline: in bid base (Volume II Technical Proposal). Kit-side pre-mark validation: pro-bono differentiator (Undertaking Letter). Ballot design study: pro-bono, outputs owned by IEBC. OCR validation: in bid base. Presiding officer training standard: in bid base (training annex). Voter education campaign: cost-shared with IEBC. Rejected-ballot reason-code standard: pro-bono.

---

## Part XVIII — Rules Compliance, Maximising Legality, and Process Awareness

### XVIII.1 — Compliance Landscape

Every actor in the election operates under specific statutory rules. Violations either invalidate the vote, create petition grounds, or expose someone to prosecution.

| Actor | Key statutory sources | Material rules (selected) | Datacraft-enabled enforcement |
|---|---|---|---|
| Voter | Constitution Art. 38; Elections Act s.5-s.15 | Register at correct station; vote once; ballot secrecy | Biometric station-binding; single-vote enforcement; booth privacy |
| Presiding officer | Elections Act s.12, s.14, s.14A, s.72, s.84; Election Offences Act | Open at 06:00; close at 17:00 (or 14A extension); manage assistance per s.72; declare s.44A per protocol | Kit opens only at 06:00; kit refuses close before reconciliation; s.44A escalation ladder |
| Party agent | Elections Act s.30, s.69; Political Parties Act s.16 | Accredited; does not interfere; signs forms | Kit records agent signatures; agent portal read-only access |
| Observer/monitor | Elections Act s.21A; IEBC accreditation regs | Accredited; does not interfere; reports accurately | Observer access to sanitised SOC dashboard |
| Returning officer | Elections Act s.39, s.84 | Aggregates per Form 34B; signs; transmits | Auto-aggregation engine; HSM-signed forms |
| Vendor (Datacraft/Mechrevo/subs) | PPADA 2015; Elections Act; DPA 2019 | No conflict of interest; deliver to spec; protect personal data | CCB log; DPO on PMO; zero-trust; PPRA compliance audit quarterly |
| Candidate / campaign team | Election Campaign Financing Act 2013; Election Offences Act; NCI Act 2008 | Declared spending; no hate speech; no bribery | Campaign spending portal; hate-speech monitoring (XVI.2) |

### XVIII.2 — Making Rules Unavoidable by Default

**Kit-side compliance guardrails:** presiding officer login requires biometric + shift assignment; two-agent quorum (no ballot issuance without at least one party-agent scan-in at station opening); s.44A invocation requires triple signature (presiding officer + 2 party agents + video capture); s.72 assistance requires presiding officer + assistant-of-voter's-choice + party-agent witness; manual override requires all three signatures + reason code + video hash; kit refuses to close before mandatory Form 32 reconciliation completes; kit refuses to close if ballot-count and authentication-count variance exceeds tolerance without a documented explanation.

**Portal-side compliance guardrails:** returning officer login requires 2FA + role-based access + geographic-jurisdiction binding; form aggregation requires all constituent forms present, all signatures verified, and all hash chains valid; publication requires IEBC Chair HSM signature; every action logged and hash-chained.

**Vendor-side compliance guardrails:** every production change requires CCB approval (PPADA 2015); every access to production requires HSM-card + on-shift roster + purpose log; every source-code change hash-committed to code escrow within 24 hours; every deployment reproducible from tagged commit.

**Campaign compliance guardrails:** campaign spending disclosure portal with real-time cross-check against bank filings; media-ad spending monitoring against declared budget; automated compliance reports to IEBC + Registrar of Political Parties.

**Security-personnel compliance guardrails:** kit-side security-officer log-in at station arrival with badge scan; any presence beyond the established KNPS deployment plan flagged in real time.

### XVIII.3 — Real-Time Compliance Verification (the Compliance Dashboard)

The Compliance Operations Centre (COC) runs 24x7 during T-30 to T+30. Displays: per-station compliance status (green/amber/red); per-actor category compliance rate; rule-violation event stream; root-cause analytics; escalation queue.

Public-facing subset: aggregate compliance rate per constituency; trend line over election day; public confidence signal.

**Metric targets:** kit-side guardrail coverage = 100% of statutory rules admitting technical enforcement; real-time violation detection latency < 5 minutes; escalation-to-authority routing < 15 minutes for MAJOR violations; public compliance dashboard update < 5 minutes.

### XVIII.4 — Maximising Legality, Minimising Illegality

Six elements of the Legality Assurance Programme:

Legal-basis-per-action logging: every consequential system action logged with its statutory basis code; IAB can query "show me every action taken under Elections Act s.44A."

Pre-election legal-clearance protocol: every technical procedure with legal-effect exposure reviewed by Attorney General's office + Kenya Law Society + Datacraft external counsel; clearance certificates published in the T-14 evidentiary bundle (Part XII.4).

Election Offences Act 2016 real-time reporting: automated evidence-package generation for detected violations (bribery s.12, undue influence s.7, personation s.6, false statements s.9); routed to ODPP with hash-verified evidence.

Retrospective legality audit: post-election IAB-commissioned independent audit of statutory-compliance rate per station; report published within 30 days; recommendations to Parliament and IEBC.

Anti-corruption compliance (EACC integration): vendor + IEBC official gift-and-hospitality register; whistleblower channel (Whistleblower Protection Act + PPADA s.66) with legal-privilege preservation; conflict-of-interest register.

Data-protection compliance (DPA 2019): ODPC-registered controller/processor/DPO; DPIA published for every material data-processing activity; data-subject rights portal; breach-notification protocol with 72-hour ODPC filing SLA (DPA 2019 s.43).

**Metric targets:** major statutory-rule violation rate per station < 0.5%; Election Offences Act evidence packages routed to ODPP = 100% of detected incidents; DPA 2019 breach-notification SLA = 100% within 72 hours; legal-clearance certificate coverage = 100% of consequential procedures.

### XVIII.5 — Maximising Voter Awareness of the Process

Eight elements:

(1) Voter Journey Explainer: "A day in the life of your vote" from registration through petition window; interactive Public Verifier "Learn" tab; video series in every Kenyan language; print booklet; radio drama series; school curriculum module.

(2) Rights Knowledge Kit: every voter's statutory rights in plain language; what to do if turned away, ballot rejected, kit fails, coerced, or bribed.

(3) Process Milestone Broadcasts: public schedule of every election-cycle milestone; automated notifications to opt-in voters.

(4) Live Process Explainer: Public Verifier explains each stage of election day in real time. Example at station close: "Your station just closed. Here's what happens next: ballot count (30-60 min), Form 34A (15 min), transmission (5 min). Form 34A appears on the portal within 30 minutes."

(5) Post-Election Process Explainer: "The results are announced. Candidates have 7 days to file a petition. Supreme Court hears any petition within 14 days. Judgment within 30 days. Then inauguration."

(6) Faith, Community, and Youth Amplification: MoUs with NCCK, KCCB, SUPKEM, Hindu Council; community-based organisation network; youth-organisation partnerships.

(7) Accessibility (Article 54, 82 Constitution): Braille + large-print + sign-language + audio versions of all materials; distributed via KNAD, KUB, APDK; kit-side audio prompts for blind voters.

(8) Pre-election Voter Simulation: community-based "practice election" events at wards; voters walk through the actual KIEMS UX.

**Metric targets:** voter self-reported process understanding > 80%; voter self-reported rights knowledge > 70%; Public Verifier "Learn" tab visitors > 5M in election cycle; school-curriculum adoption = 100% of public schools; accessibility version distribution = 100% of demand met.

### XVIII.6 — CAOC (Compliance and Awareness Operations Centre)

Extends E-SOC (Part XVI.5) and COC. Unified command centre for compliance + awareness. Reporting cadence: hourly during election day; daily T-30 to T+30; weekly T-90 to T-31; bi-weekly T-180 to T-91; monthly outside election cycle.

### XVIII.7 — Bid Packaging

Compliance Dashboard: cost-of-quality in bid base. Legality Assurance Programme: pro-bono differentiator (Volume V + Undertaking Letter). Process Awareness Programme: cost-shared with IEBC + faith/community/youth partners in-kind. Election Offences Act evidence-package infrastructure: pro-bono. DPA 2019 compliance infrastructure: cost-of-quality (mandatory). School curriculum module: pro-bono in partnership with Ministry of Education. Radio drama series: cost-shared with IEBC + broadcast partners.

---

## Part XIX — Anomaly, Fraud, and Rigging Detection with State-of-the-Art Audit Methods

### XIX.1 — Detection Layers (Defence in Depth)

A rigging attempt that survives one layer must survive all seven. Each layer is independent, uses different signals, and is operated by a different accountability chain.

**Layer 1 — Kit-side self-monitoring (real-time, per-kit)**

Kit telemetry: authentication times, throughput, failure rates, geolocation drift, battery, network state, memory/storage anomalies. Baseline: kit's own hourly baseline plus cross-station peer baseline. Alert triggers: throughput outlier, authentication-time distribution shift, geolocation-drift beyond geofence, unusual sequence of manual overrides, unexpected reboot.

**Layer 2 — Cross-station statistical monitoring (real-time, per-constituency)**

Peer-comparison at constituency level. Multi-dimensional outlier: turnout + authentication-time + rejected-ballot rate + biometric-first-try-success rate + manual-override rate. Model: unsupervised anomaly detection (Isolation Forest, One-Class SVM, or Autoencoder) trained on prior-election baselines. Alert triggers: station in top 0.5% of peer-deviation on any dimension.

**Layer 3 — Statistical distributional analysis (post-close, per-station)**

Last-digit analysis: 2BL test on final digits of station tallies (uniform distribution expected under integrity; concentrations at round digits indicate fabrication per Beber and Scacco 2012, "What the Numbers Say: A Digit-Based Test for Election Fraud Using New Data from Africa," American Journal of Political Science 56(1)).

Turnout-margin scatter: Rozenas (2017) clustering analysis on turnout-margin scatter plots; Klimek et al (2012, "Statistical Detection of Systematic Election Irregularities," PNAS) bivariate histogram method. Alert triggers: stations failing multiple distributional tests.

Caveat: Benford's Law applied to raw candidate vote counts per station is contested (Deckert, Myagkov, Ordeshook 2011 in Electoral Studies). Apply only to derived quantities (ratios, margins) where the mathematical preconditions hold. Publish methodology. Never use in isolation.

**Layer 4 — Cryptographic and reproducibility checks (post-close, whole-tally)**

Merkle-inclusion check for every Form 34A in constituency and national aggregation. Hash-chain continuity check across time. Reproducibility check: re-run aggregation from raw inputs and compare to published output bit-identically. Alert triggers: any hash mismatch, any missing inclusion proof, any reproducibility variance.

**Layer 5 — Risk-limiting audit (see XIX.2)**

Post-election sampling with mathematical guarantee. Alert triggers: audit fails to confirm reported outcome at declared risk limit.

**Layer 6 — Cross-source triangulation (real-time, whole-country)**

Compare kit-transmitted results to: (a) KEOG/ELOG parallel vote tabulation; (b) presiding-officer manual counts uploaded independently; (c) party-agent independent tallies where declared. Alert triggers: any divergence > 2% at any constituency, > 0.5% at national level.

**Layer 7 — Human-adjudicated tip line and observer signals (real-time, all-cycle)**

Grievance channels (Public Verifier + short-code + WhatsApp). Observer field reports (KEOG, ELOG, IFES, Carter Center, EU EOM). Media investigative signals. Whistleblower channel (Whistleblower Protection Act + PPADA s.66). Alert triggers: converging independent reports of the same issue.

### XIX.2 — State-of-the-Art Audit Methods

**A. Risk-Limiting Audits (RLA) — the current gold standard**

RLA (Stark 2008, "Conservative Statistical Post-Election Audits," Journal of the American Statistical Association 103(483)) provides a mathematical guarantee: if the reported winner did not actually win, the audit will find it with probability >= 1 - alpha. Typical alpha = 0.05 (95% confidence). This is qualitatively different from a fixed-percentage audit that offers no such guarantee.

**A.1 — Ballot-polling RLA:** randomly sample individual ballots from physical ballot boxes; check who each sampled ballot voted for; continue until statistical confidence is reached or a full recount is triggered. Requires no ballot-level cast-vote records (CVRs). Best fit for Kenya where counting is manual under Elections Act. Reference: Lindeman and Stark 2012, "A Gentle Introduction to Risk-Limiting Audits," IEEE Security and Privacy.

**A.2 — Comparison RLA:** requires ballot-level CVRs — each physical ballot linked to a scanned interpretation. Much smaller sample sizes than ballot-polling. Fit for Kenya only if kit-side pre-mark validation (XVII.3 Element 2) creates voluntary CVRs.

**A.3 — SHANGRLA (Stark 2020, "SHANGRLA: Sharper Discrepancy Bounds for Risk-Limiting Audits"):** framework that unifies ballot-polling, ballot-comparison, and hybrid audits. Open-source implementations: VotingWorks Arlo; RLA Tool (Colorado Secretary of State). Currently deployed at scale in California, Colorado, Georgia, Michigan.

**Recommended Kenya deployment: Ballot-polling RLA using the SHANGRLA framework.**

Design: after Form 34C is published, sample physical ballots according to a random schedule seeded by a public random beacon (drawn from cryptographic sortition using the published Merkle root — Part XI). Sample size determined by margin of victory and target risk limit (e.g., 5% risk limit + 4% national margin yields approximately 3,000 ballots; 1% margin yields approximately 50,000 ballots). Ballots physically retrieved by KEOG/ELOG/IAB sampling teams under camera + party-agent supervision. Statistical software published open-source with reproducible seed. If audit fails to confirm the reported winner, full recount triggered under Elections Act s.80.

**B. Election Forensics (statistical detection of tampering)**

**B.1 — Digit analysis:** 2BL last-digit uniformity test (Beber and Scacco 2012). Apply to per-station tallies where count is large enough for the distribution to apply. Publish methodology alongside results.

**B.2 — Turnout-vote-share scatter analysis (Rozenas 2017; Klimek et al 2012):** under integrity, turnout and winner's vote share are largely independent across precincts. Under ballot-stuffing: correlation emerges between high turnout and high winner-share. Under vote-swap fraud: horizontal lines/clusters at specific ratios. Apply as advisory, not authoritative.

**B.3 — Precinct-level regression discontinuity:** if cryptographically-sortitioned recount sample is truly random, compare sampled vs non-sampled stations to estimate any systematic effect. Regression discontinuity gives a causal estimate of any remaining fraud after all controls.

**C. Machine-Learning Detection**

Autoencoder anomaly detection: train on prior-election station-level patterns; high reconstruction error = anomaly. Used at Layer 2 for real-time cross-station monitoring. Advantage: unsupervised, no labeled fraud data needed. Caveat: hard to explain in court; use as advisory, not authoritative.

Graph neural network (GNN) for coordinated behaviour: detect correlated anomaly patterns across multiple stations suggesting coordinated tampering. Frontier research — advisory only.

**D. Adversarial / Red Team Audits**

Pre-election adversarial audit (T-90): Datacraft-funded external red team (NCC Group / IOActive / Kudelski Security / Serianu) attempts every attack in Part VIII against the pre-production system. Findings remediated and published in T-14 evidentiary bundle.

Election-day tabletop exercise (T-30): simulated attack scenarios with IEBC + KNPS + NIS + CIRT-CC + Judiciary IT + Datacraft. Response protocols validated.

Post-election adversarial re-audit (T+30): independent second team attempts to find fraud in the published data. Cash bounty for confirmed findings. Public report within 60 days.

**E. Continuous Monitoring Methods**

Publicly-observable canary values: Datacraft publishes known-good canary transactions in the RTS pipeline daily. Any modification of canary = evidence of pipeline tampering. Canaries are cryptographically bound to publication timeline — cannot be retroactively backfilled.

Warrant canary architecture: daily "no compromise has been detected" signed statement from IEBC + Datacraft security teams. Absence = signal.

### XIX.3 — Rigging-Type Taxonomy and Detection Mapping

| Rigging type | Historical Kenya precedent | Detection layer | SOTA method | Public artefact |
|---|---|---|---|---|
| Ballot stuffing | 2013, 2017 turnout implausibility (Part VIII B14) | L2, L3, L5 | Scatter analysis + ballot-polling RLA | Public map + audit report |
| Vote flipping | 2017 forensic claims (The Elephant 2017) | L4, L5 | Ballot-polling RLA | Audit report |
| Ghost stations | Part XIII.2 | L2, L4 | Geofence + Merkle | Public map |
| Voter-roll manipulation | Multi-cycle (Part VIII B15) | L6 | Cross-source | Public dataset |
| Result-form fabrication | 2013 RTS collapse (Part VIII B8, B9) | L3, L4 | Digit analysis + Merkle | Public forensics |
| Transmission tampering | 2017 log dispute (Part VIII B2) | L4, L6 | Hash chain + replay | Public logs |
| Insider database write | 2017 "Chiloba server" (Part VIII C11) | L4, L7 | Immutable log + tips | Public logs |
| Coordinated multi-station | Rare but plausible | L2, L3, L5 | GNN + RLA | Advisory + audit |

### XIX.4 — Audit Governance

IAB commissions all audits — not IEBC.

Rotating cohort: Kenyan universities (UoN, Strathmore, JKUAT, USIU, Multimedia University statistics and computer-science departments); international peer-review (IFES, Carter Center, ELOG-affiliated statisticians); private-sector forensic auditors (KPMG DFI, PwC Advisory, Deloitte Forensic); cryptographic auditors (Kudelski Security, NCC Group, IOActive).

Sample selection: cryptographic sortition seeded by a public random beacon. No human picks the sample.

Adjudication: presiding officer + party agents + observer + IAB representative + auditor.

Publication: every audit finding published within 14 days of completion.

### XIX.5 — Metric Targets

| Metric | Target | Measurement | Sign-off |
|---|---|---|---|
| RLA sample-to-outcome-confirmation | 100% confirmed at 5% risk limit | Statistical software output | IAB |
| Layer 2 anomaly detection latency | < 5 minutes | Timestamp difference | Election Command Centre |
| Distributional-analysis stations flagged | Publish observed | Software output | IAB |
| Cross-source divergence at national level | 0 | Cross-source dashboard | IEBC Chair + IAB |
| Whistleblower tips substantiated | Publish observed | Adjudicator ruling | IAB |
| Bug-bounty findings resolved within SLA | 100% | Bug-tracker + resolution log | Datacraft CISO |
| Pre-election red-team findings remediated | 100% before T-14 | Remediation register | IEBC Chair |

### XIX.6 — Bid Packaging

Multi-layer detection stack: cost-of-quality in bid base (Technical Proposal Vol. II, Section 12 "Detection and Audit Architecture"). Risk-limiting audit programme: pro-bono differentiator (Volume V / Undertaking Letter). Election forensics dashboard: pro-bono. Bug-bounty programme: pro-bono (Datacraft funds the pool). Pre-election adversarial audit: cost-of-quality. University audit partnerships: pro-bono facilitation. SHANGRLA software deployment: in bid base (open-source, low marginal cost). Cryptographic canary and warrant canary: pro-bono.

### XIX.7 — Why This Beats Every Prior Kenyan Election

2013: no formal audit method — reject-then-recount was ad-hoc. No statistical guarantee.

2017: physical scrutiny orders issued but not complied with (Raila v IEBC [2017] eKLR para 272). The Supreme Court found IEBC's response inadequate. No RLA. No reproducibility bundle.

2022: improved dashboards and data publication but no risk-limiting audit, no cross-vendor verification, no formal forensic analysis published pre-petition.

2027 with RLA + Merkle-reproducibility + seven-layer detection + pre-registered technical witness + IAB oversight: mathematically guaranteed integrity, publicly reproducible within 4 hours of tally close, court-defensible before the petition is filed, and independently verifiable by any citizen with a laptop. No prior Kenyan election has come close.

---


---

## Part XX — Minimising the Cash-to-Votes Conversion Ratio

### XX.1 — Historical Kenyan Vote-Buying Landscape

Vote buying in Kenya is not a marginal phenomenon. It is a structured, highly organised component of campaign strategy. Kramon (2016, "Where is Vote Buying Effective?", Comparative Political Studies 49(2)) documents that Kenyan politicians use cash, food, and household goods as vote-buying instruments, particularly effective in low-information, high-poverty constituencies where the credible delivery of public goods is implausible. D'Arcy and Cornell (2016) identify Kenya's "handshake" politics as an instance of the broader African patrimonial-exchange pattern where electoral competition is mediated through gift networks rather than policy competition.

Kenyan campaign colloquial vocabulary reflects the scale of the practice: "kitu kidogo" (small thing), "mkate wa kwanza" (first bread / down payment), "chai" (tea money), "handshake", "envelope." These terms appear routinely in Nation, Standard, and People Daily investigations of every election cycle from 2007 onward.

KEPSA and KEWOPA analyses (2022 cycle) estimated total candidate + party informal cash movements at several hundred billion shillings, though the figure is necessarily imprecise given the informality of the transactions. Election Campaign Financing Act 2013 (ECFA) disclosure filings have historically shown low compliance: IEBC 2017 and 2022 reports confirmed that fewer than 40% of candidates filed complete returns and enforcement actions were negligible.

Court enforcement: Election Offences Act 2016 s.12 criminalises bribery with a penalty of KES 1 million or 3 years imprisonment, or both. Prosecutions have been sporadic and convictions rare, partly because the evidence chain — an individual voter testifying they received cash in exchange for a vote — is almost impossible to produce in practice (voters have no incentive to testify, and the exchange is inherently private).

The academic consensus: vote buying is effective in Kenya when voters lack information about candidates and when ballot secrecy is imperfect (Kramon 2016; Vicente 2014 on Mozambique; Fujiwara and Wantchekon 2013 on Benin). The prescription is structural, not penal: strengthen ballot secrecy, raise voter information, and make verification of vote choice impossible.

### XX.2 — Structural Defences (Ballot Secrecy is the Ultimate Anti-Vote-Buying Tool)

The economic logic of vote buying requires enforcement: a buyer pays only if they can verify compliance (Kramon 2016). Every mechanism that makes verification impossible reduces willingness to pay. Ballot secrecy is therefore the deepest structural defence.

**Design elements extending Parts XIV and XV:**

True ballot secrecy: the voter cannot prove how they voted even if they want to. The Part XIV.3 Participation Receipt Programme confirms participation, not choice. The receipt token is random and unlinked to vote content. A buyer who demands the token gets no information about vote choice.

Voting-booth physical design: three-sided screen, hand-height, back-to-observers. No sightline from the queue or from party agents. Enforced at every station. Booth-construction specifications part of KEBS PVoC assessment.

No smartphones in booth: signage at booth entry + poll-worker enforcement + Elections Act procedural rule. A smartphone photograph of a marked ballot is the vote-buyer's verification instrument. Removing photographic possibility eliminates the cheapest verification method.

No ballot-marking exhibition: voter cannot show ballot to anyone before deposit. Presiding officer instruction + party-agent awareness briefing.

Fast throughput: voter enters, marks, exits. Queuing design prevents lingering. High-speed throughput eliminates the time window for photograph or verbal disclosure.

**Voter empowerment message:** the most-leveraged single intervention, ethically complex, delivered exclusively through trusted third parties (faith councils, civil society, youth organisations — not IEBC or Datacraft): "Take the money. Vote your conscience. Nobody can find out." This message activates ballot secrecy as the individual voter's sovereign power. Kramon (2016) notes this message — when credible — reduces buyer willingness to pay substantially.

### XX.3 — Financial Traceability (Raising the Risk to Buyers)

**The Campaign Finance Transparency Programme:**

**Element 1 — Real-time campaign spending dashboard.** Every candidate and party required under ECFA 2013 to file weekly bank statements, M-Pesa Paybill activity logs, and expense receipts during campaign period. Datacraft builds and IEBC hosts a public dashboard aggregating disclosed spending per candidate vs ECFA 2013 s.22 spending ceilings. Any candidate above ceiling flagged in real time for IEBC referral and ODPP notification.

**Element 2 — Mobile-money pattern detection with regulatory partners.** Datacraft cannot access individual M-Pesa or Airtel Money transactions (privacy law + banking secrecy + DPA 2019). But the Central Bank of Kenya and Financial Reporting Centre (FRC) operate anti-money-laundering monitoring under the Proceeds of Crime and Anti-Money Laundering Act (POCAMLA 2009). Pre-agreed election-period Suspicious Activity Report thresholds for bulk cash-out patterns near polling stations. On election day: geo-clustered cash-out spikes in proximity to polling stations flagged to FRC and DCI in real time. Public dashboard shows aggregated constituency-level unusual-cash-movement heat map without individual transaction data.

**Element 3 — Bank and Sacco disclosure integration.** Kenya Banking Act and Sacco Societies Act reporting obligations aggregated at constituency level for the election-cycle window. Any bank-account activity by candidates or party officials cross-referenced against declared campaign filings. Discrepancies flagged for IEBC and ODPP.

**Element 4 — Foreign-money detection.** ECFA 2013 ss.32-34 prohibit foreign campaign funding. FRC and Central Bank cross-border transaction pattern analysis for election-period inflows to candidate-adjacent accounts. Immigration and KRA cross-references for large-cash-carrying entrant declarations.

**Element 5 — Whistleblower and tip line.** Election Offences Act s.12 bribery reports via short-code, WhatsApp, and Public Verifier. Whistleblower Protection Act coverage for reporters. Cash and material bribery incident capture with photo, geolocation, and witness statement. Verified reports referred to DCI and ODPP within 6 hours.

**Metric targets:**
- ECFA disclosure filing compliance rate: > 95% of registered candidates (baseline < 40% in prior cycles); measurement via IEBC registry; sign-off IEBC + ODPP.
- Ceiling-breach flags routed to IEBC and ODPP: 100% of detected; measurement via compliance dashboard log; sign-off IEBC.
- SAR-flag response time (FRC + DCI): < 30 minutes for major triggers; measurement via FRC incident log; sign-off FRC.
- Election Offences s.12 prosecutions per cycle: publish observed; measurement via ODPP registry.

### XX.4 — Behavioural Interventions (Making Voters Harder to Buy)

Vicente (2014) demonstrates that voter information interventions reduce vote-selling in contexts similar to Kenya. Fujiwara and Wantchekon (2013) find that deliberative public forums reduce vote-buying incidence. Kramon (2016) isolates ballot-secrecy confidence as the largest single variable in a voter's decision to accept-and-defect. These findings guide programme design.

**Intervention 1 — Voter information campaign.** Pre-election candidate-comparison portal: policy positions, voting record (incumbents), financial disclosure summary, past-conduct facts. Multi-language, multi-channel. Peer-reviewed content standard with civil-society fact-check partner sign-off. Metric: reach > 70% of registered voters.

**Intervention 2 — Anti-vote-buying moral messaging via trusted intermediaries.** Faith-council pulpit messaging on moral cost of vote-selling delivered by NCCK, KCCB, SUPKEM, and Hindu Council. Chief and elder community-forum messaging in cultural context. "Kula lako, piga kura yako" campaign ("Eat their food, vote your choice") — culturally-legitimate framing that empowers rather than shames, adopted from documented effective interventions in comparable contexts. Youth-organisation peer-messaging targeted at first-time voters. Metric: > 80% of active congregations; youth network coverage > 50% of youth organisations.

**Intervention 3 — Ballot-secrecy reassurance.** Explicit voter education: "There is no way anyone will know how you voted. Not the party agent. Not the presiding officer. Not the buyer. Not IEBC. Not Datacraft." Interactive demonstration in Public Verifier Learn tab showing how secrecy works technically. Media coverage of the security architecture anchoring the claim credibly. Metric: voter self-reported understanding of ballot secrecy > 80%.

**Intervention 4 — Competitiveness framing.** Real-time polling and forecast transparency from independently produced polling (academic consortium + IEBC-approved pollsters). Message: "This election is close. Your vote could decide it." Care taken to deliver as neutral information, not candidate-preference signalling. Metric: pre-election polling report reach > 60% of registered voters.

**Intervention 5 — Ex-post accountability (long-term deterrence).** Public registry of Election Offences Act s.12 prosecution outcomes, updated as judgments are finalised. Media partnership for long-form investigative accountability pieces. Naming of convicted bribers once judgment is final. Metric: registry updated within 14 days of every judgment; media placements > 3 major investigations per cycle.

**Intervention 6 — Youth-focused programming.** First-time voters are the most malleable segment and the largest cohort in the 2027 electorate. School and college programming, youth-organisation targeted content. Digital-native formats: TikTok, WhatsApp Status, Snapchat. Kenyan youth influencer partnership, vetted for independence. Metric: first-time-voter engagement > 3M on digital platforms.

**Pre-election survey benchmark:** Afrobarometer question "Would you sell your vote?" (Afrobarometer 2022, Kenya Round 9 baseline) establishes the pre-intervention prevalence. Target: 10-percentage-point reduction among surveyed voters post-election.

### XX.5 — Post-Election Analytics (Measuring the Conversion Ratio)

The cash-to-votes conversion ratio is measurable at an aggregate level using a combination of disclosed spending (XX.3 Element 1), ex-post FRC aggregate cash-movement data, independent KEPSA/KEWOPA spending estimates, and published tallies with demographic and spatial data.

Methodology: per-candidate "cost per additional vote" estimated via regression discontinuity (comparing spending to vote-share at candidate spending ceiling), instrumental variables (using regulation-change years as instruments), and difference-in-differences (2013, 2017, 2022, 2027). Commissioned as pro-bono research with a Kenyan academic institution (UoN Economics, Strathmore Economics, or KIPPRA partnership).

Published annually as a public report with trend line across cycles. Downward trend in estimated cash-per-vote = programme working. This is a durable contribution to Kenyan election science independent of the 2027 cycle.

### XX.6 — Cross-Cutting Integration

| Programme element | Cross-reference |
|---|---|
| Ballot-secrecy structural design | Parts XIV (voting guarantees), XV (chain of custody) |
| Financial traceability | Part XVIII (compliance), XVI (cash-flow precedes coordinated disruption) |
| Behavioural interventions | Part XVI.4 (public information), IX (public confidence) |
| Whistleblower tip line | Part XIX.7 (human-adjudicated tip line, Layer 7) |
| Post-election analytics | Part XI (reproducibility), XIX (audit framework) |

### XX.7 — Bid Packaging

Structural ballot-secrecy defences: cost-of-quality in bid base (baseline expectation meeting Elections Act s.72 and Constitution Art. 38 secrecy obligation).

Campaign Finance Transparency Programme: pro-bono differentiator (Volume V + Undertaking Letter + post-award MoU with CBK and FRC). Requires IEBC-facilitated regulatory MoU — post-award milestone.

Behavioural intervention campaign: cost-shared with IEBC, faith/community/youth partners in-kind. Donor co-funding (UNDP/USAID elections-support window) a likely contribution.

Whistleblower and tip line: pro-bono (infrastructure already costed in Part IX).

Post-election analytics report: pro-bono; IEBC-owned outputs; published research; Datacraft named as contributor.

Prosecution registry: pro-bono.

### XX.8 — Honest Limits

Vote buying will not disappear in one cycle. It has been part of Kenyan political economy for decades and is deeply embedded in candidate-constituency relationships. Datacraft's programme shifts the risk-reward ratio for buyers, empowers voters to accept-and-defect, reduces the conversion ratio at the margin, and provides the country with a measurement framework so progress is visible. The election-outcome-integrity guarantees (Parts XI-XV) are unaffected by vote buying: votes cast are votes cast, regardless of the consideration that preceded them. The structural question for democratic health — whether votes reflect genuine preference — is a 30-year project, not a single-cycle technology fix.

---


## Part XXI — Honest Gap-Closure Register (24 Items)

This part responds directly to the Gap Register dated 2026-08-20. Each item below mirrors the register's numbering and category structure. Where a gap is outside Datacraft's contractual scope, that is stated explicitly. Residual risk is never zero; it is always quantified honestly.

---

### XXI.A — Attack Vectors (Items 1-14)

**Item 1 — Party Primaries**

**Threat:** Kenya's party primaries (Political Parties Act 2011 s.22) are the single most fraud-dense event in the electoral cycle, occurring 3-6 months before the general election. 2017 primary chaos (nominations announced days before, candidate lists published then rescinded, IEBC-supplied voter registers used improperly by parties, widespread cash-for-nomination) generated legitimacy attacks that surfaced during the general election petition. Because primaries occur outside IEBC's mandate and are party-run, no KIEMS tender can directly fix them — but primary disorder poisons the general-election legitimacy pool.

**Precedent:** 2017 primary season: multiple parties (Jubilee, NASA) held simultaneous primaries with inadequate infrastructure. Court of Appeal in Gatirau Peter Munya v Dickson Mwenda Githinji [2014] eKLR held that party nomination disputes are justiciable but narrowly. Political Parties Disputes Tribunal (PPDT) overloaded in 2017 with >2,000 cases.

**Datacraft response:** Offer a Party Primary Support Programme as pro-bono capacity contribution. This involves: KIEMS-lite kit configuration (biometric verification against party membership rolls, result-capture tablet, constituency dashboard) available to any registered party on request. Parties supply their own rolls; Datacraft supplies the hardware and integrity dashboard. Chain-of-custody and result-transmission protocols identical to general-election standard. This is explicitly voluntary, outside the IEBC tender scope, and offered under a separate pro-bono MoU between Datacraft and each requesting party, countersigned by IEBC as observer. It does not give Datacraft access to party membership data beyond what is provided per session. Data destroyed post-primary per DPA 2019.

**Where it lands:** Pro-bono / separate party MoU. Not in bid base. Mentioned in Undertaking Letter as available capacity.

**Residual risk:** Parties may refuse. Low uptake likely in 2027 cycle. The programme establishes precedent for 2032. Primary disorder will continue to generate downstream legitimacy attacks regardless.

---

**Item 2 — Diaspora Voting**

**Threat:** Article 82(1)(e) of the Constitution grants Parliament power to extend voting rights to Kenyan citizens outside Kenya. IEBC has made incremental progress; diaspora voting in 2022 was limited in geographic scope and volume (approximately 4,000 diaspora voters registered for 2022). Embassy-hosted KIEMS kits introduce distinct chain-of-custody and transmission risks: delayed-batch transmission, embassy staff operating as ad hoc presiding officers, physical-ballot cross-border logistics, and no real-time Form 34A publication without reliable embassy connectivity.

**Precedent:** Nyoro & 2 Others v IEBC [2017] eKLR petition challenged diaspora-roll exclusion. IEBC 2022 diaspora-registration expansion was judicially mandated in Katiba Institute v IEBC [2020] eKLR.

**Datacraft response:** Distinct diaspora architecture module. Embassy-hosted kits: same hardware, configured in diaspora mode (voter roll restricted to embassy-registered Kenyans, result-capture in sealed-transmission queue). Transmission: AES-256 encrypted batch uploaded at end of polling day per embassy, transmitted via embassy diplomatic network or commercial VPN with certificate pinning, queued if connectivity drops and auto-transmitted on restore. Physical ballot fallback: sealed ballot boxes, air-freighted to Nairobi with IEBC observer escort, chain-of-custody identical to Part XV. Form 34A published per embassy immediately upon transmission. IAB receives embassy-specific transmission confirmation. Cross-border result reconciliation built into the national aggregation pipeline (Part XVII). Architecture covers approximately 40 diplomatic missions; post-award specification exercise determines final kit allocation per mission.

**Where it lands:** Bid base (priced per embassy station). Architecture detailed in Technical Proposal Vol. II. Requires post-award IEBC-Foreign Affairs ministry coordination for diplomatic clearances.

**Residual risk:** Embassy connectivity is outside Datacraft's control. Physical-ballot chain of custody crosses multiple jurisdictions with varying customs regimes. Diaspora volume is small relative to total; impact on national result is bounded.

---

**Item 3 — Special-Voter Categories**

**Threat:** Several voter categories require workflows distinct from the standard polling-station model. Each creates an attack surface if undesigned. Constitution Art. 83 establishes the right to vote; the Elections Act s.5 extends voting rights to KDF, NPS, and other disciplined services; the Refugees Act 2021 creates a legal framework for refugee rights that does not explicitly include voting but may be the subject of future litigation.

Categories:
- Prisoners on remand (not convicted): Art. 83 right preserved; approximately 12,000 estimated eligible.
- Hospital/infirmary voters: Constitution Art. 38; itinerant ballot-paper system.
- KDF/GSU/Prison Service voters: Elections Act s.5; typically vote at barracks or designated stations.
- Refugees: currently excluded but subject to ongoing constitutional challenge.
- IDPs: internal displacement from 2007, 2017, 2022 post-election violence and Mount Elgon/Tana Delta conflicts; address-linked voter registration creates access barriers.

**Precedent:** Kituo cha Sheria v AG [2013] eKLR confirmed constitutional right of prisoners on remand to vote. Implementation has been inconsistent across cycles.

**Datacraft response:** Kit-side workflow module per category. Remand-prisoner stations: prison governor serves as presiding officer, IEBC-assigned deputy, reduced party-agent complement (one per party), kit locked to prison-registered roll subset, result transmitted via secure PRISONS network relay. Hospital voter: mobile KIEMS unit (tablet + portable fingerprint scanner + battery pack), checklist-based consent + witness architecture per Elections Act regulations, attending clinician as impartial witness. KDF/GSU: barracks kits integrated into standard network but administered by IEBC-seconded returning officer (not military chain of command), civilian-controlled result. IDPs: IDP register compiled from UNHCR/NDOC/Red Cross displacement data cross-referenced with CRS; pre-election IDP verification window at designated IEBC centres; special-station allocation per IDP concentration. Refugee category: monitored; if courts expand eligibility before 2027, Datacraft's architecture accommodates by activating refugee-roll subset (already built, not activated pending legal clarity).

**Where it lands:** Bid base (special-category workflows priced as line items). Legal-basis footnote in Technical Proposal. Post-award legal monitoring for refugee eligibility.

**Residual risk:** KDF voter secrecy is structurally imperfect when barracks commanding officer is nearby regardless of procedural controls. IDP roll accuracy depends on third-party displacement data quality.

---

**Item 4 — Ballot Printing Supply Chain**

**Threat:** Al Ghurair Printing and Publishing (UAE) is Kenya's established ballot printer. Chris Msando was murdered on 31 July 2017, two weeks before the general election, in circumstances that have never been fully resolved. Whether or not the murder was directly connected to the printing process, the Msando incident demonstrated that individuals with knowledge of election-critical logistics become high-value targets. The printer-to-polling-station leg involves: print plant (Dubai), air freight (Jomo Kenyatta International Airport), bonded warehouse (Nairobi), constituency distribution, station delivery. Each handoff is an interception, substitution, or counting-discrepancy risk.

**Precedent:** OAG 2024 report finding 200 missing KIEMS kits and 3,433 faulty kits directly implicates the logistics chain. Any equivalent ballot shortfall at a station voids that station's results and creates a petition ground.

**Datacraft response:** RFID tracking extended to ballot-paper consignments. Passive HF RFID tags embedded in each sealed ballot-paper bundle at point of print (Al Ghurair facility), scanned at: airport departure, JKIA arrival, Nairobi bonded warehouse intake, constituency distribution vehicle loading, constituency store, station delivery receipt. All scan events time-stamped and appended to the HSM-signed audit log. Mismatch between printed-quantity manifest and delivered-quantity at any stage triggers automatic alert to IEBC head of operations and IAB. Multi-country tender-splitting: ballots for at least two geographic regions printed by a second accredited printer (nominated post-award) to eliminate single-supplier catastrophic failure. Datacraft does not control printer selection (IEBC tender separately) but specifies RFID-tag compatibility requirement in the technical interface document.

**Where it lands:** RFID logistics extension in bid base. Multi-printer recommendation in Technical Proposal as IEBC procurement advice (not Datacraft-procured). Escrow of print manifest against delivered manifest in IAB audit bundle.

**Residual risk:** RFID tags can be cloned or removed. Multi-printer adds coordination complexity. Al Ghurair relationship is IEBC-controlled; Datacraft cannot force substitution.

---

**Item 5 — Attacks on Datacraft the Vendor**

**Threat:** Datacraft is not an abstract system: it is a company with engineers, a board, a bank account, and dependencies. A sophisticated adversary who cannot attack the KIEMS technology may instead attack the vendor. Threat vectors: hostile acquisition mid-cycle (new owners install a backdoor); bribery of a named Datacraft engineer to inject malicious firmware before escrow deposit; key-person poaching (PMO director + lead architect depart simultaneously); deliberate insolvency triggering (payment withheld to force liquidity crisis, enabling forced restructuring under new control); reputational attack to force IEBC to terminate the contract.

**Precedent:** Smartmatic's KES 570M debt dispute with IEBC (2022-2025) shows how payment disputes create vendor vulnerability. OT-Morpho's one-day tender (31 March 2017) shows how a rushed onboarding creates insider-threat pressure. Bribery of election-tech engineers is documented in Venezuela (Smartmatic 2004 audit controversy) and Philippines (Smartmatic 2016 source-code controversy).

**Datacraft response:**

*Change of control:* Subcontract Agreement clause (Clause 8.2 draft): any change of >25% of Datacraft equity or effective control triggers mandatory IEBC notification within 48 hours and a 30-day IEBC right to terminate without penalty. This is standard in critical-infrastructure contracts.

*Insider-threat programme:* All engineers with commit rights to the KIEMS codebase are background-checked (DCI clearance, credit check, foreign-contact disclosure), re-checked annually. No single developer can ship firmware unilaterally: dual-committer requirement enforced by GitHub protected-branch policy (two named reviewers with distinct HSM keys required to sign any release commit). Code-review log is part of the escrow deposit, allowing post-incident reconstruction.

*Key-person insurance + successor covenant:* PMO Director and Lead Architect named in the Subcontract Agreement. Key-person life and disability insurance (minimum KES 100M each) naming IEBC as loss payee for business interruption. Named successors (pre-agreed second-tier leads) activate automatically if a named key person is incapacitated, departs, or is unreachable for >72 hours during operational period.

*Source-code escrow with named agent:* NCC Group (Nairobi office or equivalent) holds the escrow. Release conditions include Datacraft insolvency, voluntary liquidation, or IEBC-triggered release (with 14-day cure notice). Escrow deposit updated with each firmware release, within 72 hours of release.

*Financial resilience:* Datacraft maintains a 6-month operational cash reserve in a Kenya-domiciled account designated exclusively for IEBC programme operations, ringfenced by board resolution and disclosed to IEBC CFO quarterly. This prevents a payment dispute from cascading into operational failure.

*Kenyan-shareholder structure:* A majority-Kenyan-shareholder SPV is the contracting entity for the IEBC engagement, limiting hostile-acquisition surface from foreign purchasers and satisfying local content requirements under the Public Procurement Act.

**Where it lands:** Subcontract Agreement clauses (Clause 8). Key-person insurance in Commercial Proposal. Escrow terms in Undertaking Letter.

**Residual risk:** A highly resourced state actor could acquire the SPV through a nominee Kenyan shareholder. Insider threat is bounded but not eliminated by dual-committer controls: collusion between two reviewers is possible but detectable post-incident via audit log.

---

**Item 6 — Attacks on Mechrevo the OEM**

**Threat:** Mechrevo is a PRC-domiciled hardware OEM. The Chinese state could, under the National Intelligence Law (2017) and Data Security Law (2021), compel Mechrevo to cooperate with intelligence or supply-chain interference mid-cycle. US export-control actions (EAR/ITAR) against Mechrevo's parent or components suppliers could prevent shipment. US secondary sanctions on iSoftStone (a related entity) could create OFAC compliance tripwires for the Kenyan contracting structure. A forced supplier substitution mid-cycle, with no pre-qualified alternative, leaves IEBC holding 45,000 kits of legacy hardware and a new vendor with zero run-time in Kenya.

**Precedent:** US sanctions against Huawei (2019-present) effectively ejected Huawei from multiple African government infrastructure tenders within 18 months. US BIS Entity List additions have disrupted supply chains in less than 30 days in documented cases. Kenya is not in the ITAR-controlled zone but US secondary sanctions apply to USD-denominated transactions anywhere.

**Datacraft response:**

*Contingent supplier arrangement:* A second OEM (hardware-specification compatible, under NDA, pre-screened by IEBC) is identified during post-award qualification. The second OEM can produce compliant kits to Datacraft's hardware specification document within 90 days of activation. The specification is hardware-abstracted (OS and firmware run on UEFI-standard x86-64 or ARM-64 with TPM 2.0, fingerprint + iris sensor via USB/PCIe, GPS module, LTE modem): this allows component substitution without firmware rewrite.

*Inventory buffer:* 10% spare kit tranche above contracted quantity, held in bonded warehouse at Mombasa from T-180. If Mechrevo supply is disrupted after T-180, the buffer covers election day without contingent-supplier activation.

*US sanctions monitoring:* Datacraft legal team subscribes to OFAC SDN List daily diff. Any addition affecting Mechrevo or its tier-1 suppliers triggers escalation to IEBC procurement committee within 24 hours and activation of the contingent-supplier protocol.

*KYC/AML tripwire:* Teaming Agreement Clause 2.9: Mechrevo represents and warrants quarterly that no sanctions event affecting its ability to perform has occurred. False representation triggers termination for cause without penalty to Datacraft.

*Steering Committee escalation:* Any Mechrevo supply-risk event is a standing agenda item on the weekly Steering Committee (PM-02 from Part II). IEBC has 30-day cure window before it has to trigger contingent supplier.

**Where it lands:** Teaming Agreement clauses. Contingent supplier arrangement in Commercial Proposal Vol. III (priced as optional activation fee). Buffer inventory costed in pricing base.

**Residual risk:** If both Mechrevo and the contingent supplier are simultaneously disrupted (e.g., broad EAR restriction on PRC-origin electronics), no hardware pathway exists within election-day timelines. This is a macro-geopolitical risk outside any vendor's ability to resolve.

---

**Item 7 — Attacks on the Public Verifier App**

**Threat:** The Public Verifier app (PB-02) is the public's primary interface for independent verification. A spoofed clone on the Google Play Store or Apple App Store could present falsified tally data to millions of voters, or harvest participation-token queries to build a voter-behaviour database. A MITM attack on the hash-verification API could return a falsified hash to a user verifying the audit bundle. A poisoned reproducibility bundle (legitimate download page compromised, different zip served) could fool security researchers into validating a doctored tally.

**Precedent:** Multiple documented clone-app spoofing incidents in African elections: WhatsApp-based election-result impersonation in Ghana 2020 (AFP Fact Check, December 2020). Kenyan 2022 results portal was briefly offline due to DDoS, driving traffic to unofficial aggregation sites.

**Datacraft response:**

*App store code-signing enforcement:* Single canonical signing key per platform (Android: Google Play signing; iOS: Apple Developer Program). Key held in FIPS 140-3 Level 3 HSM, offline backup with Shamir's Secret Sharing (3-of-5 Datacraft board members). Any release requires dual-board sign-off + HSM-generated code-signing credential.

*In-app binary integrity check:* On first launch and every 24 hours, the app computes SHA-256 of its own APK/IPA binary and compares against the published hash on the IEBC website (fetched via DNS-over-HTTPS to bypass local DNS hijack). Mismatch: app locks with an alert ("This app may have been tampered with — download from official source only") and reports the discrepancy to the IEBC security operations centre.

*Certificate pinning:* The app pins the TLS leaf certificate for the verification API endpoint. Any MITM presenting a different certificate (even a CA-signed one) causes the request to fail with an explicit warning. Certificate rotation follows a published 90-day schedule, pre-announced on the IEBC website.

*Tamper-evident boot check:* Android: SafetyNet / Play Integrity API check on each launch; device integrity failure displayed to user with guidance. iOS: analogous jailbreak detection. Not a hard block (a voter's right to verify cannot be conditioned on owning a non-rooted device) but a visible warning.

*Spoofed-clone detection programme:* Datacraft security team runs weekly automated searches on Play Store and App Store for apps with similar names, icons, or descriptions. Takedown requests via Google's app policy and Apple's App Store Review within 24 hours of detection. Coordinated with Communications Authority of Kenya (CA) and IEBC via pre-agreed MoU for expedited takedown. WhatsApp Business verified badge and Facebook blue tick for official distribution channels.

*Published bug bounty:* KES 500,000 reward for first demonstrable exploit of the Public Verifier app or the hash-verification pipeline. Scope published on HackerOne. Serves as both defence (whitehats find it first) and reputational signal.

*Social-graph verification of authentic install source:* IEBC + Datacraft publish official installation links via IEBC.or.ke (DNSSEC-protected), the IAB's own domain, and three verified Twitter/X accounts. Any other source is unofficial by definition.

**Where it lands:** Pro-bono (PB-02 already in Part III). App-hardening costs absorbed in pro-bono commitment. Bug bounty in Undertaking Letter.

**Residual risk:** App store takedown latency (days to weeks) leaves spoofed apps available to unsophisticated users. Certificate-pinning bypass requires nation-state capability but is theoretically possible. In-app binary integrity check cannot detect a perfectly replicated binary with a single-byte malicious payload.

---

**Item 8 — Attacks on Observer and Auditor Infrastructure**

**Threat:** KEOG/ELOG parallel-vote-tabulation (PVT) infrastructure is a soft target: observer networks typically operate on consumer-grade hardware with volunteer-managed security. IAB members are individuals who may be physically accessible to intimidation. The cryptographic sortition RNG source (used to select the RLA audit sample, Part XIX.2) must be both unpredictable and publicly verifiable; a single-source RNG that is known in advance to any party allows pre-stuffing the to-be-audited stations.

**Precedent:** In the 2017 Kenya election, key IEBC ICT staff (including Chris Msando) were killed or intimidated. Observer networks in Zimbabwe (2018) and DRC (2019) faced co-ordinated communication disruptions on election night. RNG manipulation is documented in the Venezuelan 2004 referendum controversy (Smartmatic RNG concerns raised by Carter Center).

**Datacraft response:**

*Observer infrastructure hardening:* Datacraft offers, at cost, a hardened observer-infrastructure package to KEOG and ELOG: dedicated VPN endpoint (separate from the KIEMS network), encrypted result-capture tablets, and a read-only mirror of the HSM-signed audit log fed directly to the observer network in real time. Observers can run their own Form 34A aggregation and compare against published tallies in real time. This is offered, not imposed; KEOG/ELOG accept or decline independently. Cost: approximately KES 15M for the observer package.

*IAB member security programme:* KEOG-style close protection (contracted private security, not police) for IAB members during the operational period (T-30 to T+30), funded by Datacraft. IAB members provided with secure communications (Signal-based encrypted group, dedicated SIM cards on a carrier independent of the main KIEMS carrier), and a 24-hour security operations hotline.

*Multi-source cryptographic sortition:* The RLA audit-sample seed is derived from the XOR of five independent public entropy sources at the moment of draw (documented in advance, published in the Undertaking Letter): NIST Randomness Beacon (nist.gov/beacon), UK National Lottery draw (publicly observable), BBC World Service shortwave hash (publicly verifiable), a Kenya Gazette publication hash (IEBC issues the specific Gazette notice the day before the draw), and the SHA-256 hash of the last NIST Bitcoin block header at draw time. Any adversary who controls fewer than all five sources cannot predict the sample in advance. All five source values are published alongside the draw record in the audit log.

**Where it lands:** Observer infrastructure: offered at cost (not pro-bono); separate MoU with KEOG/ELOG. IAB security: pro-bono (Undertaking Letter). Multi-source RNG: architectural requirement, bid base.

**Residual risk:** A nation-state adversary could theoretically predict or manipulate the NIST beacon (US government controlled) but this requires US government complicity, which is bounded by reputational and legal constraints. Physical protection for IAB members does not prevent coercion that does not involve physical threat (e.g., family members, financial threats).

---

**Item 9 — DNS and BGP-Level Attacks on the Result Portal**

**Threat:** Beyond volumetric DDoS (covered in Part VIII C-series), routing-level attacks on the results portal are qualitatively different: a BGP hijack can silently redirect all traffic destined for iebc.or.ke to an attacker-controlled server presenting falsified results to the entire Kenyan internet without the client receiving any indication of attack. DNS poisoning at the registrar level can achieve similar redirection. These are nation-state-grade capabilities but have been demonstrated by non-state actors (BGP hijack incidents documented by BGPmon and RIPE NCC are numerous).

**Precedent:** Multiple BGP hijack incidents affecting government services: Pakistan Telecom hijacking YouTube (2008); Nigerian ISP briefly hijacking large internet prefixes (2019); Russia-linked BGP route manipulation (multiple RIPE NCC advisories 2020-2022). Kenya's ccTLD (.ke) is managed by KENIC; domain security standards have historically lagged.

**Datacraft response:**

*DNSSEC deployment:* iebc.or.ke DNSSEC signed. DS record published in the .ke zone (requires KENIC cooperation, sought via IEBC-Communications Authority channel). Verification by any DNSSEC-validating resolver. Datacraft provides DNSSEC configuration support to IEBC ICT as pro-bono advisory.

*Multi-registrar redundancy:* Domain registered at primary registrar (KENIC-accredited) with IANA-compliant secondary nameservers hosted by two independent providers in different AS numbers. Nameserver change requires two-factor authentication plus Datacraft-IEBC dual-authorisation.

*BGP monitoring:* Datacraft's SOC subscribes to BGPmon, Cloudflare Radar, and Kentik BGP alerts for the IEBC-announced IP prefix range. Any route-origin anomaly (prefix announced from unexpected AS) triggers SOC alert within 5 minutes and escalation to upstream ISPs (Safaricom, Airtel, Telkom Kenya) for RPKI-verified route filter enforcement.

*RPKI with upstream ISPs:* Pre-election MoU with Safaricom, Airtel Kenya, and Telkom Kenya to enforce RPKI Route Origin Validation (ROV) for IEBC-announced prefixes. Any RPKI-invalid announcement is dropped by the ISPs' routers. MoU negotiated via CA of Kenya facilitation.

*Anycast + geo-distributed CDN:* Results portal served via Anycast (Cloudflare or equivalent) from at least five PoPs in East Africa and Europe. BGP hijack affecting a single PoP degrades performance but does not redirect all users.

*Alternative access channels:* On-radio broadcast (KBC, Citizen, NTV reading Form 34A tallies live on air), WhatsApp Business (results pushed to verified channel), USSD short-code (pull-based result query for feature phones), and printed gazette publication. A BGP attack that succeeds in redirecting the web portal does not affect these channels.

**Where it lands:** DNSSEC advisory: pro-bono. BGP monitoring: bid base SOC (Part VIII, PB-07). RPKI MoU: IEBC-facilitated, no direct cost to Datacraft. Anycast CDN: bid base.

**Residual risk:** DNSSEC depends on KENIC's .ke zone infrastructure, which Datacraft cannot control. RPKI ROV is not mandatory in Kenya; ISP compliance is voluntary. A nation-state BGP attack with simultaneous DNS cache poisoning and CA compromise would defeat most of these controls simultaneously.

---

**Item 10 — AI-Enhanced Attacks**

**Threat:** Generative AI lowers the marginal cost of producing convincing fake Form 34A images to near zero. An adversary can generate tens of thousands of plausible-looking Form 34A images with falsified tallies, flood social media and WhatsApp, and create mass confusion about the correct tallies before the official results are published. Deepfake presiding officers or returning officers can produce convincing video claims of electoral fraud or irregularity. LLM-generated grievance floods can overwhelm the IEBC grievance system (Part XVIII CAOC), making it impossible to distinguish genuine complaints from synthetic ones.

**Precedent:** Team Jorge (Haaretz/Guardian investigation, February 2023) documented the sale of AI-enhanced disinformation campaigns for elections. IPAC (Integrated Public Alert and Communications) was overwhelmed in multiple documented cases (India 2024, UK 2024) by coordinated synthetic-content floods. Kenyan WhatsApp networks have been documented as primary vectors for election disinformation (Mozilla/Africa Check reports, 2022).

**Datacraft response:**

*Multimodal deepfake detection ensemble:* Three independent detection models (a fine-tuned vision transformer trained on Form 34A layout patterns, a general deepfake detector tuned on Kenyan political figures, and a metadata-consistency checker that verifies EXIF data, font hashing, and ink-pattern consistency) run on every Form 34A image that arrives via any channel not already in the HSM-signed log. Ensemble vote: if two or more flag suspicious, image is flagged for human review before publication.

*C2PA content provenance:* Every Datacraft-published Form 34A image is cryptographically signed using C2PA (Coalition for Content Provenance and Authenticity) v1.3 standard. Signature embeds: capture device ID, timestamp, GPS coordinates, and HSM-signed chain from kit to publication. Consumers can verify the C2PA manifest using any compliant reader. An image circulating without a valid C2PA manifest is, by definition, not Datacraft-published.

*LLM-generated grievance detection:* Grievance intake pipeline includes a text-classification model identifying synthetic-content patterns (unusually uniform phrasing, low perplexity, high similarity among batch submissions). Clusters of similar synthetic submissions are batched and routed to a human analyst as a group rather than individually. Rate limits (max 10 grievances per phone number per 6 hours) and CAPTCHA for web submissions reduce automated flooding.

*Adversarial-training regime:* Detection models are adversarially trained quarterly using the latest generation of image and text synthesis tools (GPT-4o, DALL-E 3, Stable Diffusion 3, Sora). Red team generates synthetic Forms 34A and synthetic grievances; blue team detects; results published in the quarterly IAB security report.

*Human-in-loop for aggregation:* Any trend of > 50 similar grievances within a 2-hour window at the same station triggers a senior analyst review, not automated processing. This prevents a LLM grievance flood from generating automated escalations that overwhelm the Returning Officer.

*Dedicated AI-attack red team:* Datacraft commissions a pre-election AI-attack red team exercise (T-90 to T-60) in which external contractors attempt to fool the C2PA verification, the Form 34A detection ensemble, and the grievance classifier using the best available tools at that date. Findings disclosed to IEBC and IAB. Mitigations applied before T-30.

**Where it lands:** Detection ensemble: bid base (SOC capability). C2PA: architectural requirement, bid base. Adversarial training: pro-bono (ongoing AI capacity). Red team exercise: pro-bono (Undertaking Letter commitment).

**Residual risk:** Detection accuracy is probabilistic, not absolute. A sufficiently sophisticated adversary with access to the detection model architecture (e.g., through insider access or public disclosure) can generate adversarial examples that evade detection. C2PA verification requires public adoption of verification tools, which is low in Kenya currently.

---

**Item 11 — Quantum-Cryptography Horizon**

**Threat:** The KIEMS cryptographic architecture (HSM FIPS 140-3, RSA-2048, ECDSA P-256, AES-256, SHA-256) is secure against classical attacks for the 2027 election cycle. However, the National Institute of Standards and Technology (NIST) published final post-quantum cryptography (PQC) standards in August 2024 (ML-KEM FIPS 203, ML-DSA FIPS 204, SLH-DSA FIPS 205). A cryptographically-relevant quantum computer (CRQC) that breaks RSA-2048 is estimated (NCSC, NIST, BSI) to be 10-15 years away at current projections. However, "harvest now, decrypt later" attacks are live today: an adversary harvesting the encrypted HSM-signed audit log now can decrypt it when a CRQC is available. For a 3-year election framework generating long-lived audit records, this is a bounded but real risk.

**Datacraft response:**

*Hybrid classical + post-quantum key architecture:* All new HSM-signed audit entries use a hybrid signature scheme: ECDSA P-256 (classical) + ML-DSA (FIPS 204, post-quantum). The hybrid signature is valid against classical and post-quantum verifiers. A classical verifier ignores the ML-DSA portion; a post-quantum verifier ignores the ECDSA portion. Both must agree for the entry to be accepted as authentic.

*Long-lived signature protection:* The reproducibility bundle (Part XI) and the pre-emptive evidentiary bundle (Part XII) are re-signed with hybrid signatures at T-30, T-14, T+1, and annually for the 3-year framework. Any document in the bundle carries a timestamp of its last re-signing.

*Key-rotation policy:* All HSM keys rotated annually. Old keys archived in escrow (NCC Group) with their corresponding signed audit ranges. Escrow deposit updated at each rotation.

*Published post-quantum migration roadmap:* A public document (published at T-180) specifying the migration timeline for each cryptographic primitive in the KIEMS architecture, aligned with NIST PQC standards. This commits Datacraft to a migration path and allows IAB to audit compliance.

*Escrow with post-quantum-signed metadata:* The NCC Group escrow deposit includes a PQC-signed manifest of all deposit contents from the first deposit onward.

**Where it lands:** PQC hybrid architecture: bid base (priced as a cryptographic uplift over standard KIEMS). Migration roadmap: Undertaking Letter. Pro-bono: annual roadmap update published for the 3-year framework.

**Residual risk:** ML-DSA (FIPS 204) is new and has had less public cryptanalysis than RSA or ECDSA. A cryptanalytic break of ML-DSA before the CRQC arrives would render the hybrid signature no more secure than the classical component. This risk is considered low by NIST but non-zero.

---

**Item 12 — Coordinated Cross-Tier Collusion**

**Threat:** The most sophisticated rigging scenario is not a single bad actor but a coordinated ring: a candidate bribes a party agent, a Returning Officer, a presiding officer, and an IEBC commissioner simultaneously at a single station or across a small cluster of stations. Each individual controls one layer of the verification architecture. Taken together, they may be able to submit a falsified Form 34A, suppress the biometric log, and prevent a legitimate complaint from reaching the Commission.

This is the 2022 Cherera 4 scenario generalised: commissioners dissent, implying internal disagreement about process, which in turn implies that some layer of the architecture failed to function independently.

**Precedent:** Cherera 4 commissioners' dissent (August 2022) documented in the Raila Odinga v William Ruto [2022] eKLR Supreme Court petition. While the petition was dismissed, the dissent is on record as evidence that IEBC's internal tier did not function as a unified, independently-verified system.

**Datacraft response:**

*Structural independence across all tiers:* No single tier's action affects the final tally without independent verification at a higher tier. The kit produces a cryptographically-signed Form 34A that is transmitted simultaneously to the Returning Officer, the national aggregation server, and the IAB read-only log. A colluding Returning Officer cannot modify the Form 34A after it has been transmitted to the national server and the IAB log.

*Cross-station statistical detection:* The XIX.1 anomaly detection layer (cross-station correlation) catches consistent directional bias across a cluster of stations even if each individual station's Form 34A appears internally consistent. A ring that controls five stations in a constituency will show a statistical signature in the turnout/vote-share distribution.

*Cryptographic sortition removes tier control of audit sample:* The RLA audit sample (Part XIX.2) is drawn using the multi-source RNG (Item 8 above). No tier can control which stations are audited in advance. A ring that controls five stations cannot know whether those five will be in the audit sample.

*IAB direct-report bypass:* Any observer at any tier with access to the read-only IAB log can escalate a concern directly to the IAB without going through the IEBC chain of command. This bypasses a colluding commissioner. The IAB has its own independent legal representation and can refer concerns directly to the DPP and Judiciary without IEBC mediation.

*Financial monitoring (Part XX):* The cash flows that fund a coordinated ring are detectable in aggregate via the FRC/CBK SAR system. A ring requiring simultaneous payment to five individuals across a constituency creates a detectable cash-movement pattern.

*Residual protection: election-day red team:* Datacraft's SOC runs a live red team during polling day, actively looking for statistical patterns suggesting coordinated manipulation. Findings go directly to the IAB within 30 minutes of detection.

**Where it lands:** Structural architecture: bid base. Statistical detection: bid base SOC. IAB bypass protocol: IAB mandate document. Financial monitoring: Part XX pro-bono programme. Red team: pro-bono (Undertaking Letter).

**Residual risk:** A ring that controls sufficient tiers and is sophisticated enough to fake the biometric log and simultaneously suppress every independent observer at those stations could potentially evade detection in real time. Post-election RLA would still detect it with 95% probability if the falsification is material to the result. The residual risk is a post-election court finding, not an undetected outcome.

---

**Item 13 — State Pre-election and Post-election Crisis**

**Threat:** Constitutional crisis, presidential emergency powers invoked to delay or suspend the election, or a refusal-to-cede by the incumbent (analogous to Belarus 2020, Venezuela 2024, or Kenya's own disputed-result scenarios in 2007 and 2017). Datacraft's contract and operations exist within a legal framework that can be suspended or invalidated by executive action. The Elections Act s.74 allows IEBC to extend polling by 24 hours in specific circumstances; the Constitution Art. 140 allows the Supreme Court to extend the inauguration timeline. These are bounded mechanisms. A full-scale constitutional crisis is outside any vendor's ability to control.

**Precedent:** Kenya 2007 post-election violence (1,300+ killed); Kenya 2017 annulment and re-run. Nakuru High Court in Mwangi v IEBC [2017] confirmed IEBC's authority to call a repeat election under Art. 140(3).

**Datacraft response:** This gap is explicitly acknowledged as beyond Datacraft's ability to resolve by technology. Datacraft's contract includes:

*Force-majeure clause:* Datacraft's performance obligations are suspended during a declared state of emergency or constitutional crisis as defined by the Attorney General's office. Datacraft is not liable for non-performance during force majeure.

*Data-preservation obligation survives:* The HSM-signed audit log and all collected biometric and result data are preserved regardless of force majeure, stored in the geographically-redundant data infrastructure (Part IV). Data is available to any subsequent legitimate legal process, tribunal, or commission of inquiry.

*Named-successor covenant:* A nominated Kenyan-operated firm (identified pre-contract) holds a standby operations agreement. If Datacraft cannot operate due to force majeure, the named successor takes over all Datacraft-controlled systems under pre-agreed protocols, with IEBC as the activating party. The successor firm has access rights to all source code, keys (via escrow), and documentation from day one of the contract.

*Legal-preservation-order protocol:* Pre-agreed with the Attorney General's office (via IEBC as intermediary): in any constitutional crisis scenario, Datacraft's data is placed under a preservation order that prevents destruction or modification by any party, including Datacraft and IEBC itself.

**Where it lands:** Force majeure: Subcontract Agreement Clause 12. Named-successor: Undertaking Letter. Preservation order: pre-agreed MoU with AG's office (post-award milestone). Architecture ensures data survival: bid base.

**Residual risk:** In a genuine constitutional crisis, legal frameworks may not be enforced. Data preservation depends on physical infrastructure not being destroyed. Named-successor assumes a functioning legal and commercial environment, which may not exist in a full crisis.

---

**Item 14 — Vendor-Capture of the PMO**

**Threat:** If IEBC leadership pressures Datacraft to bend a procedural rule — suppressing a detection alert, delaying a log publication, excluding a station from the RLA sample, removing an IAB observer's access — Datacraft faces a choice between compliance (and becoming complicit in rigging) and refusal (and risking contract termination, delayed payment, or legal action). This is vendor-capture: the client uses commercial leverage to compromise the vendor's integrity.

**Precedent:** Smartmatic's involvement in Venezuela (2004, 2016) raised documented concerns about whether client pressure influenced result reporting. OT-Morpho's Kenya 2017 tenure included a disputed server-access refusal episode (Raila v IEBC [2017] eKLR para 272) that the Supreme Court found dispositive — suggesting that whatever actually happened, the vendor and IEBC's response to it was, at minimum, inadequately transparent.

**Datacraft response:**

*Independent Assurance Board direct-report line:* The IAB has direct, unmediated read-only access to the full audit log at all times. Any instruction from IEBC to Datacraft that would affect the log (delay publication, suppress an entry) would immediately be detectable by the IAB as a log anomaly (the log is append-only; any suppressed entry creates a gap in the Merkle chain that any IAB member can verify independently). The IAB cannot be instructed by IEBC.

*Datacraft ethics committee with veto power:* Datacraft's internal ethics committee (three independent non-executive board members, named in the Subcontract Agreement) holds a veto over any instruction from any client that the PMO Director believes conflicts with Datacraft's contractual integrity obligations. The veto is exercisable without IEBC consent. Exercise of the veto is logged and disclosed to the IAB within 24 hours.

*Whistleblower protection:* Any Datacraft employee who reports a client instruction they believe to be improper is protected under the Whistleblower Protection Act 2016 and under Datacraft's internal policy, with access to a confidential external reporting channel (independent law firm on retainer, not the IEBC legal team). Reports are automatically routed to the IAB chair.

*Published ethics-decision log:* All ethics-committee decisions (including vetoed client instructions, redacted to protect operational security where necessary) are published quarterly in the IAB public report. This creates a public record and a deterrent against pressure.

*"Walk-away" clause:* Subcontract Agreement Clause 9.4: Datacraft may decline to execute any client instruction it determines (by ethics-committee vote) to be unlawful or to violate the integrity commitments in the Undertaking Letter, without incurring liquidated damages. The client may refer the dispute to expedited arbitration (NCIA Rules, 30-day timeline). During the arbitration period, Datacraft continues operating under the pre-instruction status quo.

**Where it lands:** IAB mandate document. Subcontract Agreement Clauses 9.4, 12.1. Ethics committee: Undertaking Letter. Whistleblower channel: pro-bono advisory (Undertaking Letter).

**Residual risk:** If the ethics committee itself is captured (three independent members pressured simultaneously), the veto fails. The IAB's independent log access remains as the final line of defence. Ultimately, Datacraft's integrity rests on the personal courage and independence of named individuals, which is a limit no architectural control can fully overcome.

---

### XXI.B — Process Areas (Items 15-19)

**Item 15 — Voter Deregistration Mechanics**

**Threat:** Incorrectly retained voters (deceased, emigrated, court-ordered deregistered) pad the roll and create ghost-voter attack surfaces. Incorrectly deregistered voters (family notification failure, CRS data error, system bug) disenfranchise legitimate voters and generate petition grounds. Both directions create legal and operational risk.

**Precedent:** OAG 2024 report found anomalies in the voter register. Raila v IEBC [2022] eKLR included challenges to register accuracy.

**Datacraft response:** Daily CRS-IEBC register sync (automated, with daily diff report to IEBC director of registration). Death notifications: CRS triggers deregistration within 48 hours of registration; family-notification short-code (SMS to registered next-of-kin number) confirms deregistration and provides appeal reference. Emigration-status tracking: joint KRA-PIN dormancy flag (no Kenya-source taxable activity for > 3 years) plus Immigration Department departure records trigger a "review" status, not automatic deregistration; voter must actively verify retention within 90 days of notification or be deregistered. Court-order integration: Judiciary CTS (Case Tracking System) webhook for disqualification orders (convicted of election offence under Election Offences Act 2016 s.14); deregistration within 72 hours of court order transmission. Deregistration-appeal workflow: 30-day appeal window, toll-free short-code, in-person IEBC county office option, online portal. Annual deregistration transparency report: total deregistered by category, total appeals, total reinstated, published on IEBC website and in the IAB quarterly report.

**Where it lands:** CRS integration: bid base (Technical Proposal Vol. II). Deregistration workflow: bid base. Transparency report: pro-bono (annual publication, Undertaking Letter).

**Residual risk:** CRS data quality depends on civil registration completeness, which is below 100% in Kenya. A deregistered-in-error voter who misses the 30-day appeal window loses their vote in the 2027 cycle. Emigration-status deregistration using KRA dormancy as a proxy will incorrectly flag some Kenyans who have not emigrated but have low taxable income.

---

**Item 16 — Kit Decommissioning and Biometric Data Destruction**

**Threat:** KIEMS kits accumulate biometric templates (fingerprints, iris images) for 22.1 million registered voters over the 3-year framework. DPA 2019 s.41 (storage-limitation principle) requires that personal data be deleted when no longer necessary for the purpose for which it was collected. End-of-contract data destruction is a legal obligation, not an option. Failure creates ODPC enforcement risk and undermines public trust in future enrolment.

**Datacraft response:** Cryptographic erasure of all biometric templates at end-of-contract: every template stored on kit local storage or the CIMS central server is AES-256-encrypted at rest with a key held in the kit's TPM. At end-of-contract, the key destruction ceremony (ODPC-witnessed, IEBC-signed) permanently destroys all TPM keys, rendering all encrypted templates permanently unrecoverable without any requirement to overwrite storage media. ODPC-witnessed destruction certificates issued within 30 days of end-of-contract date, published on IEBC website. Physical destruction ceremony for kits and storage media: any kit determined to be beyond refurbishment is physically destroyed (degaussing for magnetic media, shredding for solid-state, witnessed by IEBC asset officer and ODPC representative). Kit-donation programme: kits in working condition post-data-wipe are donated to Kenyan public schools as computing resources (tablets, keyboards, batteries), maximising public-good value of the hardware investment. ODPC data-protection impact assessment (DPIA) updated at end-of-contract to reflect destruction outcomes.

**Where it lands:** DPA 2019 compliance obligation: bid base. Destruction ceremony: Undertaking Letter. Kit-donation programme: pro-bono (social-impact commitment, mentioned in bid narrative).

**Residual risk:** Cryptographic erasure depends on TPM key-destruction ceremony executing correctly. If any kit's TPM is faulty at end-of-contract, the encrypted data may persist in encrypted form. Encrypted persistence is low-risk (templates without decryption keys are computationally unrecoverable) but is technically a storage-limitation violation. Mitigation: physical destruction of any kit with a TPM fault confirmed at decommissioning inspection.

---

**Item 17 — By-Election Operations**

**Threat:** Kenya holds by-elections whenever a parliamentary or ward seat falls vacant (death, court nullification, recall). Frequency: approximately 10-20 by-elections per year across all levels. By-elections involve a small subset of kits, reduced staffing, and a shorter mobilisation window, creating both cost-model and operational-readiness questions outside the general-election framework.

**Datacraft response:** The 3-year framework contract includes a by-election service tier. A dedicated by-election kit pool (200 kits, held in Nairobi bonded storage) is available for rapid deployment within 72 hours of IEBC activation. Per-by-election unit pricing is pre-agreed in the Commercial Proposal (day-rate per kit deployed + logistics flat fee + comms flat fee). Existing KIEMS firmware and software cover by-elections without modification: the kit is configured for the specific constituency roll pre-deployment. Result transmission, Form 34A publication, and IAB log access all function identically to general-election operations at reduced scale. SLA: Form 34A published within 45 minutes of station close for any by-election.

**Where it lands:** By-election tier: Commercial Proposal Vol. III (unit-rate schedule). Kit pool: priced in base as standby inventory. SLA in Subcontract Agreement Schedule 3.

**Residual risk:** If more than 200 stations require simultaneous by-election coverage (e.g., a large county nullification), the standby pool is insufficient. Mitigation: right-to-mobilise-general-pool provision in Schedule 3, triggered by IEBC notice.

---

**Item 18 — Referendum Mechanics**

**Threat:** Kenya has experience with constitutional referendums (2010 Constitution; 2021 BBI initiative, ultimately invalidated by the courts). A referendum presents a simpler question-structure than a general election (single binary choice) but may involve a different voter-eligibility framework, a different Form structure (Form 38A replaces Form 34A), and a different result-computation methodology (simple majority of valid votes cast, not constituency-based).

**Datacraft response:** KIEMS kit referendum-mode switch: software configuration flag converts the kit from multi-race election mode to single-question referendum mode. Referendum-specific ballot interface (single yes/no option, translated into all 42 languages per Election Offences Act requirements). Form 38A capture and transmission in place of Form 34A. Constituency referendum tally and national tally computation in the aggregation server. Referendum-specific dashboard on the public results portal. Voter eligibility: referendum eligibility is typically the same as general-election eligibility (registered voters on the IEBC roll); if Parliament creates a different eligibility framework for a specific referendum, the roll-subset configuration is updated pre-deployment. Post-award specification exercise required if a referendum is called during the 3-year framework.

**Where it lands:** Referendum mode: bid base (included as standard feature of KIEMS architecture). Post-award specification: to be agreed with IEBC via change-control process.

**Residual risk:** If the referendum involves a fundamentally different eligibility framework (e.g., lowering the voting age to 16 as a referendum question), the CRS enrolment database may not include all eligible voters and a fresh enrolment window may be required. This is an IEBC and Parliament decision outside Datacraft's control.

---

**Item 19 — Post-Inauguration Continuous Services**

**Threat:** The 3-year framework does not end on inauguration day. IEBC's mandate continues: voter-register maintenance, boundary review (every 8-10 years per IEBC Act s.26), by-elections, tribunal support, and KIEMS infrastructure upkeep. A KIEMS vendor whose contract ends abruptly on T+30 leaves IEBC with hardware it cannot operate, software it cannot maintain, and data it cannot access without the vendor's cooperation. Vendor lock-in is a structural risk even for vendors with good intentions.

**Datacraft response:** The 3-year framework covers: voter-register maintenance (daily CRS sync, quarterly audits, annual deregistration transparency report, all per Item 15). Boundary-review support: IEBC boundary-review teams get access to the spatial analytics tool (Part XVI.1) for the full 3 years. Pre-2032-election preparatory work: at T+24 months, a 2032-cycle readiness assessment is delivered (hardware lifecycle, software upgrade path, firmware post-quantum migration status, procurement lead-time estimate). CVR (Continuous Voter Registration) daily operation: portal, short-code, and county-office integration remain live for the full 3 years. IEBC ICT support desk: Datacraft-staffed L1/L2 support, 24x7 during election periods, business-hours otherwise. Capacity-building programme: structured technology-transfer to IEBC in-house ICT team, targeting self-sufficiency by 2032. Programme includes: source-code walk-throughs, on-site training for IEBC ICT staff, access to Datacraft's internal documentation (not escrow documents, which remain with NCC Group), and a joint IEBC-Datacraft skills-transfer calendar with quarterly milestones. Goal: by 2032, IEBC's in-house team can operate and maintain the KIEMS infrastructure without Datacraft involvement, reducing procurement risk for future cycles.

**Where it lands:** 3-year maintenance and operations: Commercial Proposal Vol. III (annual fee). Capacity-building: pro-bono commitment (Undertaking Letter). 2032-cycle assessment: pro-bono (published report).

**Residual risk:** IEBC staff turnover may frustrate the capacity-building programme. Technology transfer requires willing institutional counterparts. If the 2027 election is politically disrupted, IEBC's ICT team may be unable to absorb training in a compressed timeline.

---

### XXI.C — Meta and Structural Gaps (Items 20-24)

**Item 20 — Long-Drip Attacks**

**Threat:** All prior parts model attacks bounded within the 6-12 month campaign and election window. Some of the most effective electoral manipulation unfolds over 24-36 months: slow voter-roll poisoning (systematic registration of ineligible voters over multiple enrolment windows), gradual boundary distortion (incremental constituency-boundary changes favourable to one party, each individually defensible), sustained disinformation seeding (long-term narrative building that frames a candidate as illegitimate years before the election). These attacks do not trigger the anomaly-detection systems designed for election-day spikes.

**Datacraft response:** Continuous baseline monitoring programme operating for the full 3-year framework. Quarterly voter-roll integrity audits (automated cross-referencing of the IEBC register against CRS, KRA, Immigration, and UNHCR data) published as public reports, creating a rolling public record that makes incremental roll-poisoning visible. Continuous boundary-fairness dashboard (Part XVI.1): spatial analytics updated monthly, flagging constituency-population deviations from the statutory norm (Elections Act s.24 requirement for roughly equal constituency populations). Long-horizon disinformation-seeding detection: quarterly longitudinal social-media analysis (3-year rolling window) by the social-media monitoring team (Part XVI.2), identifying narratives that begin building years before an election campaign. Annual public integrity report: comprehensive year-on-year comparison across all monitored dimensions, published by Datacraft and the IAB jointly, creating a public audit trail for long-drip attacks.

**Where it lands:** Quarterly roll audits: 3-year framework operations, Commercial Proposal Vol. III (annual fee). Boundary dashboard: bid base (3-year licence). Disinformation monitoring: pro-bono (reduced intensity outside election periods). Annual integrity report: pro-bono (Undertaking Letter).

**Residual risk:** Longitudinal disinformation detection is an art, not a science. A sophisticated long-term narrative campaign that stays below the detection threshold of automated tools will not be flagged. Boundary manipulation requires IEBC and Parliament to act; Datacraft can only surface the data, not compel a response.

---

**Item 21 — Second-Order Political Dynamics (Datacraft's Own Risk)**

**Threat:** Datacraft is a technology vendor, not a political actor. But technology vendors in high-stakes elections are inevitably drawn into the political narrative of the result. If the 2027 election produces a winner from any major political coalition, Datacraft risks being credited (or blamed) for that outcome by the losing side. Losing parties have historically targeted the election-technology vendor as a proxy for contesting the result (Smartmatic in Venezuela 2004 and 2016; OT-Morpho in Kenya 2017; Dominion Voting Systems in the USA 2020-2024). Reputational blowback can affect Datacraft's ability to bid for future contracts, attract talent, retain financing, and protect its personnel.

This is not just a reputational risk: a winning party may also attempt to claim Datacraft as its political ally, creating pressure to maintain a relationship that compromises neutrality in subsequent elections.

**Precedent:** Dominion Voting Systems suffered approximately USD 1.6 billion in claimed damages from a defamation lawsuit against Fox News following the 2020 US election (settled for USD 787.5M, April 2023). OT-Morpho's servers were the subject of a politically-charged access demand in 2017 that, regardless of its merit, permanently damaged the company's Kenyan market position. Smartmatic's global reputation was so damaged by Venezuela and Philippines controversies that it has been effectively excluded from several mature-democracy markets.

**Datacraft response:**

*Named-successor protocol:* Datacraft maintains a standby agreement with a named Kenyan successor firm (Item 13 above). If political blowback reaches a point where Datacraft's continued involvement compromises IEBC's perceived independence, the named successor can take over operations under a pre-agreed protocol. This is a last-resort option, not a default.

*Board-level ethics commitment, publicly stated:* Datacraft's board adopts and publishes a formal statement before contract signature: "Datacraft delivers electoral integrity regardless of which candidate or coalition wins. Datacraft has no interest in any election outcome. Datacraft's commercial success depends on every client trusting that the system is impartial." This statement is incorporated into the Undertaking Letter.

*Legal and insurance shield for Datacraft personnel:* Directors and officers (D&O) insurance at minimum USD 10M per claim, covering post-election defamation actions, wrongful-termination claims, and regulatory investigations. Named personnel covered by personal-liability insurance for actions taken in good-faith performance of their election duties.

*Neutral-communications posture:* Datacraft does not comment publicly on election results, candidate performance, or political narratives. Any public communication is limited to factual statements about process and system performance (e.g., "100% of Form 34A images were published within the SLA window"). All media inquiries routed through IEBC's communications team, not Datacraft's. Datacraft employees are prohibited from making public statements about election outcomes during the operational period.

*Published integrity metrics as the reputational shield:* The best defence against political attack is a public record of transparent, verifiable, independently-audited process metrics. The IAB quarterly reports, the reproducibility bundle, the C2PA-signed Form 34A archive, and the annual integrity report collectively make any post-election attack on Datacraft's integrity contestable with documentary evidence.

*ISO 27001 certification:* Datacraft pursues and maintains ISO 27001 certification (information security management system) as an independent quality signal from a reputable international body. Certification is evidence-based and survives political attack because it is issued by a body (BSI, Bureau Veritas, or equivalent) that is itself independent of Kenyan politics.

*Consideration for future cycles:* Datacraft evaluates, by T+12 months, whether to seek certification from an Africa-based independent body (e.g., ARSO, African Organisation for Standardisation) as an additional quality signal with regional credibility.

**Where it lands:** Named-successor: Undertaking Letter. Ethics commitment: Undertaking Letter (public). D&O insurance: Datacraft internal cost. Neutral-communications posture: internal policy (disclosed to IEBC). ISO 27001: Datacraft operational investment, not a bid cost.

**Residual risk:** No institutional protection eliminates the risk of sustained, well-funded political attack. Dominion Voting Systems spent over USD 200M in legal fees before settling. Datacraft is smaller and has less financial resilience for a prolonged legal campaign. The ultimate protection is personal courage of named individuals, public transparency of the process record, and the independence of the Kenyan judiciary.

---

**Item 22 — Weather and Natural Disaster**

**Threat:** Kenya's long rains (March-May) coincide with the typical election-campaign window in a March-election scenario, and have repeatedly caused localised flooding in Nyanza, Western, Coast, and Northeastern regions. The 2024 long-rains flooding (the most severe in Kenya's recorded history per Kenya Meteorological Department) killed over 200 people and displaced over 300,000. A general election scheduled for August 2027 avoids the worst of the long rains but still faces the short rains (October-December 2026 during the campaign) and the possibility of unseasonable weather. Kit hardware is rated for standard operating conditions (15-35 degrees C, up to 85% humidity); high-humidity coastal and lakeshore environments may exceed these ratings.

**Datacraft response:** Long-rains contingency: the pricing guidance (Part VI item 6 of the Pricing Guidance document) includes a spare kit tranche equal to 5% of constituency allocation, pre-positioned in constituencies at highest flood risk (identified via Kenya Meteorological Department historical-flood-probability map) at T-60. These kits are in waterproof Pelican-case transport containers, rated to IP67. Satellite-uplink backup for network-dependent stations in flood-prone areas: Iridium or Inmarsat BGAN terminal, pre-positioned at T-30. Generator-power standard: all stations with grid-power vulnerability identified at T-90 (via KETRACO and rural electrification survey); generator sets pre-positioned at T-30. Hardware environmental rating: kit hardware specification includes extended-temperature and humidity tolerance (5-45 degrees C, 10-95% RH non-condensing), achieved through selection criteria in the OEM specification document. Rescheduling-protocol: pre-agreed with IEBC for catastrophic-weather scenarios under Elections Act s.74 (24-hour extension) or Constitution Art. 138(4) (extended presidential election period in extraordinary circumstances).

**Where it lands:** Spare kit tranche: bid base (priced). Satellite uplink and generators: bid base (standby charge). Rescheduling protocol: Subcontract Agreement Schedule 4 (force majeure sub-plan).

**Residual risk:** A catastrophic weather event (e.g., the 2024 severity, occurring during polling week) could disable multiple constituencies simultaneously and exceed the spare-kit and generator pre-positioning. The rescheduling protocol requires IEBC and court approval that may not be achievable quickly under crisis conditions.

---

**Item 23 — KEBS and CA Certification Renewals**

**Threat:** KIEMS kits require KEBS type-approval (Kenya Standards KS 2400) and Communications Authority of Kenya (CA) device-type approval for the LTE modem component. These certifications have renewal cycles (typically 2-3 years for KEBS type-approval; annual registration for CA-licensed devices). If a renewal cycle falls within the election operational period, the kits may technically be operating on lapsed certification, creating a bid-protest or petition ground.

**Datacraft response:** All certification renewal cycles mapped against the election calendar at contract signature (T-0). A certification-calendar is maintained by the Datacraft compliance team and reviewed monthly in the Steering Committee (PM-02). Buffer of 60 days ahead of any critical certification expiry: renewal applications submitted at T-60 relative to expiry, not T-30. Dual-source certification where possible: for KEBS type-approval, Datacraft pursues certification of both the primary Mechrevo hardware configuration and the contingent-supplier configuration simultaneously. Contingent re-certification plan: if a certification renewal is unexpectedly delayed (KEBS or CA backlog), the Datacraft legal team prepares a legal opinion (from a Kenya-qualified commercial barrister) that the kits are operating under a pending renewal with full legal continuity, for use in any bid-protest or petition defence. Communications Authority liaison: Datacraft maintains a dedicated CA liaison officer (part-time, Nairobi-based) for the full 3-year framework.

**Where it lands:** Certification-calendar: bid base (compliance management). Dual-source certification: bid base. Contingent re-certification: bid base (legal retainer cost). CA liaison: bid base.

**Residual risk:** Certification timelines are within government agency control, not Datacraft's. A prolonged KEBS or CA backlog (documented as a common experience for complex electronic products in Kenya) could create a certification gap despite adequate advance submission. The legal-opinion mitigation reduces but does not eliminate the risk.

---

**Item 24 — Competing Vendor Sabotage During the Bid**

**Threat:** A competing vendor or a vendor's political patron may attempt to derail Datacraft's bid through: filing a speculative bid protest with PPRA or PPARB; spreading disinformation about Mechrevo's PRC ownership to IEBC evaluators or media; initiating or sponsoring defamation claims against Datacraft principals; leaking confidential bid documents to a competitor; or attempting to engineer an exclusion on technical-compliance grounds by lobbying IEBC evaluators.

**Precedent:** PPRA 2015 bid-protest mechanisms have been used routinely in Kenya to delay large procurements (telecommunications, security systems). OT-Morpho's 2017 one-day tender was itself an extreme form of competing-vendor exclusion. Media campaigns against election-tech vendors are well-documented (Smartmatic in Venezuela, Dominion in USA).

**Datacraft response:** Bid-integrity monitoring: Datacraft legal team monitors PPRA, PPARB, and media for any protest or claim relating to Tender IEBC/OIT/01/2026-2027. Legal team on retainer (Nairobi-qualified commercial litigation firm) for immediate bid-protest response. PPRA-compliance evidence file prepared at T-0: complete audit trail of Datacraft's PPRA compliance (mandatory technical requirements met, financial capacity documented, conflict-of-interest declarations filed, local content requirements met), ready for submission to PPARB within 5 working days of any protest. Defamation-response protocol: any published false statement about Datacraft or Mechrevo triggering IEBC evaluator concern is met with a factual counter-statement from Datacraft's public affairs desk within 24 hours, circulated to IEBC procurement committee and PPRA. Bid confidentiality: all Datacraft personnel with access to bid documents sign confidentiality agreements with liquidated damages of KES 50M for any breach. Evaluation-integrity: Datacraft's engagement with IEBC evaluators is conducted exclusively through the official procurement channels (written clarifications, public forums, official presentations). No side-channel communication with evaluators is permitted.

**Where it lands:** Legal retainer: Datacraft operational cost. PPRA-compliance file: bid base (legal preparation). Defamation protocol: Datacraft operational policy. Bid confidentiality: HR and legal.

**Residual risk:** A well-funded competing vendor with political connections can sustain a bid protest for months and create procurement delays that disadvantage Datacraft even if ultimately unsuccessful. The PPARB process is bounded by statute but not immune to political influence.

---

### XXI.D — What Have We STILL Not Covered

This closing section is honest. Part XXI itself is a gap-closure exercise. The act of closing 24 gaps opens secondary gaps. Here are the ten most important ones we have still not covered, with an explanation of why.

**1. Voter psychology under coercion at scale.** Items 1, 3, and 14 of Part XX address vote-buying and coercion at the individual level. Mass coercion — systematic intimidation of an entire constituency by a political-militia or landlord network — is a different phenomenon. No technology controls it. The police and the IPOA are the relevant actors. Datacraft cannot help here except by ensuring that coercion leaves no fingerprint in the KIEMS record (because the KIEMS record is secret and coercion-resistant by design).

**2. Post-election reconciliation politics.** The Ruto-Raila handshake (March 2018) and the ODM-Jubilee MOU (2023) demonstrate that Kenyan political actors often reach negotiated settlements outside the legal and technical process. If a losing party accepts a negotiated settlement in exchange for political appointments, the technical integrity of the result becomes irrelevant to the settlement. Datacraft's contribution is to ensure the result is accurate, so that any settlement is reached despite an accurate result rather than because of a falsified one.

**3. Cross-border election-technology interoperability.** East African Community integration trends (EAC Common Market Protocol) may eventually create pressure for cross-border voter-identity verification (to prevent double-registration in multiple EAC member states). The architecture in Part III and Part XIV does not address this. The EACRB (East African Court of Justice) has jurisdiction over some EAC-protocol disputes. No design exists for this and no IEBC tender scope covers it.

**4. Kenyan public-institution baseline capacity.** The integrity of the result ultimately depends on the Kenyan judiciary (Article 140 petitions), the Attorney General's office (legal-preservation orders), the DPP (prosecutions), the DCI (criminal investigations), and the KNP ICT infrastructure (biometric data sharing). All of these institutions have documented capacity constraints and are outside Datacraft's control. If the Judiciary's IT infrastructure cannot handle the expected petition filings, or the DPP declines to prosecute documented offences, Datacraft's technical integrity delivers no legal consequence.

**5. Long-term erosion of Article 88 independence.** The Constitution Art. 88 creates IEBC as an independent commission. IEBC's independence has been structurally eroded in each election cycle since 2010: commissioner replacements under political pressure (2017, 2022), funding constraints, staff insecurity. If IEBC is not genuinely independent, Datacraft's architecture — which depends on IEBC as the trustworthy counterpart — is operating on a false premise. This is a constitutional design problem that 30 years of political science literature has not resolved in similar contexts.

**6. Second-order pricing effects on the market.** If Datacraft's transparency programme (reproducibility bundle, IAB, C2PA, RLA) becomes the expected standard for IEBC procurements, every future vendor must match it. This raises the cost floor for election technology in Kenya. Whether this is sustainable at Datacraft's margin depends on scale, technology cost trajectories (hardware costs decrease; AI detection costs decrease; cryptographic infrastructure is largely fixed-cost). Datacraft's best protection is to be the vendor that defined the standard, giving it a first-mover advantage in delivering it efficiently.

**7. Successor-election reproducibility: does the 2027 architecture survive to 2032?** The capacity-building programme (Item 19) targets IEBC self-sufficiency by 2032. But IEBC self-sufficiency in operating the 2027 architecture is different from IEBC being able to procure, configure, and deploy a successor architecture for 2032. The 2027 architecture will be approximately 5 years old by the time the 2032 election procurement begins. Post-quantum migration, hardware refresh, AI-detection model updates, and KIEMS firmware evolution all create procurement complexity. Datacraft can offer, but not commit, to supporting 2032 procurement planning from T+24 months. The 2032 question is genuinely open.

**8. Public-good sustainability of pro-bono commitments.** Part III and Parts IX, X, XIV-XX collectively commit Datacraft to approximately KES 800M-1.2B in pro-bono contributions over the 3-year framework (estimated). These are funded from Datacraft's margin on the bid base and from partnership contributions (universities, civil society, CBK/FRC, KEOG/ELOG). In a scenario where the bid-base margin is compressed by post-award cost overruns or unforeseen technical issues, pro-bono commitments face budget pressure. The Undertaking Letter locks Datacraft to these commitments contractually, which is the right incentive structure, but the funding source is the same contract that may be under pressure. This needs a ring-fenced pro-bono budget line in the Commercial Proposal.

**9. Accessibility for voters with disabilities.** The kit includes audio-assisted voting (Part III PB-02 general mention) but no detailed accessibility specification for voters with visual, hearing, mobility, or cognitive disabilities. The Constitution Art. 54 guarantees the rights of persons with disabilities. Elections Act regulations require accessible polling stations. A comprehensive accessibility specification — audio-ballot in all 42 languages, Braille ballot on request, tactile guidance cards, wheelchair-accessible booth dimensions, assisted-voter consent and witness protocol — has not been detailed. This is a gap in Parts III and XIV.

**10. Climate-change long-run risk to election infrastructure.** The 2024 Kenya floods were described by Kenya Meteorological Department as a 1-in-50-year event. Climate projections for East Africa (IPCC AR6, 2021) indicate increasing frequency and severity of extreme rainfall events in the 2025-2050 window. The weather contingency in Item 22 above addresses a single-cycle event. A long-run infrastructure planning question — should KIEMS kit hardware have a higher environmental rating, should constituency offices be flood-resilient, should satellite uplinks be standard rather than backup — is outside any single procurement tender scope. Datacraft flags this for IEBC's long-term infrastructure planning.

---

### XXI.E — Final Honest Paragraph

The Datacraft bid does not claim to solve every possible attack. It claims to solve every attack solvable by a technology vendor operating within the law, and to make explicit and public the attacks it cannot solve. That honesty is not a weakness. A vendor who claims to solve everything either does not understand the threat landscape or is not being straight with their client. IEBC has been misled by vendors before, with consequences visible in every Kenyan election since 2007. The Datacraft approach treats IEBC as a sophisticated counterpart capable of understanding that residual risk is not failure — it is reality, properly managed.

---

## Executive Summary — v3.1 (Datacraft Augmented Bid Strategy, 2026-08-20)

> Parts VIII through XXI added 2026-08-20. Parts I-VII unchanged.

### A. Abstract

The Datacraft bid is not the cheapest bid. It is the bid that guarantees IEBC delivers a fair, verifiable, secure, and unchallengeable election — and that advances Kenya's democratic science beyond 2027.

Parts VIII through XXI add fourteen augmentation modules to the base strategy. The inventory-management design (INVENTORY-MANAGEMENT.md) landed after the Part XXI wrap-up and constitutes a fifteenth substantive workstream; three of its controls moved the differentiator frontier sufficiently to earn top-shortlist placement in v3.1. Part VIII enumerates 110 attack items across eleven categories. Parts IX through XIX build the affirmative programme: public confidence, AI utilisation, outcome reproducibility, a No-Challenge Guarantee, fraud prevention, three constitutional voting guarantees, verification and queue tracking, election environment integrity, automated counting, full rules compliance, and state-of-the-art audit methods including the SHANGRLA Risk-Limiting Audit framework. Part XX targets the cash-to-votes conversion ratio directly with structural ballot-secrecy design, a Campaign Finance Transparency Programme, and six evidence-based behavioural interventions. Part XXI closes the 24 remaining identified gaps with architectural responses, honest scope acknowledgements, and a second-order gap register of the 10 issues that remain genuinely open.

Any petition filed against IEBC results delivered by this system faces: evidentiary saturation, procedural pre-emption, reputational cost imposition, and deterrent economics. The 15-item pre-emptive evidentiary bundle, delivered to the Supreme Court IT Registrar at T-14, is unprecedented in Kenyan election-technology history. This is the No-Challenge Guarantee.

### B. Top-Ten Differentiators

The inventory-management design landed after the Part XXI wrap-up and introduced controls more concrete than most items the wrap-up produced. Three of those controls directly answer the biggest historical grievances — ghost stations, ballot-reconciliation gaps, and the 2017 KES 9bn Auditor-General chain-of-custody finding — so they displace three Part XXI items that lacked an equivalent hard anchor to a prior-cycle Kenyan failure.

1. **500 m TPM-enforced geofence per kit** — the first anti-ghost-station technical control in African election history. A kit refuses to activate outside its gazetted station's GPS boundary; the rule runs in the TEE/TPM, not the app layer; every enforcement event is kit-signed and logged to the Merkle chain. Software bypass is insufficient — a hardware attack on the TPM is required to defeat it. See: INVENTORY-MANAGEMENT.md §3.5.

2. **Ballot-to-voter Merkle chain with smart-lid RFID cross-check** — every ballot chip read on deposit, reconciled in real time against biometric authentications; any variance triggers an immediate P0 alert before the box is sealed. The court-admissible chain closes the single largest unresolved fraud vector in Kenyan election history. See: INVENTORY-MANAGEMENT.md §2.3, §4.3.

3. **100% reverse-logistics recovery + DPA 2019 cryptographic erasure by T+90, witnessed by IAB** — directly answers the 2017 KES 9bn Auditor-General chain-of-custody gap and removes DPA 2019 retention-petition grounds. Erasure certificate (SHA-256 hash of zeroed partition) is stored in IMS and countersigned by Datacraft DPO and IAB. See: INVENTORY-MANAGEMENT.md §7, §10.

4. Merkle-chained append-only audit log with root published every 30 minutes and an open-source verification tool — converts every "hidden servers" petition ground into a mathematical impossibility.

5. Pre-registered Supreme Court technical witness with 24-hour log-production commitment — directly pre-empts the 2017 dispositive annulment ground (Raila v IEBC [2017] eKLR para 272).

6. Ballot-polling Risk-Limiting Audit using the SHANGRLA framework — the first RLA in Kenyan election history; provides a mathematical guarantee that the reported winner actually won at 95% confidence.

7. Laptop-runnable reproduce-me bundle published within 4 hours of Form 34C — any Kenyan with 16 GB RAM and a `make` command can independently verify the national tally.

8. Independent Assurance Board with continuous read-only log access, quarterly public reports, and a direct-report bypass of the IEBC chain of command — the standing institutional immune system that vendor-capture (Item 14, Part XXI) cannot reach.

9. Seven-layer anomaly and fraud detection stack plus cross-tier collusion detection (Item 12, Part XXI) — defence in depth that a rigger must defeat simultaneously on all fronts.

10. No-Challenge Guarantee with a 15-item pre-emptive evidentiary bundle delivered to the Supreme Court IT Registrar at T-14 — no prior Kenyan KIEMS vendor has published a pre-election legal evidence bundle.

### C. Single Most Compelling Sentence for the Bid Cover Letter

"Datacraft is the first KIEMS vendor in Kenya's history to publish a pre-election mathematical proof that the results cannot have been altered, a reproduce-me bundle any Kenyan can run on their own laptop, and a 24-gap honest register of the attacks it cannot solve — because a vendor who tells you everything is covered is a vendor who has not done the analysis."

### D. Final Counts

| Item | Count |
|---|---|
| Attack items in Part VIII | 110 (categories A-K) |
| Augmentation modules (Parts VIII-XXI) | 14 |
| Gap-closure items in Part XXI | 24 |
| Second-order open gaps acknowledged | 10 |
| Concrete controls and commitments across all parts | > 320 |
| Unique legal citations and case references | > 55 |
| Pro-bono items at KES 0 to IEBC | 30+ |
| Academic references cited with author-year | > 25 |

### E. On Residual Risk

Every architecture has a residual risk. The Datacraft architecture reduces residual risk to the boundary of what a technology vendor can achieve within the Kenyan legal and commercial framework. What lies beyond that boundary — voter psychology under mass coercion, constitutional erosion, post-election political settlement, cross-border interoperability, the capacity of the Kenyan judiciary and DPP — is real, is documented in the gap registers, and is not Datacraft's to solve alone. That honesty is not a confession of weakness. It is the foundation of a trust relationship with IEBC, with civil society, and with the Kenyan electorate that no prior KIEMS vendor has offered.

### F. Bid Packaging Summary

| Volume | Content |
|---|---|
| Technical Proposal Vol. II — Implementation | Parts I-VII; PM-01 to PM-12; Part X (AI); Part XVII (counting); Part XVIII (compliance) |
| Technical Proposal Vol. II — Security | Part VIII C-series; Part XI (reproducibility); Part XIV (voting guarantees); Part XIX.2 (RLA); Part XX.2 (ballot secrecy); Part XXI Items 5-12 (attack-vector responses) |
| Technical Proposal Vol. V — No-Challenge Guarantee | Part XII (full); Part XIX.6 (RLA commitment) |
| Technical Proposal Annex A — Pro-Bono | PB-01 to PB-08; Part IX; Part XVII; Part XIX.6; Parts XX, XXI pro-bono items |
| Technical Proposal Annex B — AI Programme | Part X (full) |
| Technical Proposal Annex C — Petition Defence | PB-08; Part XII.4 (pre-emptive bundle); Part XII.5 (pre-appointments) |
| Technical Proposal Annex D — Vote Integrity | Parts XIII, XIV, XV, XX |
| Technical Proposal Annex E — Gap Register | Part XXI (full) |
| Commercial Proposal Vol. III | Parts II-III pricing; G-series mitigations; by-election unit rates; referendum mode; 3-year maintenance |
| Undertaking Letter | Firmware hash Gazette; reproducibility bundle; 24-hour log production; named technical witness; code escrow; participation receipt; campaign finance dashboard; multi-source RNG; walk-away clause; named-successor covenant; IAB security programme; post-quantum roadmap |
| Subcontract Agreement Schedules | Metric SLAs from Parts XIV, XV, XVII, XVIII, XIX, XX, XXI |

---

*Parts VIII through XXI added 2026-08-20. All counts and citations are internal research estimates. This document is internal strategy and does not represent IEBC policy or any committed Datacraft commercial position until such commitments are made in the formal bid submission.*

---

**Change log**

| Date | Version | Change |
|---|---|---|
| 2026-08-20 | v3 | Parts VIII-XXI added (14 augmentation modules, 24-item gap register) |
| 2026-08-20 | v3.1 | Promoted three inventory-management differentiators to top of shortlist; dropped Campaign Finance Transparency Programme, Participation Receipt Programme, and Multi-source sortition/Public Verifier composite (weakest prior-cycle anchors) |
