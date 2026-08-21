//! Unified attestation verifier accepting TDX, SEV-SNP, or software attestation.

use crate::attestation::AttestationProfile;
use crate::error::AttestError;
use crate::nonce::NonceStore;
use crate::policy::{AttestationPolicy, AttestationVerdict, ReportPayload};
use crate::sev::{parse_and_verify_sev_snp, DevVcekStore, TrustedVcekStore};
use crate::tdx::{parse_and_verify_tdx, DevPckStore, TrustedPckStore};

/// Unified attestation verifier.
pub struct AttestationVerifier {
    pub policy: AttestationPolicy,
    pub nonce_store: NonceStore,
    pck_store: Box<dyn TrustedPckStore>,
    vcek_store: Box<dyn TrustedVcekStore>,
}

impl AttestationVerifier {
    /// Create a verifier with dev (trust-all) cert stores.
    pub fn new_dev(policy: AttestationPolicy) -> Self {
        Self {
            policy,
            nonce_store: NonceStore::new(),
            pck_store: Box::new(DevPckStore),
            vcek_store: Box::new(DevVcekStore),
        }
    }

    /// Create a verifier with custom cert stores.
    pub fn new(
        policy: AttestationPolicy,
        pck_store: Box<dyn TrustedPckStore>,
        vcek_store: Box<dyn TrustedVcekStore>,
    ) -> Self {
        Self {
            policy,
            nonce_store: NonceStore::new(),
            pck_store,
            vcek_store,
        }
    }

    /// Issue a fresh nonce for the client to include in its attestation report.
    pub fn issue_nonce(&self) -> String {
        self.nonce_store.issue().hex()
    }

    /// Verify a raw attestation quote, consuming the nonce.
    ///
    /// The profile is auto-detected from the first byte:
    /// - 0x04 -> TDX
    /// - 0x01 -> SEV-SNP
    /// - anything else with `allow_software_attestation` -> Software
    pub fn verify(
        &self,
        enclave_id: &str,
        quote_bytes: &[u8],
        nonce_hex: &str,
    ) -> Result<AttestationVerdict, AttestError> {
        // Consume nonce first to prevent replay even on profile mismatch.
        self.nonce_store.consume(nonce_hex)?;

        let profile = detect_profile_from_bytes(quote_bytes);
        let payload = match profile {
            AttestationProfile::Tdx => {
                let report = parse_and_verify_tdx(quote_bytes, self.pck_store.as_ref())?;
                ReportPayload::Tdx(report)
            }
            AttestationProfile::SevSnp => {
                let report = parse_and_verify_sev_snp(quote_bytes, self.vcek_store.as_ref())?;
                ReportPayload::SevSnp(report)
            }
            AttestationProfile::Software => {
                tracing::warn!(
                    enclave_id,
                    "software attestation used; not accepted in production"
                );
                ReportPayload::Software
            }
        };

        let verdict = self.policy.enforce(&payload)?;

        if verdict.approved {
            tracing::info!(
                enclave_id,
                profile = ?verdict.profile,
                "enclave_attested"
            );
        }

        Ok(verdict)
    }
}

fn detect_profile_from_bytes(bytes: &[u8]) -> AttestationProfile {
    match bytes.first() {
        Some(0x04) => AttestationProfile::Tdx,
        Some(0x01) => AttestationProfile::SevSnp,
        _ => AttestationProfile::Software,
    }
}
