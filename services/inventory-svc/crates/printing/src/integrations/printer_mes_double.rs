use thiserror::Error;

use crate::domain::LotData;

#[derive(Debug, Error)]
pub enum PrintingError {
	#[error("MES submission failed: {0}")]
	MesSubmission(String),
}

#[derive(Debug, Clone, Default)]
pub struct PrinterMesDouble;

impl PrinterMesDouble {
	#[allow(unused)]
	pub async fn submit_lot_data(&self, _lot: &LotData) -> Result<(), PrintingError> {
		Ok(())
	}
}
