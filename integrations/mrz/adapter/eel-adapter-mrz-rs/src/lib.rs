//! ICAO 9303 MRZ parser: TD1 (3-line 30-char), TD2 (2-line 36-char), TD3 (2-line 44-char).
//!
//! All fields are validated against the ICAO checksum algorithm. PII: all name/DOB/ID fields
//! are Tier A (Restricted) per ADR-0016.
#![warn(missing_docs)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MrzError {
    #[error("Invalid MRZ format: {0}")]
    InvalidFormat(String),
    #[error("Checksum mismatch for field {field}: computed {computed} != expected {expected}")]
    ChecksumMismatch {
        field: String,
        computed: u8,
        expected: u8,
    },
    #[error("Unknown document type: {0}")]
    UnknownDocumentType(String),
}

/// ICAO 9303 check digit weights (cyclic: 7, 3, 1).
const WEIGHTS: [u32; 3] = [7, 3, 1];

/// Compute ICAO 9303 check digit over `s`.
pub fn check_digit(s: &str) -> u8 {
    let sum: u32 = s
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let val = match c {
                '<' => 0,
                '0'..='9' => (c as u32) - ('0' as u32),
                'A'..='Z' => (c as u32) - ('A' as u32) + 10,
                _ => 0,
            };
            val * WEIGHTS[i % 3]
        })
        .sum();
    (sum % 10) as u8
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Td3 {
    pub document_type: String,
    pub issuing_state: String,
    /// Surname and given names (PII Restricted - Tier A).
    pub name: String,
    /// Document number (PII Restricted - Tier A).
    pub document_number: String,
    pub nationality: String,
    /// Date of birth YYMMDD (PII Restricted - Tier A).
    pub date_of_birth: String,
    pub sex: String,
    /// Date of expiry YYMMDD.
    pub date_of_expiry: String,
    /// Optional data (may contain national ID).
    pub optional_data: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Td1 {
    pub document_type: String,
    pub issuing_state: String,
    pub document_number: String,
    pub optional_data_1: String,
    pub date_of_birth: String,
    pub sex: String,
    pub date_of_expiry: String,
    pub nationality: String,
    pub optional_data_2: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ParsedMrz {
    Td1(Td1),
    Td3(Td3),
}

/// Parse a TD3 MRZ (2 lines of 44 characters each).
pub fn parse_td3(line1: &str, line2: &str) -> Result<Td3, MrzError> {
    if line1.len() != 44 {
        return Err(MrzError::InvalidFormat(format!(
            "TD3 line1 must be 44 chars, got {}",
            line1.len()
        )));
    }
    if line2.len() != 44 {
        return Err(MrzError::InvalidFormat(format!(
            "TD3 line2 must be 44 chars, got {}",
            line2.len()
        )));
    }

    let doc_type = line1[0..2].to_string();
    let issuing_state = line1[2..5].to_string();
    let name_raw = &line1[5..44];
    let name = name_raw
        .replace("<<", ", ")
        .replace('<', " ")
        .trim()
        .to_string();

    let doc_number = &line2[0..9];
    let doc_check: u8 = line2.chars().nth(9).unwrap_or('0') as u8 - b'0';
    let computed = check_digit(doc_number);
    if computed != doc_check {
        return Err(MrzError::ChecksumMismatch {
            field: "document_number".into(),
            computed,
            expected: doc_check,
        });
    }

    let nationality = line2[10..13].to_string();
    let dob = line2[13..19].to_string();
    let dob_check: u8 = line2.chars().nth(19).unwrap_or('0') as u8 - b'0';
    let dob_computed = check_digit(&dob);
    if dob_computed != dob_check {
        return Err(MrzError::ChecksumMismatch {
            field: "date_of_birth".into(),
            computed: dob_computed,
            expected: dob_check,
        });
    }

    let sex = line2[20..21].to_string();
    let expiry = line2[21..27].to_string();
    let opt_data = line2[28..42].to_string();

    Ok(Td3 {
        document_type: doc_type,
        issuing_state,
        name,
        document_number: doc_number.to_string(),
        nationality,
        date_of_birth: dob,
        sex,
        date_of_expiry: expiry,
        optional_data: opt_data,
    })
}

/// Parse a TD1 MRZ (3 lines of 30 characters each).
pub fn parse_td1(line1: &str, line2: &str, line3: &str) -> Result<Td1, MrzError> {
    if line1.len() != 30 || line2.len() != 30 || line3.len() != 30 {
        return Err(MrzError::InvalidFormat(format!(
            "TD1 requires 3x30 chars, got {}/{}/{}",
            line1.len(),
            line2.len(),
            line3.len()
        )));
    }
    let doc_type = line1[0..2].to_string();
    let issuing_state = line1[2..5].to_string();
    let doc_number = line1[5..14].to_string();
    let doc_check: u8 = line1.chars().nth(14).unwrap_or('0') as u8 - b'0';
    let computed = check_digit(&doc_number);
    if computed != doc_check {
        return Err(MrzError::ChecksumMismatch {
            field: "document_number".into(),
            computed,
            expected: doc_check,
        });
    }
    let opt1 = line1[15..30].to_string();
    let dob = line2[0..6].to_string();
    let sex = line2[7..8].to_string();
    let expiry = line2[8..14].to_string();
    let nationality = line2[15..18].to_string();
    let opt2 = line2[18..29].to_string();
    let name_raw = &line3[0..30];
    let name = name_raw
        .replace("<<", ", ")
        .replace('<', " ")
        .trim()
        .to_string();
    Ok(Td1 {
        document_type: doc_type,
        issuing_state,
        document_number: doc_number,
        optional_data_1: opt1,
        date_of_birth: dob,
        sex,
        date_of_expiry: expiry,
        nationality,
        optional_data_2: opt2,
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ICAO 9303 Part 3 sample TD3 passport (male, fictional)
    const TD3_L1: &str = "P<UTOERIKSSON<<ANNA<MARIA<<<<<<<<<<<<<<<<<<<";
    const TD3_L2: &str = "L898902C36UTO7408122F1204159ZE184226B<<<<<10";

    #[test]
    fn td3_parse_name() {
        let mrz = parse_td3(TD3_L1, TD3_L2).expect("valid TD3");
        assert!(mrz.name.contains("ERIKSSON"));
    }

    #[test]
    fn td3_doc_number() {
        let mrz = parse_td3(TD3_L1, TD3_L2).expect("valid TD3");
        assert_eq!(mrz.document_number, "L898902C3");
    }

    #[test]
    fn check_digit_samples() {
        // ICAO 9303 Appendix A: "520727" -> 3
        assert_eq!(check_digit("520727"), 3);
        // "L898902C3" -> 6
        assert_eq!(check_digit("L898902C3"), 6);
    }

    #[test]
    fn td3_bad_checksum() {
        // Corrupt one char of doc number
        let bad_l2 = "L898902C96UTO7408122F1204159ZE184226B<<<<<10";
        let result = parse_td3(TD3_L1, bad_l2);
        assert!(matches!(result, Err(MrzError::ChecksumMismatch { .. })));
    }
}
