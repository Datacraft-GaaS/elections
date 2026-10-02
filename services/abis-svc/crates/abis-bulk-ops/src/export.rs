// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
