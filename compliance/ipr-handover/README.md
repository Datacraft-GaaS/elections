# IPR Handover Package

Intellectual property rights handover documentation for eel (IEBC Integrated Election Management System) under Government Common Conditions (GCC) 15.4.

## Files

| File | Purpose |
|---|---|
| `gcc-15-4-handover-template.md` | GCC 15.4 handover package template covering source inventory, licences, SBOMs, build instructions, escrow |
| `software-classification.yaml` | Custom vs Standard Software distinction for all eel components |
| `copyright-assignment.md` | Copyright and moral rights assignment template for custom software artefacts |
| `source-inventory.yaml` | Source code inventory of all custom eel components |

## Usage

1. Before contract signature: review `software-classification.yaml` to confirm which components are Custom (IP transferred to IEBC) and which are Standard (FOSS, obligations in `license-obligations`).
2. At each release: update `source-inventory.yaml` with the git hash and SBOM path for the released version.
3. At handover: complete `gcc-15-4-handover-template.md` with actual values and submit to IEBC.
4. For escrow: prepare the escrow manifest section and deposit materials with the agreed escrow agent.

## GCC 15.4 summary

GCC 15.4 vests ownership of Custom Software in the procuring entity (IEBC) on delivery. The supplier retains no rights to use Custom Software for other clients. Standard Software is licensed under the applicable open-source or proprietary licence terms and is not transferred to IEBC.
