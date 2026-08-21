//! Attestation policy enforcement.

use serde::{Deserialize, Serialize};

use crate::attestation::AttestationProfile;
use crate::error::AttestError;
use crate::sev::SevSnpReport;
use crate::tdx::TdReport;

/// Result of enforcing a policy against an attestation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationVerdict {
    pub approved: bool,
    pub profile: AttestationProfile,
    pub reason: Option<String>,
}

/// An attestation payload accepted by policy enforcement.
#[derive(Debug, Clone)]
pub enum ReportPayload {
    Tdx(TdReport),
    SevSnp(SevSnpReport),
    /// Software attestation for development environments.
    Software,
}

impl ReportPayload {
    pub fn profile(&self) -> AttestationProfile {
        match self {
            ReportPayload::Tdx(_) => AttestationProfile::Tdx,
            ReportPayload::SevSnp(_) => AttestationProfile::SevSnp,
            ReportPayload::Software => AttestationProfile::Software,
        }
    }
}

/// Policy that controls which attestation profiles and measurements are accepted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationPolicy {
    /// Require Intel TDX attestation.
    pub require_tdx: bool,
    /// Require AMD SEV-SNP attestation.
    pub require_sev: bool,
    /// Allowed MRTD hashes (empty = any). Each element is 48 bytes.
    pub allowed_mrtd_hashes: Vec<Vec<u8>>,
    /// Allowed SEV-SNP measurement hashes (empty = any). Each element is 48 bytes.
    pub allowed_sev_measurements: Vec<Vec<u8>>,
    /// Minimum CPU SVN required for TDX.
    pub min_cpu_svn: u16,
    /// Whether to accept software attestation (dev only; always false in prod).
    pub allow_software_attestation: bool,
}

impl AttestationPolicy {
    /// Production policy: require TDX, no software.
    pub fn production() -> Self {
        Self {
            require_tdx: true,
            require_sev: false,
            allowed_mrtd_hashes: vec![],
            allowed_sev_measurements: vec![],
            min_cpu_svn: 0,
            allow_software_attestation: false,
        }
    }

    /// Development policy: accept software attestation.
    pub fn development() -> Self {
        Self {
            require_tdx: false,
            require_sev: false,
            allowed_mrtd_hashes: vec![],
            allowed_sev_measurements: vec![],
            min_cpu_svn: 0,
            allow_software_attestation: true,
        }
    }

    /// Enforce the policy against a report payload.
    pub fn enforce(&self, report: &ReportPayload) -> Result<AttestationVerdict, AttestError> {
        let profile = report.profile();

        // Software attestation: reject unless explicitly allowed.
        if matches!(report, ReportPayload::Software) {
            if !self.allow_software_attestation {
                return Err(AttestError::SoftwareAttestationRejected);
            }
            return Ok(AttestationVerdict {
                approved: true,
                profile,
                reason: Some("software attestation (dev mode)".into()),
            });
        }

        // Profile requirements.
        if self.require_tdx && !matches!(report, ReportPayload::Tdx(_)) {
            return Err(AttestError::PolicyViolation(format!(
                "policy requires TDX but got {:?}",
                profile
            )));
        }
        if self.require_sev && !matches!(report, ReportPayload::SevSnp(_)) {
            return Err(AttestError::PolicyViolation(format!(
                "policy requires SEV-SNP but got {:?}",
                profile
            )));
        }

        match report {
            ReportPayload::Tdx(r) => self.enforce_tdx(r)?,
            ReportPayload::SevSnp(r) => self.enforce_sev_snp(r)?,
            ReportPayload::Software => unreachable!(),
        }

        Ok(AttestationVerdict {
            approved: true,
            profile,
            reason: None,
        })
    }

    fn enforce_tdx(&self, report: &TdReport) -> Result<(), AttestError> {
        if report.cpu_svn < self.min_cpu_svn {
            return Err(AttestError::PolicyViolation(format!(
                "cpu_svn {} below minimum {}",
                report.cpu_svn, self.min_cpu_svn
            )));
        }
        if !self.allowed_mrtd_hashes.is_empty()
            && !self.allowed_mrtd_hashes.iter().any(|h| h == &report.mrtd)
        {
            return Err(AttestError::PolicyViolation(
                "MRTD hash not in allowed list".into(),
            ));
        }
        Ok(())
    }

    fn enforce_sev_snp(&self, report: &SevSnpReport) -> Result<(), AttestError> {
        if !self.allowed_sev_measurements.is_empty()
            && !self
                .allowed_sev_measurements
                .iter()
                .any(|h| h == &report.measurement)
        {
            return Err(AttestError::PolicyViolation(
                "SEV-SNP measurement not in allowed list".into(),
            ));
        }
        Ok(())
    }
}
