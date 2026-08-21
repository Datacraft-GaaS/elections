# eel-abis-fp-dl-onnx

Deep-learning fingerprint matcher for the eel ABIS pipeline.

Implements a DeepPrint-style 384-d embedding with cosine-similarity matching
via ONNX Runtime, with a deterministic hash-based fallback when no model is
available.

## Reference

Engelsma, Cao, Jain, "Learning a Fixed-Length Fingerprint Representation,"
IEEE Trans. Pattern Anal. Mach. Intell. 43(6), 2021.

## Model acquisition

1. Obtain a DeepPrint ONNX model (input: `[1, 1, 224, 224]` float32, output:
   `[1, 384]` float32 L2-normalised embedding).
2. Set `EEL_FP_DL_ONNX_PATH=/path/to/model.onnx`.
3. Enable the `onnx` cargo feature: `--features onnx`.

## Naming convention

ONNX model files: `deepprint_v{VERSION}_{DATASET}.onnx`
(e.g. `deepprint_v1_nist_sd4.onnx`).

## Score fusion

Use `fusion::fuse` to combine the DL score with MCC and Bozorth3 scores.
Calibrate weights with `fusion::fit_weights` on a held-out corpus.
