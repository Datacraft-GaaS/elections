# IAB Witness Ceremony Runbook

**Service:** `services/transparency-svc/crates/tlog-witness`
**Window:** T-1 through T+7 (election day minus one through plus seven)
**Cadence:** twice daily at 07:00 EAT and 18:00 EAT
**Owner:** Independent Audit Body (IAB) Operations Lead
**Last reviewed:** 2026-08-22

---

## 1. Purpose and Scope

The IAB witness ceremony produces a cryptographically signed attestation of the
transparency log (tlog) root hash for every 24-hour batch window during the
election period. Two IAB officers independently verify the root hash and sign it
with hardware-backed smartcard keys before the attestation is submitted to a
public Rekor instance. Any third party holding the IAB public key registry can
verify the ceremony output without trusting any eel platform service.

**What the ceremony signs.** The `tlog-witness` service aggregates Merkle leaves
from `ballot-svc`, `rts-svc`, and `kit-svc` into a 24-hour batch and exposes
the resulting tree root hash at `/v1/witness/root`. The ceremony signs this hash.

**Why it matters.** An unwitnessed root hash is self-asserted by the platform.
Dual-officer, hardware-backed signing converts platform assertions into
externally verifiable facts. Rekor provides immutable timestamping, satisfying
RFC 9162 section 5 and ADR-0005 (M-of-N ceremony).

---

## 2. Prerequisites

Confirm all items before the 06:00 shift briefing.

**Personnel**

- Five IAB smartcard operators confirmed on the shift roster.
- At least two officers present for each ceremony window.
- HSM operator on shift. PKCS#11 PIN held in escrow under two-party control.
  Retrieve the escrow envelope from the IAB safe and sign the custody log before 06:30.

**Service health**

```bash
curl -sf http://tlog-witness.eel-svc:8080/healthz | jq .
# Expected: {"status":"ok","batch_sealed":true}
```

A `batch_sealed: false` response means the 24-hour batch has not closed yet.
Wait until after 07:00 EAT and retry.

**Tooling**

```bash
cosign version | grep -E '^GitVersion'  # must be >= 2.4
export REKOR_URL=https://rekor.sigstore.dev
export PKCS11_URI="pkcs11:token=eel-iab-hsm;object=eel-iab-signing-key"
curl -sf $REKOR_URL/api/v1/log | jq .treeSize  # confirm network reachability
```

**Network.** Ceremony workstation must be on the isolated election VLAN (VLAN 200).

---

## 3. 24-Hour Ceremony Cadence

| Time (EAT) | Activity |
|------------|----------|
| 06:00 | Shift briefing. Verify roster, retrieve HSM escrow envelope, confirm tooling. |
| 06:30 | HSM operator opens escrow under witness of two officers. PIN entered. Custody log signed. |
| 07:00 | Morning ceremony window opens. Batch sealed for previous 24-hour window. |
| 07:00-08:00 | Run ceremony script (section 5). Root hash captured. |
| 08:00 | Dual-witness sign-off (section 4). Both officers sign. |
| 08:30 | Verification checklist (section 7) completed and signed. |
| 09:00 | Ceremony log uploaded to CRO portal. Rekor UUIDs recorded. |
| 18:00 | Evening ceremony window opens. Repeat 07:00-09:00 sequence. |
| 20:00 | HSM PIN re-sealed in escrow envelope. Signed back into IAB safe. |

---

## 4. Dual-Witness Sign-Off Procedure

1. Officer 1 retrieves `/tmp/eel-tlog-root-<timestamp>.json` and reads the
   `root_hash` field aloud. Officer 2 independently fetches the same value from
   the tlog-witness API and confirms a verbal match.
2. Officer 1 authenticates with their smartcard (PIN plus physical token) and
   runs the sign-blob command (step d). Output: `/tmp/eel-sig-officer1.sig`.
3. Officer 2 takes over the keyboard. Officer 1 steps back. Officer 2
   authenticates and runs the sign-blob command (step e). Output:
   `/tmp/eel-sig-officer2.sig`.
4. Neither officer may observe the other's PIN entry.
5. The ceremony workstation operator (a third person) posts the attestation
   payload to `/v1/witness/attest`. The payload includes both signatures, both
   officer IDs, the root hash, and an ISO 8601 timestamp.

The witness log entry records: `officer1_id`, `officer2_id`, `root_hash` (hex
SHA-256), `timestamp` (RFC 3339), `sig1`, `sig2`, `rekor_entry_uuid`.

---

## 5. Ceremony Script

Record every command exit code in the ceremony log.

**a. Health check**

```bash
curl -sf http://tlog-witness.eel-svc:8080/healthz | jq .
```

**b. Fetch tlog root hash**

```bash
TIMESTAMP=$(date +%Y%m%d%H%M%S)
ROOT_FILE="/tmp/eel-tlog-root-${TIMESTAMP}.json"
curl -sf http://tlog-witness.eel-svc:8080/v1/witness/root | tee "$ROOT_FILE"
```

**c. Verify root hash matches Rekor**

```bash
REKOR_HASH=$(curl -sf "$REKOR_URL/api/v1/log" | jq -r '.rootHash')
LOCAL_HASH=$(jq -r '.root_hash' "$ROOT_FILE")
[ "$REKOR_HASH" = "$LOCAL_HASH" ] && echo "PASS" || { echo "FAIL mismatch"; exit 1; }
```

**d. Officer 1 signs**

```bash
COSIGN_PKCS11_TOKEN_URI=$PKCS11_URI cosign sign-blob \
  --key "$PKCS11_URI" "$ROOT_FILE" \
  --output-signature /tmp/eel-sig-officer1.sig
```

