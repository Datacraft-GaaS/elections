use crate::FusionError;

pub fn majority_vote(decisions: &[bool]) -> Result<bool, FusionError> {
    if decisions.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    let trues = decisions.iter().filter(|&&d| d).count();
    Ok(trues * 2 > decisions.len())
}

pub fn weighted_vote(decisions: &[bool], weights: &[f64]) -> Result<bool, FusionError> {
    if decisions.len() != weights.len() {
        return Err(FusionError::DimensionMismatch {
            expected: decisions.len(),
            found: weights.len(),
        });
    }
    let total_weight: f64 = weights.iter().sum();
    let true_weight: f64 = decisions
        .iter()
        .zip(weights.iter())
        .filter(|(&d, _)| d)
        .map(|(_, &w)| w)
        .sum();
    Ok(true_weight > 0.5 * total_weight)
}

pub fn borda_count(candidates: &[Vec<usize>]) -> Result<Vec<usize>, FusionError> {
    if candidates.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    let n_candidates = candidates.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut scores = vec![0usize; n_candidates];
    for ranking in candidates {
        let n = ranking.len();
        for (rank, &candidate) in ranking.iter().enumerate() {
            if candidate < n_candidates {
                scores[candidate] += n - 1 - rank;
            }
        }
    }
    let mut indexed: Vec<(usize, usize)> = scores.into_iter().enumerate().collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(indexed.into_iter().map(|(i, _)| i).collect())
}

pub fn bayesian_belief(likelihoods: &[f64], prior: f64) -> Result<f64, FusionError> {
    if likelihoods.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    let mut posterior = prior;
    for &l in likelihoods {
        let numerator = l * posterior;
        let denominator = numerator + (1.0 - l) * (1.0 - posterior);
        posterior = if denominator.abs() < f64::EPSILON {
            posterior
        } else {
            numerator / denominator
        };
    }
    Ok(posterior)
}
