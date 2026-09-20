# ADR 0025: Factory distribution, offshore build, and pull+test hosting

Status: proposed
Date: 2026-09-19
Authors: Datacraft (software prime)
Scope: how the China factory (hardware prime) obtains, builds, tests, and loads kit software without receiving the full `eel` monorepo, production keys, or voter data.

## Context

### 1. Mandate and roles

`eel` is the integrated elections management platform built under tender IEBC/OIT/01/2026-2027. Software prime: Datacraft. Hardware prime: Mechrevo. Data residency is Kenya (two IEBC DR sites + five regional edges); audit retention is fifteen years; T1 election-critical modules run at 99.99% with synchronous DR replication.

The factory's job is narrow: receive kit software, build it reproducibly where necessary, validate it on real devices (KIEMS kits + legacy Android), flash/load approved APKs via `adb`, run the scripted drills, and return signed test evidence. It does not tally, deduplicate biometrics, sign transparency-tree heads, operate HSMs, or deploy backend services.

The factory is in China. Datacraft engineering is in Kenya. Authoritative infrastructure available today is Contabo Hub-Europe plus one Linode UK Nanode (see `infra/docs/INFRASTRUCTURE.md`, `NETWORK_TOPOLOGY.md`, `PLATFORM_ACCESS.md`, `PORT_MAP.md`). There is no Asia point of presence and no budget assumed for one in this ADR.

### 2. What "all this code" actually is

The `eel` monorepo is 18 Kubernetes services across Rust (~workspace with `Cargo.lock` pinned, `rust-toolchain.toml`), Flutter/Dart kits, Django, Elixir LiveView, Astro, Tauri, plus Python ONNX sidecars:

- Device-adjacent (factory-relevant): `services/biometric-edge-svc` (M01/M05/M24), `services/field-mobile-svc` (M13, M29–M34), `services/rts-svc` device crates (M08), `services/bvr-svc` device crates (M01), Flutter kit apps, `factory_pack/` (APKs, `antiris/` JARs + `.so` for `armeabi-v7a`/`arm64-v8a`, `factory_test.sh`, `smoke.sh`, `kill-drill.sh`, `offline-drill.sh`, `test_harness.sh`, `SHA256SUMS`, EN + 中文 guides).
- Backend-only (factory must never receive): `services/abis-svc` enclaves and matcher crates, `services/transparency-svc` Merkle log + HSM signer (P06), `services/rts-svc` backend tally/declaration (M09/M10/M51/M63), `services/governance-svc`, backend DB migrations, HSM policies, signing keys, any voter PII or biometric templates.

The existing `factory_pack/` + `.woodpecker/` (`build/test/sign/sbom.yaml`) already encodes the intended contract: versioned APKs, hash-pinned artifacts, scripted pass/fail drills, signed provenance. This ADR promotes that contract to hosted infrastructure.

### 3. Infrastructure reality (read from `infra/docs` at decision time)

| Host | Public IP | WG | Role today | Relevance to this ADR |
|---|---|---|---|---|
| `spare-3` | `161.97.121.202` | `10.8.0.11` | Spare / unallocated, 200 GB, Caddy `:80/:443`, only loopback `8100/8110` live | **Authoritative factory host.** Only fleet member with spare capacity and a clean port map. |
| `linode` (UK Nanode) | `88.80.188.224` | `10.8.0.14` | Egress node, Caddy live, user `nyimbi` (not root), Cloak/Nitter/StratEx | **Read mirror + artifact CDN failover.** Too small to build; correct size to mirror. |
| `auth` | `62.84.181.55` | `10.8.0.3` | Keycloak, SpiceDB, MinIO (`s3.lindela.io` → `127.0.0.1:9002`), OpenBao, Temporal, Documenso | Identity, artifact bucket, release-key custody. |
| `ml` | `62.169.25.77` | `10.8.0.4` | Ollama (`qwen3:4b`, `qwen3:8b`, `nomic-embed-text`), LiteLLM `:4000`, NLLB `:5500`, MOSS/STT; **no `:443` listener, Caddy installed but disabled** (2026-09-17) | Chat-with-code inference. Internal/WG only until TLS is restored. |
| `appserver` | `84.46.253.178` | `10.8.0.10` | Product apps + Qdrant `:6333/:6334` loopback, Caddy gateway | Vector store for scoped code index. |
| `mail` | `217.76.53.144` | `10.8.0.5` | Stalwart, Prometheus `:9090`, Grafana, Loki, Wazuh manager | Monitoring + SIEM. Factory host must be enrolled here. |
| `db-main` / `db-backup` | `144.91.122.29` / `144.91.112.190` | replication pair | PostgreSQL 17 + pgBackRest to Linode Object Storage (`gb-lon-1`) | Pattern to reuse for `git bundle` backups. Not otherwise involved. |

