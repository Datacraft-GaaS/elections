# 6. Universal edge-case matrix

Every WP must map each applicable row to ≥1 tagged passing test. Surface key: K kit · M mobile · B backend · P portal · D desktop · L ML · T batch. Rows marked (gap) are not anticipated in the spec corpus and are added here.

| ID | Category | Edge case | Surfaces | Required behaviour | Test |
|---|---|---|---|---|---|
| EC-CON-01 | connectivity | Offline 7 days (kit) / 48 h results queue | K | Ring-buffer queue, signed+sequenced, sync with gap audit | kit-simulator netem loss 100%; proptest queue never drops |
| EC-CON-02 | connectivity | 3G (500 kbps, 300 ms RTT) | K M P | Module 3G SLAs met; CBOR+zstd negotiated | k6 behind toxiproxy; patrol throttled emulator |
| EC-CON-03 | connectivity | 2G only | K M | Kit refuses 2G data, uses signed SMS/USSD; app shows USSD path | modem-sim 2G; SMS gateway contract |
| EC-CON-04 | connectivity | Site partition primary/mirror | B T | Patroni fencing, no split-brain, MirrorMaker lag alert, RPO ≤5 min | Chaos Mesh partition; linearizability check on tally stream |
| EC-CON-05 | connectivity | Flapping link | K M B | Backoff+jitter, retry budget, no duplicates | toxiproxy toggle; server dedupe count 0 |
| EC-CON-06 | connectivity | Captive portal | K M D | Pinning fails closed, "network blocked" UX, no token leak | mitmproxy captive mode; zero auth headers |
| EC-CON-07 | connectivity | Satellite 600 ms+ RTT | K B | Transport-tuned timeouts, MTU-safe, SMS-size compression | netem satellite; fuzz payload sizes |
| EC-CON-08 | connectivity | DNS poisoned/down | K M B | DNSSEC, hardcoded fallback endpoints, circuit opens | DNSChaos; bogus CoreDNS |
| EC-CON-09 | connectivity | IPsec tunnel drop mid-batch | K B | Resume at sequence, idempotent re-send | strongSwan kill; gap audit |
| EC-CON-10 | connectivity | Dual-SIM failover mid-send | K | Same signed blob + key on SIM B | modem-sim switch; one row |
| EC-CON-11 | connectivity | Kafka ISR shrink | B | Producer blocks, outbox accumulates, page, no loss | kill 2 brokers; outbox drains |
| EC-TIM-01 | time | Kit clock skew >5 min at boot | K | Refuse session until NTS/GNSS time | time-fence FSM unit; faked RTC |
| EC-TIM-02 | time | NTP/NTS unreachable all day | K B | Last auth time + monotonic, `time_confidence=low`, tolerance window | proptest HLC; chrony blocked |
| EC-TIM-03 | time | Backend clock jumps backward | B | HLC never regresses, skew alert | TimeChaos −10 min; kani HLC merge |
| EC-TIM-04 | time | DST/TZ on diaspora kits | K B P | UTC storage, gazetted TZ display, close computed in UTC | tz snapshot; kit in America/New_York DST day |
| EC-TIM-05 | time | Leap second (gap) | B | Tolerates 23:59:60 | fuzz timestamp parser |
| EC-TIM-06 | time | Event time < causation time | B T | Accept, audit skew anomaly, never reorder | projection independent of wall order |
| EC-TIM-07 | time | s.14A close moves 17:00→19:00 | K B | HSM-signed extension reopens fence; forged rejected | queue-svc flow; forged negative |
| EC-TIM-08 | time | Petition window T+7 midnight | B P | EAT statutory, §75(1) inclusive | boundary table ±1 s |
| EC-TIM-09 | time | Register freeze race T-60/T-30 (gap) | B T | OPA hard-close at tx time; all-or-nothing | concurrent writers; TLA+ freeze model |
| EC-IDN-01 | identity | Kit cert revoked mid-session | K B | Refuse ≤60 s; honey mode if stolen; queue quarantined | SPIRE/step-ca revoke; ≤60 s |
| EC-IDN-02 | identity | JWT expires mid-request | M P B | In-flight completes; next 401 + silent refresh | Keycloak 5 s TTL |
| EC-IDN-03 | identity | SVID rotates during batch | B T | Old SVID valid to expiry; `producer_key_id` per event | SPIRE 30 s rotation |
| EC-IDN-04 | identity | Attestation fails (custom ROM) | M | Public: USSD/SMS path; accredited: refuse + support code | patrol MEETS_NONE stub |
| EC-IDN-05 | identity | Rooted/emulator/Frida | M | RASP terminate, wipe, tamper event | rooted AVD matrix |
| EC-IDN-06 | identity | Clone app | M B | Pin mismatch; backend rejects digest; M28 store detector | resigned APK |
| EC-IDN-07 | identity | SIM swap on staff OTP (gap) | M B | No SMS OTP for accredited; public OTP device-bound | ASVS V2; new-device denied |
| EC-IDN-08 | identity | Duress finger | K B | Covert ECC alert, UI identical | simulator duress template; snapshot equal |
| EC-IDN-09 | identity | Dual-control, approver revoked mid-flow | B P | Re-validate both at commit | revoke between steps |
| EC-IDN-10 | identity | SpiceDB down | B | Fail closed writes; class-C reads only | PodChaos; 503 problem+json |
| EC-IDN-11 | identity | Realm key rotated, stale JWKS | B P | Refetch on kid miss | rotate in testcontainer |
| EC-DAT-01 | data | Duplicate submission, same key+body | K M B | Cached response, no re-execution | auto-generated per write endpoint |
| EC-DAT-02 | data | Same key, different body | B | 422 `idempotency-key-reuse` | auto-generated |
| EC-DAT-03 | data | Out-of-order in stream | B | Buffer until gap filled or DLQ | shuffled replay property |
| EC-DAT-04 | data | Replayed signed event | K B | Reject on counter; `.tamper` | re-send capture; fuzz counter |
| EC-DAT-05 | data | Poison message (votes > registered) | B L | DLQ with reason; partition continues | inject; lag returns 0 |
| EC-DAT-06 | data | Schema v1/v2 coexist | B M | Both accepted; dual-emit; compat check | Karapace compat; mixed stream |
| EC-DAT-07 | data | Partial batch failure | B T | Per-row report; valid committed | 30M synthetic seeded errors |
| EC-DAT-08 | data | 22-30M row ops | B T | Streaming, checkpoint-resumable, bounded memory | kill-resume test |
| EC-DAT-09 | data | Unicode names | K B P M | NFC, ICU collation, grapheme-safe | hypothesis unicode; 11-locale snapshots |
| EC-DAT-10 | data | Zero results | B P D L | Valid; no div-by-zero; renders "0" | unit + property; Playwright |
| EC-DAT-11 | data | Spoilt>valid, totals>registered | K D B | Block with reason; RO override → DisputeCase | table-driven; TLA+ invariant |
| EC-DAT-12 | data | Debezium slot lag / WAL bloat | B | Alert 60 s; outbox retained; disk guard | pause connector; no loss |
| EC-DAT-13 | data | Soft-deleted row in search | B P | Tombstone honoured; purge removes | delete-then-search; purge job |
| EC-DAT-14 | data | Oversized event | B | Outbox rejects >1 MB; blob→MinIO + hash | unit; fuzz size |
| EC-DAT-15 | data | IPRS returns two persons | B | AdjudicationCase; no auto-merge | IPRS stub duplicate |
| EC-DAT-16 | data | Legacy kit pre-CloudEvents payload | K B | Shim upgrades envelope; signature preserved | Pact shim↔bvi; golden field corpus |
| EC-SEC-01 | security | Signature mismatch | B | Never applied; tamper topic; page ≤30 s | flip byte |
| EC-SEC-02 | security | Tamper (seal/chassis) | K B | Kill-on-open; quarantine stream; ECC alert; custody event | simulator GPIO; inventory FSM |
| EC-SEC-03 | security | HSM unavailable | B | STH pauses, ingest continues, countersigns queue, degraded mode | kill SoftHSM; no gap |
| EC-SEC-04 | security | KEK rotation mid-cycle | B T | Online re-wrap; old readable until done; audit | OpenBao transit rotate under writes |
| EC-SEC-05 | security | Injection on every input | B P | Parameterised only | Semgrep; sqlx compile-time; ZAP; fuzz |
| EC-SEC-06 | security | SSRF via URL fields | B | Allowlist, no private ranges, rebinding guard | rebinding resolver; Nuclei |
| EC-SEC-07 | security | Path traversal / zip-slip | B D T | Sanitised | fuzz extractor; `../` corpus |
| EC-SEC-08 | security | XXE / deserialization (SAML, shapefile, PDF) | B | DTD off; no pickle; strict models | billion-laughs; Bandit; fuzz parsers |
| EC-SEC-09 | security | Rate-limit evasion | B P | Key subject>device>/64; headers; priority shed | k6 rotating; Nuclei |
| EC-SEC-10 | security | Voter enumeration (M04/M07/M64) | B P | Constant-time shape; caps; step-up; redacted | timing oracle p95 Δ<5 ms; harness |
| EC-SEC-11 | security | Timing side-channel on secrets | K M B | Constant-time compare | dudect-style; Semgrep ban `==` |
| EC-SEC-12 | security | Cache poisoning of snapshots | P B | Key includes sig version; client verifies | Varnish injection; SDK verify |
| EC-SEC-13 | security | Split-view on log | B | Witness divergence; consistency fails | forked log; kani |
| EC-SEC-14 | security | Stolen kit honey mode | K B | Sandbox served; attacker biometric captured | revoked-cert simulator |
| EC-SEC-15 | security | LiveView session hijack | P | CSRF + session binding + re-auth on step-up | ExUnit + Playwright |
| EC-SEC-16 | security | Secrets in logs/traces | all | Telemetry refuses Tier A/B | log-scanner fixture; CI regex |
| EC-SEC-17 | security | PQ hybrid negotiation failure | K B | Classical TLS 1.3 only; never 1.2 | testssl.sh; PQ-disabled client |
| EC-HUM-01 | human | Worn/faded fingerprints | K | Multi-finger retry, adaptive threshold, s.44A-compliant fallback | degraded corpus; UAT |
| EC-HUM-02 | human | Amputee (gap) | K | Face/ID path, `biometric_exception`, no denial | FSM unit; patrol |
| EC-HUM-03 | human | Gloves/wet/dust (gap) | K | Liveness reject + prompt; latency kept | HIL bench; quality injection |
| EC-HUM-04 | human | Low-light / direct sun | K | Auto-exposure, torch, perspective, quality gate | ML eval set; golden images |
| EC-HUM-05 | human | Assisted voting | K M B | Assistant recorded, consent, no secrecy leak, audio | workflow tests; PWD UAT |
| EC-HUM-06 | human | Screen readers | M P K | Full task completion; AA, AAA M68 | axe + manual scripts; semantics |
| EC-HUM-07 | human | Deaf / KSL video | M P | KSL for every explainer/M13 flow; captions | asset presence; UAT |
| EC-HUM-08 | human | 11 languages, long strings | all | No truncation; ICU plurals; font fallback | 11 locales + 150% pseudo |
| EC-HUM-09 | human | Colour-blind symbols/colours | P K D | Symbol+text+pattern; ΔE registry | ΔE unit; axe; simulation |
| EC-HUM-10 | human | Low-literacy USSD | M B | Numeric ≤5 steps; IVR alt | USSD simulator walk |
| EC-HUM-11 | human | Braille/tactile overlay (gap) | K | ≥48 dp calibrated; audio confirm | bench; hit-target tests |
| EC-HUM-12 | human | Clerk fatigue wrong column | K D | ICR cross-check; low confidence forces re-entry | ML eval; validator unit |
| EC-LEG-01 | legal | Candidate withdraws/dies post-print (gap) | B P K D | Status frozen on ballot; tallied + annotated; vacancy rules | rules unit; TLA+ annotated candidate |
| EC-LEG-02 | legal | Court-ordered postponement (gap) | B P K | Signed calendar amendment; kits refuse to open; portal notice | calendar→bundle; e2e |
| EC-LEG-03 | legal | s.44A invoked | K B P | Dual-officer S44AEvent; manual IDs logged; ECC + public count | drill script; audit replay |
| EC-LEG-04 | legal | Recount ordered | B D P | New version via compensating event; both shown with proofs | amend; kani append-only |
| EC-LEG-05 | legal | Presidential run-off (gap) | B P | Rules engine; declaration blocked; run-off created | TLA+; property distributions |
| EC-LEG-06 | legal | Tie (gap) | B P | `TIE_PENDING`; no certificate | unit + TLA+ |
| EC-LEG-07 | legal | Referendum 24-of-47 off-by-one | B | Exact ≥ semantics | TLA+ + boundary |
| EC-LEG-08 | legal | Evidence request at T+6d 23h | B P | 24 h clock; not freeze-blocked | timed; 200 concurrent bundles |
| EC-LEG-09 | legal | T-60 auto-referral race | B | Single referral; idempotent scheduler | double-fire |
| EC-LEG-10 | legal | Duplicate Daraja callback | B | Single payment; idempotent refund | stub replay |
| EC-LEG-11 | legal | Correction after declaration | B P | Amendment + re-declaration dual custody | integration + tlog inclusion |
| EC-OPS-01 | operations | Kit battery death mid-session | K | Journal-before-act; exact resume; no double token | power-cut at every transition |
| EC-OPS-02 | operations | Kit swap ≤10 min | K M B | Inherit roll + watermark; old revoked; streams linked | no double-ID |
| EC-OPS-03 | operations | Seal broken in transit | K M B | Custody event; quarantine; alert | inventory FSM |
| EC-OPS-04 | operations | Dual-control, officer absent | K B | Time-boxed ECC remote co-sign | integration + ExUnit |
| EC-OPS-05 | operations | Power outage at tally centre | D B | SQLite WAL resume; no duplicate 34B | tauri-driver kill |
| EC-OPS-06 | operations | Flood/curfew relocation (gap) | K B P | HSM-signed polygon update only; portal notice | geofence bundle; forged negative |
| EC-OPS-07 | operations | Staff no-show / swap | M B | Reassignment; credential exactly-once; old revoked | exactly-once credential |
| EC-OPS-08 | operations | Disk full / bucket quota | B | Bounded queues; 507; alert; no corruption | IOChaos fill |
| EC-OPS-09 | operations | Wrong degradation mode | B P | Dual sign-off on T-0; reversible; logged | flag flow; drill |
| EC-OPS-10 | operations | PITR restore during petition | B T | Drill; RPO/RTO; restore logged to tlog | DR harness |
| EC-SCL-01 | scale | 5M visitors/hr | P B | ≥95% edge; origin p99 500 ms | k6+Gatling 5 regions; hit ratio |
| EC-SCL-02 | scale | Thundering herd on STH | B M | SDK jitter; ETag 304; rate-limit | 100k verifiers simulator |
| EC-SCL-03 | scale | Hot partition | B | stream-id spread; lag alert; HPA | 100k ev/s skew 30% |
| EC-SCL-04 | scale | Consumer lag >60 s | B | Page; backpressure; no OOM | slow consumer |
| EC-SCL-05 | scale | Sequencer stall | B | Writes unaffected; cadence alert; drain ≥ peak | pause 10 min |
| EC-SCL-06 | scale | Tiering lag / cold replay | B T | ≤10 MB/s throttle; snapshot-then-tail | tiered container |
| EC-SCL-07 | scale | 250k check-ins, 60k RTS, 18k BVI | B M K | SLAs at 1x; ≤10% p99 regression at 5x | simulator pre-prod |
| EC-SCL-08 | scale | Valkey eviction loses idempotency | B | PG-durable store; Valkey cache only | flush mid-test |
| EC-ML-01 | ML | Model drift | L B | Drift alert; review ratio rises; card versioned | held-out eval; drift sim |
| EC-ML-02 | ML | Adversarial input | L | Confidence gate; no crash | hypothesis + ART |
| EC-ML-03 | ML | Torn/wet/stamped/screen-photo forms | L K D | Quality gate → manual | 2k degraded corpus |
| EC-ML-04 | ML | Bias across groups | L | Per-group metrics in tolerance; in card | fairness harness |
| EC-ML-05 | ML | PII in ML logs/prompts | L B | PIIStrippingEvent before external call | regression corpus; Semgrep |
| EC-ML-06 | ML | Wrong/unsigned model served | L K | cosign-verified; kit refuses | unsigned push rejected |
| EC-ML-07 | ML | Alert flood | L P | Dedup, rate-limit, ranked | 10k alerts; ECC inbox |
| EC-MOB-01 | mobile | OS kills mid-sync | M | Resumable; no dup; WAL intact | background-kill each step |
| EC-MOB-02 | mobile | Storage full | M | Graceful refusal; bounded queue | filled disk |
| EC-MOB-03 | mobile | 1 GB RAM KES-10k device | M | ≤15 MB; p99 2 s 3G; no OOM | device farm; memory gate |
| EC-MOB-04 | mobile | Doze/background limits | M | Resume on foreground; accredited terminate | patrol Doze |
| EC-MOB-05 | mobile | Push token rotation / FCM down | M B | Idempotent re-register; SMS for critical | FCM stub |
| EC-MOB-06 | mobile | Update mid-election below floor | M B | Force floor; 7-day block; queue survives | N-1→N migration |
| EC-MOB-07 | mobile | GPS spoof check-in | M B | Cell/Wi-Fi delta >1 km flagged | mock-location |
| EC-MOB-08 | mobile | Device time wrong | M | Server authoritative | unit |
| EC-MOB-09 | mobile | Biometric sensor broken | M | PIN fallback rate-limited; no policy downgrade | patrol |
| EC-KIT-01 | kit | GNSS jam/spoof | K | Multi-constellation + IMU; refuse outside fence | HIL GNSS sim |
| EC-KIT-02 | kit | Legacy Android parity | K B | Software attestation tier; policy-marked; parity matrix | Pact; matrix both targets |
| EC-KIT-03 | kit | OTA interrupted | K | A/B slots; rollback; signed only | power-cut during OTA |
| EC-KIT-04 | kit | Mode switch not propagated | K B | Signed bundle acked; unacked listed in ECC | 65k ack tracking |

118 rows.

