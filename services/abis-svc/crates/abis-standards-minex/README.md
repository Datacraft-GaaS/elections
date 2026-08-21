# eel-abis-standards-minex

Rust implementation of NIST MINEX III fingerprint template format (NIST SP 500-304), including template construction, FMR serialization, conformance validation, and cross-vendor interoperability scoring.

## Overview

MINEX III (Minutiae Interoperability Exchange) is NIST's ongoing evaluation of fingerprint template interoperability. Templates must conform to ISO/IEC 19794-2 FMR (Finger Minutiae Record) format and meet specific quality and conformance criteria to be submitted to the evaluation program. This crate provides a conformant FMR encoder/decoder plus a local interop score for pre-submission testing.

## Template Format

Templates follow the ISO 19794-2 FMR layout:

```
Offset  Size  Field
0       4     Magic: "FMR\x00"
4       4     Version: "20\x00\x00"
8       4     Record length (u32 BE)
12      1     Finger position (1-10)
13      1     View number
14      1     Impression type
15      2     Minutiae count (u16 BE)
17+     6*N   Minutiae records
```

Each minutia is 6 bytes:

```
Bytes 0-1: type(2 bits) | x(14 bits)  -- big-endian u16
Bytes 2-3: reserved(2 bits) | y(14 bits) -- big-endian u16
Byte  4:   angle (0-255, units of 1.40625 deg)
Byte  5:   quality (0-100)
```

Minutia types: 0 = other, 1 = ridge termination, 2 = bifurcation.

## MINEX III Conformance Criteria

| Criterion | Limit |
|---|---|
| Maximum minutiae count | 128 |
| Image dimensions | 500 x 500 pixels |
| Image resolution | 500 ppi |
| Coordinate range | [0, 499] on both axes |
| Quality range | [0, 100] |
| Finger position | 1-10 (ANSI/NIST-ITL coding) |

## Usage

```rust
use abis_standards_minex::template::{MinexTemplate, MinexMinutia};
use abis_standards_minex::validator::MinexValidator;
use abis_standards_minex::interop::compute_interop_score;

let mut tmpl = MinexTemplate::new(2); // right index finger
tmpl.add_minutia(MinexMinutia {
    x: 120, y: 200, angle: 64, quality: 85, mtype: 1,
})?;

// Serialize to FMR wire bytes
let bytes = tmpl.serialize();

// Parse back
let parsed = MinexTemplate::parse(&bytes)?;

// Validate conformance
let result = MinexValidator::validate(&parsed);
assert!(result.passes);
println!("quality: {:.1}", result.estimated_quality);

// Score against a gallery template
let score = compute_interop_score(&probe_tmpl, &gallery_tmpl);
println!("overlap: {}, score: {:.3}", score.minutiae_overlap, score.score);
```

## Conformance Validation

`MinexValidator::validate` checks all MINEX III criteria and returns a `ConformanceResult`:

```rust
pub struct ConformanceResult {
    pub passes: bool,
    pub violations: Vec<String>,
    pub minutiae_count: usize,
    pub estimated_quality: f32,  // mean quality across all minutiae
}
```

Estimated quality is the arithmetic mean of per-minutia quality scores.

## Interoperability Score

`compute_interop_score` implements the MINEX III matching protocol:

- Two minutiae match if Euclidean distance <= 15 pixels AND angle difference <= 22.5 degrees (16 units in 256-unit circle)
- Each gallery minutia is used at most once (greedy matching)
- Score = matched pairs / max(probe count, gallery count)
- Score of 1.0 means perfect overlap; 0.0 means no matches

This score is a local proxy for the black-box MINEX III evaluation. Actual NIST evaluation uses proprietary matchers; use this for pre-submission regression testing only.

## Error Handling

- `InvalidHeader(String)` - bad magic bytes or truncated header
- `InvalidMinutiaeCount { found, max }` - exceeds 128 minutiae limit
- `InvalidCoordinate { x, y }` - coordinate outside 500x500 image area
- `InvalidQuality(u8)` - quality above 100
- `ChecksumFailed` - optional integrity check failure

## Test Vectors

The test suite includes round-trip serialization, boundary condition checks, and interop scoring on known cases (identical templates score 1.0, spatially separated templates score 0.0).

## References

- NIST SP 500-304: MINEX III Performance of Fingerprint Algorithms
- ISO/IEC 19794-2: Biometric Data Interchange Formats - Finger Minutiae Data
- ANSI/NIST-ITL 1-2011: Data Format for the Interchange of Fingerprint Data
- NIST MINEX III evaluation: https://www.nist.gov/programs-projects/minex-iii
