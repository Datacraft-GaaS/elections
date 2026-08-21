# eel-abis-iris-dl

Deep iris embedding for the EEL ABIS system, inspired by DeepIrisNet (Gangwar and Joshi, 2016).

Default build uses a hash-based deterministic embedder (no ONNX runtime required). Enable the `onnx` feature for model-backed inference.

## Modules

- `embedder`: `Embedding`, cosine similarity, `IrisEmbedder` interface.
- `hash_embedder`: SHA-256 salt-expanded deterministic embedder producing 256-d unit vectors.
