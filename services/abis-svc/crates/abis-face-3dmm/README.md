# eel-abis-face-3dmm

3D Morphable Model interface for the EEL ABIS pipeline.

Implements the `Model3dmm` trait with a deterministic stub for CI. Provides
BFM (85 shape + 64 expression PCA coefficients), pose-normalisation to frontal,
and a geometric pose-consistency spoof detection stub.

## References

- Blanz & Vetter (1999). A morphable model for the synthesis of 3D faces. SIGGRAPH.
- Paysan et al. (2009). A 3D Face Model for Pose and Illumination Invariant Face
  Recognition. AVSS.
- Deng et al. (2019). Accurate 3D Face Reconstruction with Weakly-Supervised Learning.
  CVPRW.
- Boulkenafet et al. (2017). Face Anti-Spoofing Using Texture and Depth. ICCV Workshop.
