use super::DiasporaLogisticsError;
use crate::domain::PreClearanceManifest;

#[derive(Debug, Clone, Default)]
pub struct KraDouble;

impl KraDouble {
	#[allow(unused)]
	pub async fn submit_eclearance(
		&self,
		_manifest: &PreClearanceManifest,
	) -> Result<(), DiasporaLogisticsError> {
		Ok(())
	}
}
