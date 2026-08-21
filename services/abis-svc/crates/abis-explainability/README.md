# eel-abis-explainability

Explainability primitives for ABIS (Automated Biometric Identification Systems): minutiae overlay SVGs, gradient saliency maps, SHAP/LIME facades, attention maps, and natural-language reasoning text.

## Minutiae Overlay SVG

`generate_svg` produces a side-by-side SVG with the probe fingerprint on the left panel and the gallery fingerprint on the right. Unmatched probe minutiae appear as red circles; unmatched gallery minutiae appear as blue circles. Matched minutiae appear green, connected by lines whose opacity reflects the pair confidence score. Each minutia also shows an angle indicator as a short line segment from its center.

## Vanilla Grad Saliency

`compute_saliency` divides an image into a grid of regions and assigns each a gradient magnitude equal to the absolute elementwise difference between two embedding vectors, indexed cyclically. This is a Vanilla Grad approximation: in production, wire automatic differentiation from the face embedding model to get true gradient norms per spatial region.

## SHAP Facade

`LocalShapExplainer` implements `ShapExplainer` and returns a per-feature contribution vector: the difference from baseline, rescaled so the absolute values sum to 1.0. This is a local approximation. Production plugs into a Python worker via gRPC for exact SHAP values using the SHAP library.

## LIME Facade

`LocalLimeExplainer` implements `LimeExplainer` and returns uniform feature importances as a stub. Production trains a local linear model on perturbed samples weighted by a kernel of width `kernel_width`, fitting coefficients as importances.

## Attention Maps

`UniformAttentionEmitter` implements `AttentionEmitter` and returns a flat uniform attention map over all pixels. Production emits per-pixel attention weights extracted from the face model's self-attention heads.

## Natural Language Reasoning

`generate_reasoning_text` converts a slice of `ModalityResult` records into human-readable lines: one line per modality reporting matched minutiae, probe and gallery totals, fusion score, threshold, and local decision. A final line states the overall decision. This text is suitable for audit trails and operator review.

## Error Handling

All fallible functions return `Result<_, ExplainError>`. No `unwrap()` calls in library code. Errors:

- `EmptyTemplate`: empty input list
- `DimensionMismatch`: mismatched vector lengths
- `NoMatchData`: no match data available
- `InvalidInput(String)`: invalid parameter with description
