# eel-abis-face-detect

Face detection for the EEL ABIS pipeline.

Provides the `FaceDetector` trait, `FaceBox` and `BoundingBox` types, greedy NMS,
and a pure-Rust Haar-cascade heuristic baseline suitable for CI without model
weights.

## Features

- `onnx` (disabled by default): enables bridging calls to the face-onnx-worker
  sidecar for RetinaFace/MTCNN-quality detections.

## References

- Viola, P. & Jones, M. (2001). Rapid Object Detection using a Boosted Cascade
  of Simple Features. CVPR 2001.
- Hosang, J. et al. (2017). Learning Non-Maximum Suppression. CVPR 2017.
