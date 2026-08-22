# Independent Assurance Group (IAG) Charter

> Version 1.0. Approved by the Transparency Oversight Board. Separate from and independent of the
> Independent Audit Board (IAB). The IAG reproduces election-system outputs independently and
> publishes its findings without review or approval by any government body.

## 1. Purpose

The Independent Assurance Group exists to provide an externally verified, publicly auditable
confirmation that the eel platform produces the same tally, audit-log hash chain, and
reproducibility outputs that any member of the public could derive from the published artefacts.

Concretely, the IAG:

- Runs a full `make reproduce` against each official publication checkpoint during the election
  window (T-1 to T+7).
- Diffs its local output against the official Signed Tree Head (STH) published by the transparency
  log (`services/transparency-svc/crates/tlog-witness`).
- Publishes the diff and a signed attestation within four hours of each run.
- Reports any discrepancy to the public, to the IAB, and to independent media simultaneously,
  without prior notice to any electoral commission or government body.

The IAG does not adjudicate disputes. It does not have access to sealed biometric templates or
pre-publication tallies. Its role is evidence-preservation and independent verification, not
decision-making.

## 2. Composition

The IAG has five seats (minimum three active to constitute a quorum for any signed attestation):

| Seat | Background required | Appointing body |
|------|--------------------|-----------------------|
| 1 | Academic computer science or cryptography, active publication record | Consortium of three public universities nominated by senate education committee |
| 2 | Open-source community, maintainer of a widely deployed cryptographic or distributed-systems project | Open-source foundation (e.g., Linux Foundation affiliate, Apache Foundation) |
| 3 | Press oversight, accredited journalist with a track record of technical reporting on elections or security | Press freedom organisation independent of government |
| 4 | Legal, admitted advocate with election law or privacy law specialism | Bar association's technology law committee |
| 5 | International observer, nominated by a foreign electoral management body or a multilateral organisation | International IDEA or equivalent body |

No seat may be occupied by a current or recent (within three years) employee or contractor of:

- The electoral commission.
- Any vendor supplying hardware or software to the election system.
- Any political party registered in the jurisdiction.
- Any government ministry.

Each member must sign and publish a conflict-of-interest declaration within 14 days of appointment
and annually thereafter. Declarations are stored as entries in the transparency log.

## 3. Operating Cadence

### Pre-election (T-30 to T-2)

- Members complete Nix flake pin verification: `nix flake check` against the tagged release.
- Each member independently runs `make reproduce` in a sandboxed environment and signs the output.
- Members publish their reproduce hashes to the IAG mailing list and to the transparency log.
- Any divergence triggers an immediate public notice.

### Election window (T-1 through T+7)

- The IAG runs reproduce every 24 hours at a randomised offset from the official publication time
  (to prevent synchronisation attacks).
- Each run produces a signed attestation (Ed25519) committed to the transparency log.
- Members rotate primary responsibility daily so no single member is a single point of failure.
- If a run is missed for any reason, the reason is published within two hours of the scheduled time.

### Post-election (T+8 to T+30)

- The IAG publishes a consolidated final report covering all runs during the election window.
- The report includes a diff summary, any anomalies, and recommendations.
- The report is submitted to the parliamentary oversight committee and published on the IAG public
  website.

## 4. Technical Protocol

### Environment setup

```bash
# Clone the official reproducibility repository (read-only mirror, no electoral-commission access)
git clone https://gitea.internal/eel/eel-reproducibility iag-workspace
cd iag-workspace

# Pin to the exact Nix flake lock from the official release tag
git checkout tags/election-2027-v1.0.0
nix flake check

# Verify the flake lock hash matches the transparency-log entry
cat flake.lock | sha256sum
```

### Reproduce run

```bash
# Full reproduce: pulls published artefacts, reruns tally derivation, emits output STH
make reproduce ELECTION_TAG=election-2027-v1.0.0

# Compare against official STH
diff reproduce-output/sth.json official/sth.json
```

The `make reproduce` target:

1. Fetches the published ballot-leaf Merkle root from the transparency log.
2. Re-derives the tally from published 34A leaf hashes (no raw templates accessed).
3. Recomputes the STH using `services/transparency-svc/crates/tlog-witness` primitives.
4. Emits `reproduce-output/sth.json` with root hash, tree size, and timestamp.

### Signing an attestation

Each IAG member signs the reproduce output with their personal Ed25519 key registered in the
transparency log at appointment time:

```bash
eel-iag-sign \
  --key ~/.iag/ed25519.pem \
  --input reproduce-output/sth.json \
  --output reproduce-output/attestation-<member-id>.json
```

The attestation JSON includes:

```json
{
  "iag_version": "1.0",
  "election_tag": "election-2027-v1.0.0",
  "run_ts": "<ISO-8601>",
  "sth_root": "<hex>",
  "tree_size": 12345,
  "official_sth_root": "<hex>",
  "match": true,
  "member_id": "<uuid7>",
  "signature": "<base64-ed25519>"
}
```

### Submitting to the transparency log

```bash
eel-tlog-append \
  --log-endpoint https://tlog.eel.internal/v1 \
  --entry reproduce-output/attestation-<member-id>.json
```

The append returns a Merkle inclusion proof which the member archives locally and publishes.

## 5. Publication Channels

Every IAG attestation and every discrepancy notice is published through all of the following
simultaneously. No channel has priority over another.