**e. Officer 2 signs**

```bash
COSIGN_PKCS11_TOKEN_URI=$PKCS11_URI cosign sign-blob \
  --key "$PKCS11_URI" "$ROOT_FILE" \
  --output-signature /tmp/eel-sig-officer2.sig
```

**f. Submit witness attestation**

```bash
SIG1=$(base64 -w0 /tmp/eel-sig-officer1.sig)
SIG2=$(base64 -w0 /tmp/eel-sig-officer2.sig)
ROOT_HASH=$(jq -r '.root_hash' "$ROOT_FILE")

curl -sf -X POST http://tlog-witness.eel-svc:8080/v1/witness/attest \
  -H "Content-Type: application/json" \
  -d "{\"root_hash\":\"$ROOT_HASH\",\"officer1_id\":\"$OFFICER1_ID\",
       \"officer2_id\":\"$OFFICER2_ID\",\"sig1\":\"$SIG1\",\"sig2\":\"$SIG2\",
       \"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" \
  | tee /tmp/eel-attest-response.json
```

**g. Publish to Rekor**

```bash
cosign upload blob --rekor-url "$REKOR_URL" \
  --signature /tmp/eel-sig-officer1.sig "$ROOT_FILE" | tee /tmp/eel-rekor-officer1.txt
cosign upload blob --rekor-url "$REKOR_URL" \
  --signature /tmp/eel-sig-officer2.sig "$ROOT_FILE" | tee /tmp/eel-rekor-officer2.txt
```

**h. Verify Rekor entries**

```bash
UUID1=$(grep -oE '[0-9a-f]{64,}' /tmp/eel-rekor-officer1.txt | head -1)
UUID2=$(grep -oE '[0-9a-f]{64,}' /tmp/eel-rekor-officer2.txt | head -1)
rekor-cli get --rekor_server "$REKOR_URL" --uuid "$UUID1" | jq .verification
rekor-cli get --rekor_server "$REKOR_URL" --uuid "$UUID2" | jq .verification
echo "UUIDs: $UUID1  $UUID2"  # record both in ceremony log
```

---

## 6. tlog Cosign Submission

Each Rekor entry contains the root hash JSON and one officer's signature. Two
entries are created per window, one per officer. Archive these files to
`/mnt/eel-ceremony-archive/<date>/` after each ceremony:

```
eel-tlog-root-<timestamp>.json
eel-sig-officer1.sig  eel-sig-officer2.sig
eel-attest-response.json
eel-rekor-officer1.txt  eel-rekor-officer2.txt
ceremony-log-<date>-<session>.pdf
```

Any external auditor can verify independently:

```bash
rekor-cli verify --rekor_server "$REKOR_URL" \
  --artifact eel-tlog-root-<timestamp>.json \
  --signature eel-sig-officer1.sig \
  --key eel-iab-officer1-pub.pem
```

---

## 7. Verification Checklist

Both officers sign the printed checklist after each ceremony window.

- [ ] `/healthz` returned `{"status":"ok","batch_sealed":true}`
- [ ] Root hash matches Rekor latest checkpoint (step c: PASS)
- [ ] Officer 1 signature verified against IAB public key registry
- [ ] Officer 2 signature verified against IAB public key registry
- [ ] Officer 1 Rekor entry UUID recorded in ceremony log
- [ ] Officer 2 Rekor entry UUID recorded in ceremony log
- [ ] `/v1/witness/attest` returned HTTP 200 with attestation ID
- [ ] Ceremony artifacts archived to `/mnt/eel-ceremony-archive/<date>/`
- [ ] No HSM errors in `journalctl -u pkcs11-daemon` since 06:00
- [ ] Ceremony log PDF uploaded to CRO portal, confirmation number recorded
- [ ] Next ceremony window confirmed on shift roster

---

## 8. Escalation Matrix

| Condition | Immediate action | Escalation contact | SLA |
|-----------|------------------|--------------------|-----|
| `tlog-witness /healthz` non-200 | Stop ceremony. Follow `witness-divergence-response.md`. | IAB Ops Lead, then Platform On-Call | 15 min to restore or escalate |
| HSM PKCS#11 error during sign-blob | Abort. Do not retry PIN more than twice. | HSM Operator, then IAB Security Officer | 30 min |
| Smartcard auth failure for one officer | Substitute next officer on roster. Document in ceremony log. | IAB Ops Lead | Proceed if substitute available within 30 min |
| Rekor submission timeout | Retry once after 5 min. Store artifacts locally if still failing. | Platform On-Call, Rekor admin | 1 hour or switch to fallback Rekor instance |
| Root hash mismatch (step c FAIL) | Halt. Do not sign. Preserve all artifacts. | IAB Ops Lead, Platform On-Call, CRO | Immediate, P0 incident |
| Fewer than 2 IAB officers available | Do not proceed. Document in shift log. | IAB Ops Lead, CRO | Report to CRO within 1 hour |

---

## 9. References

- **Source:** `services/transparency-svc/crates/tlog-witness`
- **ADR-0005:** M-of-N Ceremony Design for tlog Dual Witnessing
- **RFC 9162:** Certificate Transparency Version 2.0
- **Runbooks:** `hsm-key-ceremony.md`, `witness-divergence-response.md`,
  `transparency-log-witness-cross-check.md`
- **IAB public key registry:** `services/transparency-svc/config/iab-pubkeys/`
- **CRO portal:** `docs/integrations/cro-portal.md`
