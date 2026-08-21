# eel-abis-face-embed

Face embedding for the EEL ABIS pipeline.

Defines `FaceEmbedder` trait for ArcFace/AdaFace/CosFace 512-d L2-normalised
embeddings. Provides a hash-based deterministic baseline so CI passes without
ONNX model files.

## References

- Deng et al. (2019). ArcFace. CVPR 2019.
- Kim et al. (2022). AdaFace. CVPR 2022.
- Wang et al. (2018). CosFace. CVPR 2018.
