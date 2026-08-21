# eel-abis-fusion-advanced

Advanced multi-modal score, feature, and decision fusion for ABIS (Automated Biometric Identification Systems).

## Score-Level Fusion

Score-level fusion combines raw similarity scores from multiple biometric matchers before a binary accept/reject decision.

### Normalization

Raw scores from different matchers operate on different ranges and distributions. Four normalization methods are provided:

- **Min-max**: Linearly rescales to [0, 1] using the range of the score set. Returns 0.5 for uniform inputs.
- **Z-score**: Centers and scales by standard deviation. Returns 0.0 for zero-variance inputs.
- **Tanh**: A soft sigmoid using tanh(0.01*(x - mu)/sigma) shifted to [0, 1]. Robust to outliers.
- **Double sigmoid**: Two-piece sigmoid with inflection at the mean. Compresses extreme values while keeping rank order.

### Fusion Rules

- **T-norm** (product): `a * b`. Conservative AND-like combination.
- **S-norm** (probabilistic sum): `a + b - a*b`. Liberal OR-like combination.
- **Weighted sum**: Dot product of scores and weights, divided by weight sum. The standard workhorse.
- **Product fusion**: Naive product of all scores. Suitable when scores are independent likelihoods.
- **Likelihood ratio fusion**: Converts scores to log-odds using per-score priors, sums them, and converts back. Principled Bayesian combination.

## Feature-Level Fusion

Feature-level fusion concatenates or projects feature vectors before matching.

- **concatenate**: Simple vector concatenation of two feature sets.
- **pca_reduce**: Stub that returns the first n components. Wire a real PCA (ndarray-linalg) in production.
- **KernelPca trait**: Interface for kernel PCA; implement `compute_kernel_matrix` and `project`. Production integrates a backend.

## Decision-Level Fusion

Decision-level fusion combines binary match/no-match outputs.

- **majority_vote**: Accepts if more than 50% of matchers accept.
- **weighted_vote**: Weighted majority using per-matcher confidence weights.
- **borda_count**: Rank aggregation using Borda points (n-1 for rank 0, etc.). Returns candidates sorted by descending total score.
- **bayesian_belief**: Chains Bayes updates starting from a prior, accumulating evidence from each likelihood.

## Cohort Normalization

`CohortNormalizer` holds per-demographic Z-score parameters (mean, std, count). Calling `normalize(cohort_id, score)` returns `(score - mean) / std` for the registered cohort. Returns `InsufficientCohortData` for unknown cohort IDs.

## Spatiotemporal Contextual Filtering

`SpatiotemporalFilter` prunes candidates that are geographically or temporally implausible for a given probe event. Haversine distance enforces a `max_distance_km` constraint; a timestamp window enforces `max_time_diff_seconds`. Candidates outside either bound are excluded before score-level fusion.

## Error Handling

All fallible functions return `Result<_, FusionError>`. No `unwrap()` calls in library code. Errors:

- `EmptyScores`: empty input slice
- `DimensionMismatch { expected, found }`: mismatched lengths
- `WeightSumZero`: weight vector sums to zero
- `InsufficientCohortData(String)`: cohort not registered