Firewall posture fleet-wide: UFW `deny incoming`, allow `22/80/443` + `51820/udp`, plus source-specific application rules (cf. `NETWORK_TOPOLOGY.md`). SSH is `root` + `~/.ssh/id_lindela` / `id_ed25519`; secrets live in local `pass` (`pjs/...`). Nothing in this ADR changes those primitives; it composes them.

### 4. China-link constraints

- Nairobi/Frankfurt/London → China is 180–400 ms with loss and periodic throttling of long-lived SSH and `github.com` raw fetches by the GFW. HTTPS/443 through Caddy survives far better than SSH or WireGuard-UDP alone.
- `crates.io`, `pub.dev`, PyPI, and HuggingFace are slow or flaky from China without in-country mirrors. Any workflow that requires the factory to fetch upstream on every build will fail intermittently and produce irreproducible artifacts.
- Time zones: Nairobi UTC+3 vs Beijing UTC+8 (5 h). All CI timestamps, evidence filenames, and release notes are UTC (ISO-8601, `factory-report-<APP>-<serial>-<UTC>.json`).
- Language: factory operators work in Chinese; engineers bilingual. Every operator-facing page and every chatbot answer must be available EN + 中文, following the existing `INSTALLER_TESTER_GUIDE.md` precedent.

## Principles

1. **Least privilege is design, not obscurity.** Kerckhoffs's principle requires the system to remain secure even if the adversary knows the algorithms. It does not require handing keys, HSM code, tally logic, and biometric enclaves to the hardware builder. Compartmentation between builder (Mechrevo) and verifier (Datacraft/IEBC) is the control; this ADR mechanises it.
2. **Builder/verifier separation.** The factory builds kits and asserts "it runs on hardware." Only Kenya signs releases and promotes to MDM/production. No factory credential can promote.
3. **Reproducibility over trust.** Acceptance is bit-identical rebuilds + `SHA256SUMS` + SLSA/in-toto provenance + drill evidence — not "factory says it works." Extends ADR-0006 (Nix, Woodpecker, cosign, SLSA L3) and the M14 reproducibility bundle.
4. **Sovereignty by default.** Code the factory may see may transit EU mirrors; voter data, biometric templates, HSM keys, and `pass` secrets never leave Kenya/IEBC infrastructure. SaaS LLMs and SaaS CI that retain code are rejected for this corpus (see §8).
5. **Pull-only factory.** The factory has no push path to any authoritative branch. Its only upstream message is test-evidence upload. This single rule eliminates an entire class of supply-chain writes.
6. **Bilingual operability.** A Chinese-speaking installer with `adb` and this ADR's download page must complete a load without asking Nairobi a question. Anything that fails that test is incomplete.

## Decision

### D1. Ship a stripped `eel-factory` repo, not the monorepo

