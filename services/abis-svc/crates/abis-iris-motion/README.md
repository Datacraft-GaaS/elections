# eel-abis-iris-motion

Motion analysis for iris video capture: blur estimation, multi-frame stacking, and identity tracking.

## Modules

- `blur`: Laplacian-variance motion blur estimator with configurable acceptance threshold.
- `stack`: Mean-averaging frame stacker to improve SNR across multiple captured frames.
- `tracker`: Frame-to-frame iris identity consistency tracker with configurable drift threshold.
