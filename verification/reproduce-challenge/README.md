# Eel Reproduce-Challenge Bounty Programme

## Purpose

The reproduce-challenge programme invites independent third-party teams to replicate the official election tally from raw station data and submit their results nightly. An automated pipeline diffs each submission against the reference bundle and publishes results on a public static page. This creates a continuous, externally verifiable integrity signal for the eel election platform.

If your independently computed tallies match ours every night, voters and observers gain confidence that the numbers are not being manipulated between ingestion and publication.

## Eligibility

- Any team or individual with verifiable independence from the election management body.
- Academic institutions, civil society organisations, accredited observer groups, and independent audit firms are all welcome.
- Registered participants receive a per-team Bearer token; see Contact below to register.
- No registration fee. Participation is voluntary and non-binding.

## How It Works

1. The reference bundle is published to a public S3 path at 23:00 UTC each night.
2. Registered teams receive a webhook notification with the bundle URL and the verification day.
3. Teams have 8 hours to download the bundle, reproduce the tally independently, and submit via the HTTP endpoint documented in `submit-endpoint.md`.
4. The automated verifier (`verify-submission.sh`) diffs each submission against the reference and generates a signed report.
5. Results are published to the public Astro static page by 08:00 EAT each morning.

## Submission Format

Submit a `submission.tar.gz` containing:

```
manifest.json          # required; lists every file with its SHA-256
tally.json             # your independently computed tally
events.ndjson          # the CloudEvents log your pipeline consumed
```

`manifest.json` schema:

```json
{
  "files": [
    {"path": "tally.json",    "sha256": "<hex>"},
    {"path": "events.ndjson", "sha256": "<hex>"}
  ],
  "day":     "YYYY-MM-DD",
  "team_id": "your-team-id"
}
```

All paths are relative to the tarball root. Extra files are permitted but must be listed in the manifest. Files listed in the manifest but absent from the tarball cause an immediate rejection.

## Scoring

| Outcome | Points |
|---|---|
| PASS: submission matches reference exactly | +10 |
| PARTIAL: diff present but manifest intact | +2 |
| FAIL: manifest missing or SHA mismatch | 0 |
| No submission | 0 |

A cumulative leaderboard is updated nightly. Teams with 30+ consecutive PASS nights receive a "Verified Reproducer" badge on the public results page.

## Timeline

| Event | Time (EAT) |
|---|---|
| Reference bundle published | 02:00 |
| Webhook notifications sent | 02:05 |
| Submission window opens | 02:05 |
| Submission window closes | 10:00 |
| Verification run completes | 10:30 |
| Results published | 11:00 |

## Contact

To register your team or report an issue, open a GitHub issue tagged `reproduce-challenge` in the eel repository, or email the programme coordinator at the address listed in the project's `SECURITY.md`.

All programme communications use PGP-signed messages. The programme public key fingerprint is published in `docs/pgp-keys.md`.
