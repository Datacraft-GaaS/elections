use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{error::EbtsError, tot::TypeOfTransaction, transaction::EbtsRecord};

/// Outcome of an EBTS transaction submission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    Accept,
    Reject(String),
    Error(String),
}

/// A response message returned for an EBTS transaction.
///
/// Binary layout matches `EbtsTransaction` for the header portion, then
/// appends a status field and optional additional records.
///
/// ```text
/// [1]  record_type = 1
/// [1]  idc = 0
/// [2]  header_len: u16 (be)
/// [N]  "TOT:<tot>\x1fTCN:<tcn>\x1fSTATUS:<status_tag>[:<payload>]"
/// additional records as in EbtsTransaction
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EbtsResponse {
    pub tot: TypeOfTransaction,
    pub tcn: String,
    pub status: ResponseStatus,
    pub records: Vec<EbtsRecord>,
}

const US: u8 = 0x1F;

impl EbtsResponse {
    pub fn parse(bytes: &[u8]) -> Result<Self, EbtsError> {
        let mut pos = 0;

        if pos + 4 > bytes.len() {
            return Err(EbtsError::InvalidFormat(
                "truncated response header".to_string(),
            ));
        }
        let rt = bytes[pos];
        if rt != 1 {
            return Err(EbtsError::InvalidFormat(format!(
                "expected record_type=1, got {rt}"
            )));
        }
        let header_len = u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;
        pos += 4;

        if pos + header_len > bytes.len() {
            return Err(EbtsError::InvalidFormat(
                "response header content truncated".to_string(),
            ));
        }
        let content = std::str::from_utf8(&bytes[pos..pos + header_len])
            .map_err(|e| EbtsError::InvalidFormat(format!("non-UTF-8 response header: {e}")))?;
        pos += header_len;

        let (tot, tcn, status) = parse_response_content(content)?;

        // Additional records (same wire format as EbtsTransaction)
        let mut records = Vec::new();
        while pos < bytes.len() {
            if pos + 6 > bytes.len() {
                return Err(EbtsError::InvalidFormat(
                    "truncated response record".to_string(),
                ));
            }
            let record_type = bytes[pos];
            let idc = bytes[pos + 1];
            let data_len = u32::from_be_bytes(bytes[pos + 2..pos + 6].try_into().map_err(|_| {
                EbtsError::InvalidFormat("bad response record data_len".to_string())
            })?) as usize;
            pos += 6;

            if pos + data_len > bytes.len() {
                return Err(EbtsError::InvalidFormat(
                    "response record data truncated".to_string(),
                ));
            }
            let hex_str = std::str::from_utf8(&bytes[pos..pos + data_len])
                .map_err(|e| EbtsError::InvalidFormat(format!("non-UTF-8 response data: {e}")))?;
            let data = hex_decode(hex_str)?;
            pos += data_len;

            records.push(EbtsRecord {
                idc,
                record_type,
                data,
            });
        }

        Ok(Self {
            tot,
            tcn,
            status,
            records,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();

        let status_str = match &self.status {
            ResponseStatus::Accept => "ACCEPT".to_string(),
            ResponseStatus::Reject(msg) => format!("REJECT:{msg}"),
            ResponseStatus::Error(msg) => format!("ERROR:{msg}"),
        };

        let content = format!(
            "TOT:{}{US}TCN:{}{US}STATUS:{}",
            self.tot,
            self.tcn,
            status_str,
            US = US as char,
        );
        let content_bytes = content.as_bytes();
        let header_len = content_bytes.len() as u16;

        out.push(1_u8);
        out.push(0_u8);
        out.extend_from_slice(&header_len.to_be_bytes());
        out.extend_from_slice(content_bytes);

        for rec in &self.records {
            out.push(rec.record_type);
            out.push(rec.idc);
            let hex = hex_encode(&rec.data);
            let data_len = hex.len() as u32;
            out.extend_from_slice(&data_len.to_be_bytes());
            out.extend_from_slice(hex.as_bytes());
        }

        out
    }
}

fn parse_response_content(
    content: &str,
) -> Result<(TypeOfTransaction, String, ResponseStatus), EbtsError> {
    let mut tot = None;
    let mut tcn = None;
    let mut status = None;

    for part in content.split('\x1f') {
        if let Some(v) = part.strip_prefix("TOT:") {
            tot = Some(TypeOfTransaction::from_str(v)?);
        } else if let Some(v) = part.strip_prefix("TCN:") {
            tcn = Some(v.to_owned());
        } else if let Some(v) = part.strip_prefix("STATUS:") {
            status = Some(parse_status(v)?);
        }
    }

    Ok((
        tot.ok_or_else(|| EbtsError::MissingField("TOT".to_string()))?,
        tcn.ok_or_else(|| EbtsError::MissingField("TCN".to_string()))?,
        status.ok_or_else(|| EbtsError::MissingField("STATUS".to_string()))?,
    ))
}

fn parse_status(s: &str) -> Result<ResponseStatus, EbtsError> {
    if s == "ACCEPT" {
        return Ok(ResponseStatus::Accept);
    }
    if let Some(msg) = s.strip_prefix("REJECT:") {
        return Ok(ResponseStatus::Reject(msg.to_owned()));
    }
    if let Some(msg) = s.strip_prefix("ERROR:") {
        return Ok(ResponseStatus::Error(msg.to_owned()));
    }
    Err(EbtsError::InvalidFormat(format!(
        "unknown status tag: '{s}'"
    )))
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, EbtsError> {
    if s.len() % 2 != 0 {
        return Err(EbtsError::InvalidFormat(
            "odd-length hex string".to_string(),
        ));
    }
    (0..s.len() / 2)
        .map(|i| {
            u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
                .map_err(|e| EbtsError::InvalidFormat(format!("bad hex byte at {i}: {e}")))
        })
        .collect()
}
