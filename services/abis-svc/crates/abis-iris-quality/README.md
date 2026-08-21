# eel-abis-iris-quality

Iris image quality assessment per ISO/IEC 29794-6:2021.

## Metrics (src/metrics.rs)

- Sharpness: Laplacian variance on iris annulus
- Contrast: 90th-10th percentile histogram spread
- Dilation: pupil/limbus radius ratio (ideal 0.3-0.5)
- Iris/sclera ratio: annulus area fraction
- Gaze angle: pupil offset from limbus centre (degrees)
- Occlusion: fraction of annulus masked

## Policy (src/policy.rs)

`QualityPolicy::evaluate(report, limbus_radius_px)` returns `(Verdict::Accept | Reject, RejectionReasons)`.
Default thresholds: min iris diameter 200 px, dilation 0.2-0.6, occlusion < 30%, gaze < 15 deg.
