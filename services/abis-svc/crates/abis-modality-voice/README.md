# eel-abis-modality-voice

Speaker verification modality crate for the EEL ABIS stack.

## Standard

ISO/IEC 19794-13:2018 (Voice Data) informing the domain model. WAV/PCM ingest with MFCC feature extraction (13 coefficients, 25 ms Hamming window, 10 ms step, 26 mel filterbank channels).

The FFT is a self-contained Cooley-Tukey radix-2 DIT implementation with no external dependencies.

## Matcher Algorithms

Two modes are available:

1. Cosine similarity on embeddings (text-independent, i-vector style):
   - `match_voice_cosine(probe, gallery)` computes dot(a,b) / (|a| * |b|).
   - A `VendorAdapter` trait allows injecting vendor SDK embeddings.

2. Dynamic Time Warping on MFCC frames (text-dependent):
   - `match_voice_dtw(probe, gallery)` aligns frame sequences using DTW, normalises by path length, and converts to similarity via exp(-dtw/50).

Default threshold: 0.55 (cosine), 0.55 (DTW).

## Quality Scorer

Returns a `QualityReport` with:

- `snr_score`: clamped (0-40 dB) mapped linearly to 0-100.
- `duration_score`: optimal 2-10 seconds (4 seconds = 100).
- `speech_activity_ratio`: fraction of frames with RMS above 500.0.
- `overall`: 0.4 * snr + 0.3 * duration + 0.3 * activity, scaled 0-100.

## Example Usage

```rust
use eel_abis_modality_voice::{parser, matcher};

let header = parser::parse_wav_header(&wav_bytes)?;
let template = parser::parse_wav(&wav_bytes)?;
let score = matcher::match_voice_cosine(&probe, &gallery);
```