1. **Transparency log Merkle entry** in `services/transparency-svc/crates/tlog-witness`. The STH
   after appending the attestation is itself published as a new tlog entry, creating a chain of
   custody.

2. **IAG public website** (static site, hosted on infrastructure independent of the electoral
   commission). Each attestation is a HTTPS-accessible JSON file with an ETag matching the Merkle
   leaf hash.

3. **Independent press release** to a pre-registered list of accredited journalists. The list is
   maintained by the press-oversight seat holder and is not controlled by any government body.

4. **IAB signed comment** submitted to the IAB inbox within one hour. The IAG makes no request for
   acknowledgement; this is notification only. The IAB is free to respond publicly.

5. **Parliamentary committee registry** for formal post-election reports (T+8 onwards), submitted
   as a structured PDF with a Merkle inclusion proof cover page.

## 6. Governance

### Rotation

- Seats rotate on a five-year cycle, staggered so no more than two seats change in any single year.
- Outgoing members transfer signing keys according to the key-handover ceremony documented in
  `docs/governance/iag-key-handover.md`.
- Members may not serve more than two consecutive terms (10 years total) in any seat.

### Conflict-of-interest disclosure

- Any member who becomes aware of a potential conflict of interest during their term must notify the
  other members within 48 hours.
- If three or more members agree a conflict exists, the affected member recuses from the current
  run and the disclosure is published.
- Recusal does not invalidate a run provided quorum is maintained.

### Immunity for good-faith reporting

- Members acting in good faith to report discrepancies are granted immunity from civil proceedings
  by any party arising from that report, subject to the statutory framework enacted by the
  legislature at the time of this charter.
- Good faith is established by publishing through the official channels defined in section 5, not
  making personal accusations, and providing the raw reproduce output as evidence.
- The IAG legal seat holder maintains a standing retainer with independent counsel to advise
  members on immunity scope.

### Decisions

- Quorum: three of five members (or three of however many seats are filled if seats are vacant).
- Routine attestation signing: majority of quorum.
- Public discrepancy notice: single member with written justification, notified to all other
  members before publication. Any member may append a dissent within two hours.
- Charter amendments: four of five seats plus public comment period of 30 days.

## 7. Accountability

### Public transparency

- All IAG attestations, conflict-of-interest declarations, key registrations, and meeting minutes
  are published to the transparency log and to the IAG public website without redaction.
- IAG members do not have the ability to delete or amend a published entry. The Merkle structure
  of `services/transparency-svc/crates/tlog-witness` makes this computationally infeasible.

### Parliamentary oversight

- The parliamentary committee with oversight of electoral integrity may summon any IAG member to
  give evidence. Members will cooperate subject to legal privilege.
- The committee may commission an independent review of any IAG run at any time. The IAG must
  provide all reproduce artefacts within 72 hours of such a request.
- The committee has no authority to suppress an IAG publication or to delay a discrepancy notice.

### International observer reporting

- The international observer seat holder submits a copy of each attestation to the nominating body
  (International IDEA or equivalent) within 24 hours.
- The nominating body may publish its own commentary independently.

## 8. Termination Criteria

The IAG is dissolved only if:

1. The legislature passes an act dissolving it with a two-thirds majority and providing an
   alternative independent verification mechanism with equivalent or stronger guarantees.
2. The transparency log (`services/transparency-svc/crates/tlog-witness`) is permanently
   decommissioned and no successor system exists.
3. Fewer than three seats can be filled after 90 days of active recruitment.

Dissolution does not affect the validity of previously published attestations or their Merkle
inclusion proofs, which remain verifiable for as long as the transparency log data is publicly
archived.

## 9. Budget and Operational Cost Model

### Funding structure

The IAG is funded by a statutory levy on the electoral commission budget, ring-fenced so that the
commission has no discretion over whether to pay it. The levy amount is set by the parliamentary
committee and reviewed every five years.

Estimated annual operational costs (2027 baseline):

| Category | Estimated cost (USD) |
|----------|---------------------|
| Member stipends (5 seats x USD 8 000/year) | 40 000 |
| Cloud infrastructure (reproduce runners, static site) | 12 000 |
| Legal retainer (standing counsel) | 18 000 |
| Key management hardware (HSM tokens per member) | 5 000 |
| Travel (annual in-person key ceremony) | 15 000 |
| Contingency (15%) | 13 500 |
| **Total** | **103 500** |

All expenditure is published quarterly in the transparency log. No member may receive payment from
any other election-related source while serving.

### Infrastructure independence

The IAG runs its reproduce infrastructure on cloud providers that have no commercial relationship
with the electoral commission or with any vendor in the eel supply chain. The provider selection
is reviewed annually by the open-source community seat holder.

## 10. Technical Integration Point

The IAG interacts with the platform exclusively through published read-only interfaces:

- **`services/transparency-svc/crates/tlog-witness`**: append IAG attestation entries; verify
  consistency proofs between successive STHs; fetch inclusion proofs for audit.
- **Published Merkle roots**: fetched from the public-facing transparency log API.
- **`make reproduce`**: the reproducibility target defined in the top-level `Makefile`, pinned to a
  specific Nix flake lock. No special credentials required.

The IAG has no write access to any electoral-commission system, no access to sealed biometric
templates, and no access to pre-publication tallies. All inputs to the reproduce run are publicly
available artefacts.

---

*This charter is itself a transparency-log entry. Its Merkle inclusion proof is recorded at
appointment of the first IAG cohort.*
