use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::EbtsError;

/// FBI EBTS 10.0 Type-of-Transaction codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeOfTransaction {
    /// Criminal 10-print Array (arrest submission).
    CAR,
    /// Search Result Entry.
    SRE,
    /// Non-Fingerprint Array Print.
    NFAP,
    /// Identity Request.
    IRQ,
    /// Unsolicited Latent Fingerprint.
    ULF,
    /// Unsolicited Latent with Minutiae.
    ULM,
    /// Latent Fingerprint Search.
    LFS,
    /// Latent Fingerprint with Feature Set.
    LFFS,
    /// Error Response Message.
    ERM,
    /// Error Response Update.
    ERU,
    /// Electronic Notifier Update.
    ENU,
    /// Electronic Notifier Entry.
    ENE,
    /// Identity Verification.
    IDV,
}

impl fmt::Display for TypeOfTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::CAR => "CAR",
            Self::SRE => "SRE",
            Self::NFAP => "NFAP",
            Self::IRQ => "IRQ",
            Self::ULF => "ULF",
            Self::ULM => "ULM",
            Self::LFS => "LFS",
            Self::LFFS => "LFFS",
            Self::ERM => "ERM",
            Self::ERU => "ERU",
            Self::ENU => "ENU",
            Self::ENE => "ENE",
            Self::IDV => "IDV",
        };
        f.write_str(s)
    }
}

impl FromStr for TypeOfTransaction {
    type Err = EbtsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CAR" => Ok(Self::CAR),
            "SRE" => Ok(Self::SRE),
            "NFAP" => Ok(Self::NFAP),
            "IRQ" => Ok(Self::IRQ),
            "ULF" => Ok(Self::ULF),
            "ULM" => Ok(Self::ULM),
            "LFS" => Ok(Self::LFS),
            "LFFS" => Ok(Self::LFFS),
            "ERM" => Ok(Self::ERM),
            "ERU" => Ok(Self::ERU),
            "ENU" => Ok(Self::ENU),
            "ENE" => Ok(Self::ENE),
            "IDV" => Ok(Self::IDV),
            _ => Err(EbtsError::UnknownTot(s.to_owned())),
        }
    }
}
