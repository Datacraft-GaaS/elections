//! Standardised adjudication reason-code catalogue.
//! A1-A8 = rejection reasons; V1-V3 = acceptance reasons.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Adjudication decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdjudicationDecision {
    Reject,
    Accept,
}

/// Reason-code catalogue entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonCode {
    pub code: &'static str,
    pub decision: AdjudicationDecision,
    pub description_en: &'static str,
    pub description_sw: &'static str,
}

/// Full catalogue: A1-A8 (rejection) and V1-V3 (acceptance).
pub static CATALOGUE: &[ReasonCode] = &[
    ReasonCode {
        code: "A1",
        decision: AdjudicationDecision::Reject,
        description_en: "Ballot not official: missing security features",
        description_sw: "Karatasi si rasmi: vipengele vya usalama havipo",
    },
    ReasonCode {
        code: "A2",
        decision: AdjudicationDecision::Reject,
        description_en: "Voter intention not clear: mark ambiguous",
        description_sw: "Nia ya mpiga kura haiko wazi: alama si dhahiri",
    },
    ReasonCode {
        code: "A3",
        decision: AdjudicationDecision::Reject,
        description_en: "Mark made for more candidates than allowed",
        description_sw: "Alama zimewekwa kwa wagombea zaidi ya idadi inayoruhusiwa",
    },
    ReasonCode {
        code: "A4",
        decision: AdjudicationDecision::Reject,
        description_en: "Ballot has identifying marks or writing",
        description_sw: "Karatasi ina alama au maandishi ya utambuzi",
    },
    ReasonCode {
        code: "A5",
        decision: AdjudicationDecision::Reject,
        description_en: "Ballot torn or damaged beyond readable state",
        description_sw: "Karatasi imechanika au kuharibiwa kupita kiasi",
    },
    ReasonCode {
        code: "A6",
        decision: AdjudicationDecision::Reject,
        description_en: "No mark made for any candidate",
        description_sw: "Hakuna alama iliyowekwa kwa mgombea yeyote",
    },
    ReasonCode {
        code: "A7",
        decision: AdjudicationDecision::Reject,
        description_en: "Mark is outside all candidate boxes",
        description_sw: "Alama ipo nje ya masanduku yote ya wagombea",
    },
    ReasonCode {
        code: "A8",
        decision: AdjudicationDecision::Reject,
        description_en: "Counterfeit ballot detected by security feature scan",
        description_sw: "Karatasi bandia imegunduliwa na uchunguzi wa vipengele vya usalama",
    },
    ReasonCode {
        code: "V1",
        decision: AdjudicationDecision::Accept,
        description_en: "Clear mark in candidate box; intention unambiguous",
        description_sw: "Alama wazi ndani ya sanduku la mgombea; nia iko dhahiri",
    },
    ReasonCode {
        code: "V2",
        decision: AdjudicationDecision::Accept,
        description_en: "Mark slightly outside box but voter intention clear by proximity",
        description_sw: "Alama kidogo nje ya sanduku lakini nia ya mpiga kura iko wazi",
    },
    ReasonCode {
        code: "V3",
        decision: AdjudicationDecision::Accept,
        description_en: "Assisted voter: mark position consistent with stated intent and witness record",
        description_sw: "Mpiga kura aliyesaidiwa: nafasi ya alama inaendana na nia iliyosemwa na rekodi ya shahidi",
    },
];

#[derive(Debug, Error)]
pub enum CatalogueError {
    #[error("unknown reason code '{0}'; must be A1-A8 or V1-V3")]
    UnknownCode(String),
}

/// Look up a reason code from the catalogue.
pub fn lookup(code: &str) -> Result<&'static ReasonCode, CatalogueError> {
    CATALOGUE
        .iter()
        .find(|r| r.code == code)
        .ok_or_else(|| CatalogueError::UnknownCode(code.to_string()))
}

/// Validate that a code matches the expected decision direction.
pub fn validate_decision(code: &str, claimed: &AdjudicationDecision) -> Result<(), CatalogueError> {
    let entry = lookup(code)?;
    if &entry.decision != claimed {
        return Err(CatalogueError::UnknownCode(format!(
            "code {code} has decision {:?}, not {claimed:?}",
            entry.decision
        )));
    }
    Ok(())
}
