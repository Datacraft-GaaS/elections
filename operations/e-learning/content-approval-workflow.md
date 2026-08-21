# Content approval workflow

## Overview

All training content published to the eel LMS must pass a structured review and be released as an HSM-signed bundle. The signing key is held by the election commission; unsigned bundles are rejected by `training-svc` at ingest.

## Roles

| Role | Responsibility |
|------|---------------|
| Content Author | Drafts modules; submits for review |
| Subject Matter Expert (SME) | Reviews content for legal and procedural accuracy |
| Fact-check Officer | Independent verification of factual claims against official electoral code |
| Content Lead | Final approval before release bundle is created |
| Security Officer | Signs the release bundle with the HSM key |

## Workflow

```
Content Author -> SME Review -> Fact-check -> Content Lead Approval -> Security Officer Signs -> Published
```

### Step 1: draft submission

The Content Author submits a module draft to the LMS staging environment:

```bash
curl -sf -X POST \
  http://training-svc.eel-system.svc.cluster.local:8080/admin/content/draft \
  -H "Content-Type: application/json" \
  -d "{\"module_id\":\"<MODULE_ID>\",\"version\":\"<VERSION>\",\"author\":\"<AUTHOR_ID>\",\"content_url\":\"<STAGING_URL>\"}"
```

### Step 2: SME review

The SME reviews the draft against the electoral code and marks it reviewed:

```bash
curl -sf -X POST \
  http://training-svc.eel-system.svc.cluster.local:8080/admin/content/<MODULE_ID>/review \
  -H "Content-Type: application/json" \
  -d '{"reviewer":"<SME_ID>","outcome":"approved","notes":"<NOTES>"}'
```

### Step 3: fact-check

The Fact-check Officer verifies all factual claims independently:

```bash
curl -sf -X POST \
  http://training-svc.eel-system.svc.cluster.local:8080/admin/content/<MODULE_ID>/factcheck \
  -H "Content-Type: application/json" \
  -d '{"checker":"<CHECKER_ID>","outcome":"approved","notes":"<NOTES>"}'
```

### Step 4: Content Lead approval

The Content Lead approves the module for release:

```bash
curl -sf -X POST \
  http://training-svc.eel-system.svc.cluster.local:8080/admin/content/<MODULE_ID>/approve \
  -H "Content-Type: application/json" \
  -d '{"approver":"<CONTENT_LEAD_ID>"}'
```

### Step 5: create and sign release bundle

The Content Lead triggers bundle creation; the Security Officer signs it with the HSM:

```bash
# create bundle:
curl -sf -X POST \
  http://training-svc.eel-system.svc.cluster.local:8080/admin/content/<MODULE_ID>/bundle \
  -o /tmp/<MODULE_ID>-<VERSION>.tar

# sign with HSM (Security Officer action):
eel-hsm-admin --device /dev/hsm \
  sign --input /tmp/<MODULE_ID>-<VERSION>.tar \
  --output /tmp/<MODULE_ID>-<VERSION>.tar.sig \
  --key-alias eel-training-release

# verify signature:
eel-hsm-admin verify \
  --input /tmp/<MODULE_ID>-<VERSION>.tar \
  --sig /tmp/<MODULE_ID>-<VERSION>.tar.sig \
  --key-alias eel-training-release
```

### Step 6: publish to LMS

```bash
curl -sf -X POST \
  http://training-svc.eel-system.svc.cluster.local:8080/admin/content/<MODULE_ID>/publish \
  -F "bundle=@/tmp/<MODULE_ID>-<VERSION>.tar" \
  -F "signature=@/tmp/<MODULE_ID>-<VERSION>.tar.sig"
# training-svc verifies the signature before accepting the bundle
```

## Emergency content correction (during freeze)

If a factual error is found during the content freeze window, the correction follows the same workflow but requires:

- Dual-officer override to unlock the module for editing.
- Expedited SME review (24-hour window).
- Security Officer re-signature.
- Re-distribution of offline bundles to all counties.

Document the correction in the incident ticket and emit a tlog leaf:

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"training.content.emergency-correction.v1\",\"payload\":{\"module_id\":\"<MODULE_ID>\",\"version\":\"<VERSION>\",\"ticket\":\"<TICKET>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```
