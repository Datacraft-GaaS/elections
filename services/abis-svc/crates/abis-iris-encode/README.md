# eel-abis-iris-encode

Iris template encoding: Daugman rubber-sheet normalisation + 2D Gabor IrisCode + PIE variant.

## Algorithms

- **Rubber-sheet** (`normalise.rs`): polar unwrap to 512x64 normalised image
- **Gabor bank** (`gabor.rs`): 32 filters (8 frequencies x 4 orientations)
- **Daugman code** (`daugman_code.rs`): sign(Gabor) -> 2048-bit IrisCode + mask
- **PIE** (`pie.rs`): majority-vote across 7 angular offsets for rotation robustness
- **Encoder** (`encode.rs`): `IrisEncoder::encode(seg, image)` -> `IrisTemplate`