- CI on the Datacraft side generates `eel-factory` from `eel` `main` on every kit tag via `git subtree split` (or `git filter-repo` allowlist). Inclusion allowlist (only these paths enter the factory repo):
  - `services/biometric-edge-svc/apps/**`, `services/field-mobile-svc/apps/**`, kit Flutter apps under `services/*/apps/**` per ADR-0024,
  - `services/bvr-svc` + `services/rts-svc` **device crates only** (explicit crate list; backend crates excluded by name),
  - `factory_pack/**` (scripts, `antiris/` binaries, guides, `SHA256SUMS`),
  - `docs/plans/modules/{bvr-device,bvi-device,rts-device}.md`, `factory_pack/{INSTALLER_TESTER_GUIDE,ENGINEERING_FAQ,factory_guidance,compile_apks}.md`,
  - `rust-toolchain.toml`, kit `Cargo.lock`/`pubspec.lock` snapshots, `flake.nix` kit profile, SBOM for the tag.
- Everything else — `abis-svc`, `transparency-svc` + P06 `hsm-signer`, backend tally/declaration, governance/court tooling, DB migrations, `infra/` secrets, `.env`, `*.pem` — is structurally incapable of reaching the factory because it is never in its repo.
- Tags are the contract: `kit-v<MAJOR>.<MINOR>.<PATCH>` (e.g. `kit-v1.4.2`). `main` of `eel` is never exposed. Each tag carries: source zip, APKs, `SHA256SUMS`, CycloneDX SBOM, Woodpecker build log with embedded `GIT_SHA`, and the drill scripts at that tag.

### D2. Hosting topology: authoritative EU-DE, mirror EU-UK, identity/store on auth

```text
Datacraft (Kenya) ──push──> spare-3 [AUTHORITATIVE eel-factory + evidence inbox]
                              │  Forgejo loopback :3007 ← Caddy factory.datacraft.co.ke
                              │  Woodpecker runner (authoritative signer)
                              │  artifacts → MinIO on auth, bucket: eel-factory
                              │  nightly git bundle → auth MinIO + Linode Object Storage
                              ╰──WG rsync──> linode [READ MIRROR + APK CDN]
                                               Caddy factory-mirror.lindela.io → loopback :3008
China factory ──HTTPS pull (primary spare-3, failover linode)──> tags + APKs + SHA256SUMS
              ──HTTPS POST evidence JSON──> spare-3 inbox (no git push exists)
```

- `spare-3` (`161.97.121.202`): Forgejo on loopback `127.0.0.1:3007` (free per `PORT_MAP.md`; only `8100/8110` live there today), fronted by existing Caddy as `factory.datacraft.co.ke → 127.0.0.1:3007`. One Woodpecker agent on the same host for authoritative (signing) builds. Static download vhost `factory-dl.datacraft.co.ke` serving the current tag's pack (redirects to MinIO presigned URLs; no MinIO credentials issued to factory).
- `linode` (`88.80.188.224`): hourly `git push --mirror` + `rclone sync` of the current pack over WG (`10.8.0.11 → 10.8.0.14`). Caddy `factory-mirror.lindela.io → 127.0.0.1:3008` serving a static mirror + APK cache. No builders, no runners, no secrets on this host. If DE→CN stalls, factory retries UK with the same tag and same hashes.
- `auth` (`62.84.181.55`): Keycloak realm/coins for factory service accounts, MinIO bucket `eel-factory` (versioned, object-lock for released tags), OpenBao custody of the release-signing key (cf. ADR-0005 M-of-N ceremony). Factory never talks to `auth` directly; `spare-3` mints short-lived presigned GET/PUT URLs.
- No new hardware. A Singapore mirror is explicitly deferred: add it only if 30 days of pull metrics show DE+UK p95 tag-fetch time breaching the SLO (§D9).

### D3. DNS, Caddy, ports, firewall (concrete)

| Record / route | Target | Notes |
|---|---|---|
| `factory.datacraft.co.ke` | `161.97.121.202` (spare-3) Caddy → `127.0.0.1:3007` | Forgejo HTTPS. HSTS, TLS via existing Caddy/ACME. |
| `factory-dl.datacraft.co.ke` | spare-3 static → MinIO presigned | Download page + `SHA256SUMS`. Rate-limited (see below). |
| `factory-mirror.lindela.io` | `88.80.188.224` (linode) Caddy → `127.0.0.1:3008` | Static git mirror + APK cache. |
| Forgejo `:3007`, mirror `:3008` | loopback only | Matches fleet pattern (loopback app + Caddy 443). Never wildcard-bound. |
| Evidence inbox | `factory.datacraft.co.ke/api/evidence` → loopback `:3009` | Authenticated POST, 10 MB cap, schema-validated. |

