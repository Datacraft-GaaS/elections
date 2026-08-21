//! Bulk export worker (CSV output).

use crate::error::BulkOpsError;
use csv::Writer;

/// Export a list of subject records as CSV bytes.
pub fn export_to_csv(records: &[Vec<String>], headers: &[&str]) -> Result<Vec<u8>, BulkOpsError> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(headers)?;
    for row in records {
        wtr.write_record(row)?;
    }
    let data = wtr.into_inner().map_err(|e| {
        BulkOpsError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    Ok(data)
}
