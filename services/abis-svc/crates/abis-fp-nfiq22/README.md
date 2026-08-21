# eel-abis-fp-nfiq22

NFIQ 2.2 fingerprint image quality assessment for the eel ABIS pipeline.

## Reference

NIST Special Publication 500-333, "NFIQ 2.2 Fingerprint Image Quality,"
F. Schwaiger, T. Schiller, J. Merkle, C. Busch, 2021.
<https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.500-333.pdf>

## Features extracted (Section 4)

| # | Name | Description |
|---|------|-------------|
| 1 | LCS | Local Clarity Score |
| 2 | OCL | Orientation Certainty Level |
| 3 | OFL | Orientation Flow |
| 4 | ROI | Region of Interest area |
| 5 | FDA | Frequency Domain Analysis |
| 6 | Gabor | Gabor filter response |
| 7 | MU | Mean gray value |
| 8 | MMB | Mean of Minimum Block |
| 9 | RVU | Ridge-Valley Uniformity |
| 10 | S | Sigma of foreground |
| 11 | FingerJetFX_OSE | Minutia count quality proxy |
| 12 | GSHEAR | Mean gray value shear |
| 13 | RVFL | Ridge-Valley Frequency and Local Clarity |
| 14 | PIX | Pixel neighbour standard deviation |

## Runtime calibration

Load the NIST-published Random Forest model (JSON) at runtime:

```rust
let model = RandomForestModel::from_path(Path::new("/path/to/nfiq22_model.json"))?;
let score = score_with_model(&image, &model)?;
```

Recalibrate for local population via `Calibrator::fit`.
