use super::DiasporaLogisticsError;

#[derive(Debug, Clone, Default)]
pub struct MfaDouble;

impl MfaDouble {
	#[allow(unused)]
	pub async fn check_preclearance(&self, _manifest_id: &str) -> Result<(), DiasporaLogisticsError> {
		Ok(())
	}
}
