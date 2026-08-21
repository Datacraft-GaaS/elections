# eel-abis-iris-fusion

Bilateral (left+right iris) and biometric (iris+periocular) score fusion for the EEL ABIS system.

## Modules

- `bilateral`: Fuses left and right iris match scores using Average or LogicalAnd strategy, with optional quality weighting.
- `biometric`: Fuses iris and periocular similarity scores using configurable per-modality weights.
