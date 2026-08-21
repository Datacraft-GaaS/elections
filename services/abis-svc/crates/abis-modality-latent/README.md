# eel-abis-modality-latent

Forensic latent fingerprint modality crate for the EEL ABIS stack.

## Standard

ANSI/NIST-ITL 1-2011 Type-13 latent image record. The parser handles a simplified binary encoding of the Type-13 fields: magic header [0x41, 0x4E, 0x13], record length, IDC, impression type, image dimensions, minutiae count, and 6-byte minutia records.

## Matcher Algorithm

Minutiae subset matching for partial prints:

- Score = (matched_minutiae / probe_count) * usable_ridge_pct.
- Rotation tolerance: +/- 30 degrees, translation: +/- 80 px (same as palm).
- Partial-print specific: score is normalized against probe count (never gallery), favouring high-quality partials.
- Default threshold: 0.35 (lower than 10-print due to partial nature).

## Quality Scorer

Quality = usable_ridge_pct * (1.0 - distortion_score) * density_factor, scaled to 0-100. A distortion_score of 0.0 and full ridge coverage gives maximum quality.

## Chain of Custody

`append_custody_event(template, event)` pushes a `CustodyEvent` with actor_id, action, timestamp, and optional base64 signature onto the template's chain_of_custody vec. Events preserve insertion order.

## Example Usage

```rust
use eel_abis_modality_latent::{parser, matcher};

let mut template = parser::parse_nist_type13(&bytes)?;
parser::append_custody_event(&mut template, CustodyEvent {
    event_id: uuid,
    actor_id: examiner,
    action: "lifted".into(),
    ..
});
let score = matcher::match_latent(&probe, &gallery);
```
