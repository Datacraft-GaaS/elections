# ABIS Performance Card

> Model/performance transparency card for the ABIS module (M03), in the spirit
> of open-iris's `IRIS_PERFORMANCE_CARD.md` / `SEMSEG_MODEL_CARD.md`. The
> authoritative accuracy numbers come from the external NIST evaluation
> harnesses; this card states *what* is measured and *how*, never invented
> figures.

## 1. What this is

M03 `abis` is the biometric matching & de-duplication engine: enrolment,
1:N search, and adjudication across 11 modalities (fingerprint, face, iris,
palm, latent, voice, vein, signature, periocular, ear, gait). Match volume is
~60k rps at peak (plus audit shadow).

## 2. Evaluation harnesses (source of truth)

| Modality | Harness | Output |
|---|---|---|
| Fingerprint | NIST FpVTE | FNMR @ FMR 0.01/0.001, template interoperability |
| Face | NIST FRVT 1:1 / 1:N | FNMR/FNIR @ FMR, demographic differentials |
| Iris | NIST IREX | FNMR @ FMR, image-quality sensitivity |

Scores are populated from each harness run and recorded in the report
catalogue (25 standard reports + ad-hoc); they are **not** self-reported.

## 3. Algorithms (as wired)

- Fingerprint: MCC, CDEFFS, L3, SourceAFIS, DeepPrint.
- Face: ArcFace, 3DMM, age & cross-domain normalization.
- Iris: Daugman iris code, PIE, DeepIrisNet.
- Presentation-attack detection: PAD Level 2.
- Privacy: cancelable templates, FHE + SMPC matching paths.

## 4. Data classification & PII

Templates are data-class **A** (never logged, never cached); match events are
class B; quality/report aggregates are class C/D. See `data_dictionary.yaml`.

## 5. Fairness & limitations

- Face matching is monitored for demographic differentials per FRVT
  reporting; any cohort above the fairness threshold raises a `eel_fairness_*`
  evaluation record (V024).
- Latent/vein/gait modalities are lower-assurance; they feed adjudication
  queues, never automated single-factor decisions.

## 6. Review cadence

This card is updated on every harness re-run and at each NIST evaluation
cycle; drift alerts (`drift_alerts`, V006) flag model regression between runs.
