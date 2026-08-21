# eel-abis-iris-irex

Rust implementation of the NIST IREX (Iris Exchange) evaluation framework.

Covers IREX I through IREX X: corpus management, 1:1 verification, 1:N identification,
ROC curve generation, and per-group fairness analysis.

Reference: <https://www.nist.gov/programs-projects/iris-exchange-irex-overview>

## Modules

- `corpus` - Load and validate IREX test corpora; generate genuine and impostor pairs.
- `verify` - 1:1 verification with configurable threshold; returns score and decision.
- `identify` - 1:N identification returning rank-ordered candidates.
- `report` - Report card with ROC curve, rank-N rates, and demographic fairness slices.
