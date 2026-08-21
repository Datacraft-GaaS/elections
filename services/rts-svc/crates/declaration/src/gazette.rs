//! GazettePublisher trait and NoopPublisher stub (WP-3.3, M51).
//! Wired to the real Government Printer adapter in Wave 4.

use std::future::Future;
use std::pin::Pin;

/// Outcome of a gazette package submission.
#[derive(Debug, Clone)]
pub struct GazetteAck {
    pub gazette_ref:  String,
    pub submitted_at: String,
}

/// Error during gazette submission.
#[derive(Debug)]
pub enum GazetteError {
    Unavailable(String),
    Rejected { reason: String },
}

impl std::fmt::Display for GazetteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GazetteError::Unavailable(msg) => write!(f, "gazette unavailable: {msg}"),
            GazetteError::Rejected { reason } => write!(f, "gazette rejected: {reason}"),
        }
    }
}

/// Package pushed to the Government Printer on declaration.
#[derive(Debug, Clone)]
pub struct GazettePackage {
    pub declaration_id:  String,
    pub contest_id:      String,
    pub winner_id:       Option<String>,
    pub certificate_ref: Option<String>,
    /// MinIO reference to the signed certificate PDF.
    pub pdf_minio_ref:   Option<String>,
}

/// Trait for submitting gazette packages.  Implementations live in integrations/.
pub trait GazettePublisher: Send + Sync {
    fn submit<'a>(
        &'a self,
        package: &'a GazettePackage,
    ) -> Pin<Box<dyn Future<Output = Result<GazetteAck, GazetteError>> + Send + 'a>>;
}

/// No-op publisher used in tests and during Wave 3 development.
/// Wired to the real adapter by the application layer in Wave 4.
pub struct NoopPublisher;

impl GazettePublisher for NoopPublisher {
    fn submit<'a>(
        &'a self,
        package: &'a GazettePackage,
    ) -> Pin<Box<dyn Future<Output = Result<GazetteAck, GazetteError>> + Send + 'a>> {
        Box::pin(async move {
            tracing::info!(
                declaration_id = %package.declaration_id,
                "NoopPublisher: gazette submission skipped (Wave 3 stub)"
            );
            Ok(GazetteAck {
                gazette_ref: format!("noop-{}", &package.declaration_id[..8]),
                submitted_at: chrono::Utc::now().to_rfc3339(),
            })
        })
    }
}