UFW (additive, following `NETWORK_TOPOLOGY.md` source-specific style; no `ufw enable` recipes, no default changes):

- `spare-3`: allow `80,443/tcp` world; `51820/udp` to WG peers only; `3007/3009` from `127.0.0.1` only; `22/tcp` from Datacraft office/bastion only; `9100/tcp` from `mail` (`217.76.53.144`) for node_exporter (authorised monitoring pattern).
- `linode`: allow `80,443/tcp` world; `51820/udp` to WG peers; `22/tcp` from Datacraft only; `3008` loopback only.
- Caddy hardening on both factory vhosts: `rate_limit 20r/m per IP` on clone/zip endpoints, `max_body 10MB` on evidence, access logs shipped to Loki on `mail`, `X-Content-Type-Options`/`frame-ancestors 'none'`.

### D4. Access model (pull-only)

- Factory holds exactly two credentials, both least-privilege: (a) one Forgejo **read-only deploy token** per site (`factory-shenzhen-ro`), IP-pinned, 90-day expiry, scope `read:contents` on `eel-factory` only; (b) one evidence-upload token (`factory-evidence-w`) scoped to `POST /api/evidence` + presigned MinIO PUT for its own reports. No SSH accounts, no WG membership for factory laptops, no `id_lindela` reuse, no shared passwords.
- Datacraft holds push keys (YubiKey-backed SSH, expiry 1 y) for `eel → eel-factory` split publishing and tag signing. Tag signatures are HSM-anchored II per ADR-0005/0006; factory verifies with the published public key (`apksigner verify --verbose`, `cosign verify`, `sha256sum -c SHA256SUMS`).
- Branch protection is moot (factory cannot push), but `eel` side keeps: protected `main`, required Woodpecker green (`build/test/contracts/pii-lint/sbom/sign`), required human review, 30-day pre-election freeze gate per ADR-0014/0022.

### D5. Build, sign, and artifact flow (extends ADR-0006, does not fork it)

1. Datacraft tags `eel` (`kit-vX.Y.Z`) → Woodpecker `build.yaml` + `mobile.yaml` produce `bvi/bvr/rts-kit-{debug,release}.apk`, `factory_test_harness.apk`, SBOM (Syft/CycloneDX), SLSA provenance + in-toto attestations, `cosign` signatures (keyless SPIFFE for CI, HSM-anchored for release).
2. Split job publishes the stripped `eel-factory` repo at the same tag + uploads APKs/SBOM/logs to `auth` MinIO `eel-factory/kit-vX.Y.Z/` (object-lock on `*-release.apk` + `SHA256SUMS`).
3. `spare-3` verifies hashes, serves the tag; WG job replicates to `linode`.
4. Factory flow per device batch (unchanged scripts, hosted artifacts):
   ```bash
   curl -O https://factory-dl.datacraft.co.ke/kit-v1.4.2/SHA256SUMS
   curl -O https://factory-dl.datacraft.co.ke/kit-v1.4.2/<APP>-release.apk
   sha256sum -c SHA256SUMS                  # hard fail on mismatch
   apksigner verify --verbose <APP>-release.apk
   adb install -r -g <APP>-release.apk
   ./factory_test.sh <APP> --device <serial>  # smoke → kill-drill → offline-drill → harness
   curl -F "report=@factory-report-<APP>-<serial>-<UTC>.json" \
        -H "Authorization: Bearer $EVIDENCE_TOKEN" \
        https://factory.datacraft.co.ke/api/evidence
   ```
5. Promotion to MDM/production is a Kenya-side dual-sign action reading the evidence inbox; factory credentials cannot promote. Debug APKs remain drill/test-harness only (`run-as` retrieval needs debuggable builds); release APKs in this channel carry the **factory-validation key**, never the production MDM key.

