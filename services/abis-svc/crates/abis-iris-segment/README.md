# eel-abis-iris-segment

Iris boundary segmentation using the Daugman integro-differential operator (Daugman 1993) with
OSIRIS Viterbi-based boundary refinement (Othman et al. 2016).

## Algorithms

- **Daugman IDO** (`daugman.rs`): coarse circle search by maximising annular derivative
- **OSIRIS Viterbi** (`osiris.rs`): per-angle radial refinement via gradient maximisation
- **Masking** (`mask.rs`): eyelid parabola fit, eyelash detection, specular reflection removal
- **Pipeline** (`segment.rs`): `SegmentIris::segment(image)` returns `IrisSegmentation`

## Types exported

- `GrayImage` -- 8-bit row-major grayscale image
- `Circle` -- centre + radius
- `IrisMask` -- per-pixel validity bitmap
- `IrisSegmentation` -- pupil + limbus + mask + iris_sclera_ratio
