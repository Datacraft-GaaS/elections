# eel-abis-fp-cdeffs

CDEFFS Extended Feature Set for ANSI/NIST-ITL Type-9 fingerprint records.

## Reference

NIST SP 500-290 e3 Annex A, "Candidate Definition of Extended Feature Set for
Fingerprints (CDEFFS)"; ANSI/NIST-ITL 1-2011 Type-9 fields 300-999.

## Features Covered

| Field | Description |
|-------|-------------|
| 9.300 | Level-1 pattern class (Arch, Loop, Whorl, ...) |
| 9.301 | Singular points (cores + deltas) with position + orientation |
| 9.302 | Ridge quality map (per-pixel 0-100) |
| 9.303 | Ridge flow / orientation map (per-pixel 0-179 degrees) |
| 9.304 | Ridge wavelength map (inter-ridge spacing) |
| 9.305 | Segmentation mask (foreground/background) |
| 9.320 | Minutia extended attributes (ridge counts, feature type, dot/incipient flags) |
| 9.330 | Isolated ridge dots |
| 9.331 | Incipient ridges (nascent ridges between mature ones) |
| 9.350 | Creases, scars, warts, skin conditions |
| 9.360 | Ridge edge shapes (curvature + width per minutia) |

## Usage

```rust
use eel_abis_fp_cdeffs::{parse_efs, serialise_efs, CdeffsMatcher};

let efs = parse_efs(&type9_bytes)?;
let matcher = CdeffsMatcher::new();
let fused_score = matcher.fuse(base_minutiae_score, &probe_efs, &gallery_efs);
```
