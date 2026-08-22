# eel-governance-parallel-manual-count

Parallel manual count at T+3 (three days after election day) covering 1% of polling stations. Feeds into the Risk-Limiting Audit (RLA) pipeline (HARD-20).

## Purpose

Independent human recount of a statistically selected sample of polling stations. Conducted in parallel with the platform tally to detect systematic errors or manipulation. The sample size of 462 stations out of 46,229 total (1%) provides sufficient statistical power to detect divergences larger than the margin that would affect the outcome.

## Sampling Methodology

Stations are selected via Neyman (1934) stratified allocation. The population is divided into 9 strata corresponding to Kenya's administrative regions plus a diaspora/special category. Within each stratum, a proportional share of the 462-station sample is allocated.

Under equal within-stratum variance assumption, Neyman allocation reduces to proportional allocation:

    n_h = n * (N_h / N)

where N = 46,229 total stations, n = 462 target sample, N_h = station count in stratum h, n_h = allocated sample for stratum h.

Stratum totals:

| Stratum       | Stations | Allocation |
|---------------|----------|------------|
| Nairobi       | 2,645    | ~26        |
| Central       | 3,201    | ~32        |
| Coast         | 2,868    | ~29        |
| Eastern       | 5,891    | ~59        |
| NorthEastern  | 1,234    | ~12        |
| Nyanza        | 4,502    | ~45        |
| RiftValley    | 10,388   | ~104       |
| Western       | 3,109    | ~31        |
| Diaspora      | 12,391   | ~124       |
| **Total**     | **46,229** | **462**  |

Selection within each stratum uses a seeded linear congruential generator (LCG) for determinism and auditability. The seed is recorded in the SamplePlan and must be published before counting begins.

## Divergence Computation

For each sampled station, returning officers submit per-candidate manual counts. The platform compares these against its own tally to compute:

- Per-candidate delta: manual_count - platform_count
- Per-station max absolute divergence
- Report-level mean absolute divergence across all candidate-station pairs

Stations where any candidate's absolute delta exceeds zero are flagged.

## RLA Integration

The DivergenceReport is serialized and submitted to the RLA service (`eel-governance-rla`) as an RlaAuditInput. The RLA service runs SHANGRLA (Stark 2020) ballot-polling or ballot-comparison audits informed by this divergence signal to decide whether to escalate to a full hand recount.

## References

- Neyman, J. (1934). "On the Two Different Aspects of the Representative Method: The Method of Stratified Sampling and the Method of Purposive Selection." Journal of the Royal Statistical Society, 97(4), 558-625.
- Stark, P. B. (2020). "SHANGRLA: Sets of Half-Average Nulls Generate Risk-Limiting Audits." In International Joint Conference on Electronic Voting.
