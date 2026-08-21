use super::DiasporaLogisticsError;

#[derive(Debug, Clone, Default)]
pub struct AirlineDouble;

impl AirlineDouble {
	#[allow(unused)]
	pub async fn get_shipment_status(&self, _awb: &str) -> Result<String, DiasporaLogisticsError> {
		Ok(String::new())
	}
}
