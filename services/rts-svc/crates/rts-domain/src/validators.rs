use thiserror::Error;
use crate::aggregates::form34a::Form34A;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("EC-DAT-10: {0}")]
    EcDat10(String),
    #[error("EC-DAT-11: {0}")]
    EcDat11(String),
}

/// EC-DAT-10: Arithmetic consistency check.
pub fn validate_ec_dat_10(form: &Form34A) -> Result<(), ValidationError> {
    if form.registered_voters == 0 {
        return Err(ValidationError::EcDat10(
            "registered_voters must be greater than zero".to_string(),
        ));
    }
    let computed_cast = form.valid_votes + form.rejected_votes + form.spoilt_ballots;
    if computed_cast != form.votes_cast {
        return Err(ValidationError::EcDat10(format!(
            "votes_cast ({}) must equal valid_votes ({}) + rejected_votes ({}) + spoilt_ballots ({}), got sum {}",
            form.votes_cast, form.valid_votes, form.rejected_votes, form.spoilt_ballots, computed_cast
        )));
    }
    Ok(())
}

/// EC-DAT-11: Spoilt and rejected votes sanity check.
pub fn validate_ec_dat_11(form: &Form34A) -> Result<(), ValidationError> {
    if form.rejected_votes + form.valid_votes > form.registered_voters {
        return Err(ValidationError::EcDat11(format!(
            "rejected_votes ({}) + valid_votes ({}) = {} exceeds registered_voters ({}): impossible result",
            form.rejected_votes,
            form.valid_votes,
            form.rejected_votes + form.valid_votes,
            form.registered_voters
        )));
    }
    if form.spoilt_ballots > form.valid_votes {
        tracing::warn!(
            reason = "spoilt_exceeds_valid",
            station_code = %form.station_code,
            spoilt = form.spoilt_ballots,
            valid = form.valid_votes,
            "EC-DAT-11: spoilt ballots exceed valid votes (allowed with annotation)"
        );
    }
    Ok(())
}