### D6. China-proof dependencies (reproducibility is the feature)

- All factory builds run `cargo --frozen --locked`, `flutter pub get --offline` after first fetch, `uv --frozen`, `pnpm --frozen-lockfile`, against vendored inputs shipped in the tag: `Cargo.lock`, `pubspec.lock`, `antiris/*.jar|*.so`, Nix flake inputs pinned. No live upstream fetch on the build path (ADR-0006 air-gap rule applies to the factory runner too).
- `compile_apks.md` documents the sanctioned CN mirrors (`rsproxy.cn` / TUNA for crates.io and Flutter, TUNA PyPI) as *bootstrap* sources only; the authoritative inputs remain the vendored locks. Any build that resolves a version not in the lock fails closed.
- Base images, JDK (17), Flutter (3.19.0+), Android SDK platform-tools/`aapt`/`apksigner` versions are pinned in the tag's `factory_toolchain.txt` and checked by `factory_test.sh` preamble.

### D7. Test and evidence gate (what "deploy to Android devices" means here)

- "Deploy" in this ADR means **load to attached devices over `adb` on the factory floor**, never push to production, MDM, or any backend. Scope is `adb install`, `force-stop` drills, airplane-mode drills, harness runs, JSON reports.
- Pass criteria per `<APP>/<serial>`: `smoke.sh` (cold-launch alive, crash-free), `kill-drill.sh` (new pid + no `FATAL EXCEPTION` at every FSM stage), `offline-drill.sh` (survives airplane-mode + reconnect; dedup hook `EEL_DEDUP_CHECK` SKIP-not-pass when unset), `test_harness.sh` (iris/fingerprint/printer/modem/scanner/camera/GNSS/NTS/battery/USB/BT/telephony all PASS or documented SKIP with reason, e.g. missing Antiris `.so` on device).
- Evidence inbox validates schema, binds `report.GIT_SHA == tag.SHA` and `report.apk_sha256 == SHA256SUMS`, and rejects mismatches with a machine-readable reason (EN + 中文). Acceptance dashboards live on `spare-3` (internal) — factory sees only its own batch status.

### D8. Chat with the code (scoped, self-hosted, bilingual)

- Corpus is exactly the `eel-factory` tag + its docs (`ENGINEERING_FAQ`, `factory_guidance`, `compile_apks`, kit module pages). ABIS, HSM, tally, governance, and all secrets are never indexed. Redaction runs before embedding (`*.pem`, `HSM_*`, `pass …`, `pgbackrest.conf`, connection strings, `BuildConfig` secrets).
- Stack reuses fleet capacity: embeddings `nomic-embed-text` on `ml`, vectors in Qdrant collection `eel-factory-v1` on `appserver` (`:6333` loopback, accessed over WG), chat via LiteLLM (`ml:4000`) with `qwen3:8b` (strong EN/中文 for factory support), NLLB (`:5500`) for glossary-grade translation of guides. Frontend is Open WebUI on `ml` loopback; public exposure waits on the `ml` TLS remediation (no `:443` today — §Context.3) and stays WG/internal until then.
- Behaviour contract: read-only `search → read file:line` tool, every answer cites `path:line`, "out of scope — contact Datacraft" + logged refusal for anything outside the factory corpus, every Q&A logged (who/when/query/sources) to Loki and retained under the 15-year audit rule. No write, no shell, no artifact signing from the bot.
- SaaS copilots (Copilot Chat, hosted Cody, ChatGPT) are rejected for this corpus: retention/training reuse, subpoena surface, and un-auditable exfiltration violate the sovereignty principle (§Principles.4), even though the algorithms themselves are not secret. If IEBC later approves full open source, that is a separate deliberate publication (scrubbed history, explicit licence), not a side effect of debugging prompts.

### D9. Observability, backup, audit, freeze

