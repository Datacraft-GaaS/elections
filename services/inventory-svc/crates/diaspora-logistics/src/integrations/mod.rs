use thiserror::Error;

pub mod airline_double;
pub mod kra_double;
pub mod mfa_double;

#[derive(Debug, Error)]
pub enum DiasporaLogisticsError {
	#[error("MFA preclearance failed: {0}")]
	MfaPreclearance(String),
	#[error("KRA eclearance failed: {0}")]
	KraEclearance(String),
	#[error("Airline status unavailable: {0}")]
	AirlineStatus(String),
}
