use thiserror::Error;

use crate::domain::LotData;

#[derive(Debug, Error)]
/// PrintingError.
pub enum PrintingError {
	#[error("MES submission failed: {0}")]
	/// variant.
	MesSubmission(String),
}

#[derive(Debug, Clone, Default)]
/// PrinterMesDouble.
pub struct PrinterMesDouble;

impl PrinterMesDouble {
	#[allow(unused)]
	/// method.
	pub async fn submit_lot_data(&self, _lot: &LotData) -> Result<(), PrintingError> {
		Ok(())
	}
}