- `spare-3` and `linode` factory vhosts join existing fleet monitoring: node_exporter → Prometheus on `mail`, Caddy access logs → Loki, Wazuh agent → `10.8.0.5:1514/1515`, Uptime-Kuma checks for `factory.datacraft.co.ke`, `factory-mirror.lindela.io`, and mirror-lag (`spare-3 tag != linode tag` > 2 h = page). SLOs: tag publish → mirror visible p95 < 2 h; pack download p95 < 10 min for 60 MB APK on CN broadband; evidence POST success > 99.5%.
- Backup: nightly `git bundle --all` + MinIO bucket versioning snapshot from `spare-3` → `auth` MinIO and onward to Linode Object Storage (`gb-lon-1`, same pattern as `pjs-pgbackrest` in `PLATFORM_ACCESS.md`); restore drill monthly; any drill failure blocks the next kit tag (mirrors ADR-0022's "drill failure blocks release").
- Audit: every pull (token, tag, bytes), every evidence POST (schema result, hashes), every chatbot Q&A, and every promotion is appended to the audit store with the standard 5-axis reference + `proofHash` + `signature` + `auditId` (UUIDv7) required of all services.
- Freeze: kit tags obey the 30-day pre-election freeze (ADR-0014/0022). Inside the window only CVE-critical kit fixes ride, with dual sign-off + tabletop note; the CI gate refuses all other merges to the split source.

## Repository structure (factory view)

```text
eel-factory/ @ kit-vX.Y.Z
├── apps/
│   ├── bvi-kit/            # Flutter + Rust plugin (device only)
│   ├── bvr-kit/
│   └── rts-kit/
├── crates/                 # device crates only (explicit allowlist)
├── factory_pack/
│   ├── *.apk               # debug + factory-validation release builds
│   ├── SHA256SUMS  factory_toolchain.txt  SBOM.cdx.json
│   ├── factory_test.sh smoke.sh kill-drill.sh offline-drill.sh test_harness.sh
│   ├── lib/{common,device}.sh
│   ├── antiris/*.jar + lib/{armeabi-v7a,arm64-v8a}/*.so
│   └── *.md (INSTALLER_TESTER_GUIDE, ENGINEERING_FAQ, factory_guidance, compile_apks)
├── docs/kit/               # bvr-device, bvi-device, rts-device module pages only
└── .woodpecker/            # factory-visible subset (build/mobile/test/sign/sbom)
```

What is deliberately absent (and why): `abis-svc/`, `transparency-svc/`, `hsm-signer/`, backend tally, migrations, `infra/`, keys, PII fixtures. Presence would convert a hardware-build compromise into a total-system compromise.

## Hosting specification (build of record)

| Item | Value | Notes |
|---|---|---|
| Authoritative git + evidence | `spare-3`: Forgejo `127.0.0.1:3007`, inbox `:3009`, Caddy `factory.datacraft.co.ke` | 200 GB disk; current use trivial; APKs live in MinIO, not on root disk. |
| Mirror + CDN | `linode`: static `:3008`, Caddy `factory-mirror.lindela.io` | Hourly WG sync; no secrets; Nanode-sized workload. |
| Artifacts | `auth` MinIO bucket `eel-factory`, versioned + object-lock on releases | Presigned URLs only to factory. |
| Identity | Keycloak service accounts `factory-shenzhen-ro`, `factory-evidence-w` | 90-day rotation, IP-pinned. |
| Signing | HSM-anchored release key via OpenBao on `auth` (ADR-0005); CI keyless via SPIFFE (ADR-0006) | Factory holds public keys only. |
| Code Q&A | `ml` Ollama/LiteLLM + `appserver` Qdrant `eel-factory-v1` | Internal until `ml` TLS restored; EN/中文. |
| Monitoring | Prometheus/Grafana/Loki + Wazuh on `mail` | Same enrolment as the other 12 agents. |
| DNS | Linode DNS (per `PLATFORM_ACCESS.md` token flow) | A records for the three factory names, 300 s TTL. |

Estimated marginal cost: $0 new infra (all hosts already paid; total Contabo fleet ~$153/mo). Only cost is engineering time + ~20 GB MinIO growth per retained kit tag.

## Consequences

Positive: factory can work daily over plain HTTPS with zero VPN onboarding; Kenya retains exclusive signing/promotion; blast radius of a factory compromise is bounded to kit-load scope; China-link flakiness is absorbed by vendored locks + UK failover; every load is attributable to a tag, a hash, and a drill report for 15 years; bilingual chatbot deflects the repetitive HAL/signing/ABI questions without leaking backend IP.

Negative: split-repo publishing is ongoing toil (allowlist maintenance, split-job ownership); two-URL mental model for factory operators (mitigated by the download page auto-selecting the faster mirror); `ml` Q&A stays internal until its TLS is fixed, so early chatbot access is Datacraft-mediated; object-locked APK retention grows MinIO usage (budgeted, but must be lifecycle-managed after the election).

## Alternatives rejected

- **Full-monorepo access for factory.** Rejected: violates builder/verifier separation and sovereignty; converts any factory683 compromise into backend/HSM exposure; no voter-trust benefit beyond what the Merkle log + M14 bundle already provide.
- **SaaS git + SaaS runners as authoritative (GitHub/GitLab.com).** Rejected as authoritative: US retention/subpoena surface, weaker 15-year audit story, upstream fetch on the build path. Accepted only as an optional *public* transparency mirror of released tags, after IEBC written approval — never as the signing path.
- **Factory laptops on the WG mesh / SSH accounts.** Rejected: GFW UDP instability, key-sprawl, and privilege far beyond pull+test. HTTPS tokens + presigned URLs cover every legitimate factory action.
- **SaaS code chat (Copilot/Cody/ChatGPT) over the corpus.** Rejected: un-auditable retention and exfiltration of election IP; the self-hosted Qwen3/Qdrant stack answers the same questions within the audit boundary.
- **New Singapore/China-adjacent host now.** Deferred, not rejected: revisit on metrics (§D9 SLOs). DE+UK plus vendoring is sufficient until proven otherwise.

## Rollout

1. **Week 1 — publish path:** Forgejo on `spare-3` (`:3007` + Caddy), `eel-factory` split job, `kit-v1.x` tag, MinIO bucket + presigned download page, UFW/monitoring enrolment, factory tokens issued.
2. **Week 2 — mirror + evidence:** WG sync to `linode` + `factory-mirror` vhost, evidence inbox + schema gate, dashboard, first full batch (all three kits × smoke/kill/offline/harness) with JSON evidence.
3. **Week 3 — Q&A:** Qdrant collection + redacted index of the tag, LiteLLM `qwen3:8b` assistant behind auth (internal first), EN/中文 evaluation on the top-50 factory questions, then scoped factory access.
4. **Pre-election:** freeze gate live, restore drill green, mirror-lag paging proven, 15-year audit export verified.

## Related

- ADR-0004 (K8s/Harbor/Caddy platform), ADR-0005 (crypto/HSM/M-of-N), ADR-0006 (Nix/Woodpecker/Nexus/cosign/SLSA — extended here), ADR-0014 (schema freeze), ADR-0016 (PII tiers — why factory sees no PII), ADR-0017 (transparency log — public trust without backend disclosure), ADR-0022 (release/DR/election-day mode), ADR-0023 (RFP reconciliation), ADR-0024 (kit placement under services).
- `factory_pack/{README,INSTALLER_TESTER_GUIDE,ENGINEERING_FAQ,factory_guidance,compile_apks}.md`, `.woodpecker/{build,mobile,test,sign,sbom}.yaml`, `infra/docs/{INFRASTRUCTURE,NETWORK_TOPOLOGY,PLATFORM_ACCESS,PORT_MAP}.md`.

## Open questions (for IEBC sign-off, not blocking Week 1)

1. Confirm code-offshore scope: `eel-factory` subset on EU hosts approved; voter/biometric data and HSM keys remain Kenya-only — written confirmation.
2. Confirm optional delayed *public* mirror of released kit tags (GitHub/IPFS, post-IEBC review) vs EU-private only.
3. Confirm production MDM signing ceremony participants and whether factory-validation keys must be per-batch or per-release.
