//! Evidence bundle builder.
//!
//! Assembles a court-admissible evidence bundle for a telemetry query result:
//!   - PDF report (printpdf)
//!   - CSV export (csv crate)
//!   - Parquet export (parquet + arrow)
//!   - SHA-256 manifest over all three artefacts
//!   - HSM-signed manifest stub (real signing via eel-crypto-rs in production)
//!   - tlog anchor placeholder (Merkle inclusion proof)
//!
//! The bundle is written to a `tempfile::TempDir` and the paths returned so
//! the HTTP handler can stream each file to the client.

use std::io::Write as IoWrite;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::dsl::TelemetryQuery;
use crate::executor::Row;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BundleError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("PDF error: {0}")]
    Pdf(String),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("arrow error: {0}")]
    Arrow(String),
}

// ---------------------------------------------------------------------------
// Bundle manifest
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub bundle_id: String,
    pub query_id: String,
    pub actor_svid: String,
    pub executed_at: DateTime<Utc>,
    pub row_count: usize,
    pub pdf_sha256: String,
    pub csv_sha256: String,
    pub parquet_sha256: String,
    pub manifest_sha256: String,
    pub tlog_anchor: Option<String>,
    /// HSM signature over manifest_sha256 (hex). Stub in dev.
    pub hsm_signature: String,
}

// ---------------------------------------------------------------------------
// Evidence bundle
// ---------------------------------------------------------------------------

pub struct EvidenceBundle {
    pub manifest: BundleManifest,
    pub pdf_path: PathBuf,
    pub csv_path: PathBuf,
    pub parquet_path: PathBuf,
    pub manifest_path: PathBuf,
    /// Temp dir kept alive as long as this bundle exists.
    _dir: tempfile::TempDir,
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

pub struct EvidenceBundleBuilder {
    pub query_id: String,
    pub actor_svid: String,
    pub query: TelemetryQuery,
    pub rows: Vec<Row>,
    pub executed_at: DateTime<Utc>,
}

impl EvidenceBundleBuilder {
    pub fn build(self) -> Result<EvidenceBundle, BundleError> {
        let dir = tempfile::TempDir::new()?;
        let bundle_id = uuid::Uuid::now_v7().to_string();

        // --- CSV ---
        let csv_path = dir.path().join("result.csv");
        let csv_sha256 = write_csv(&csv_path, &self.rows)?;

        // --- Parquet ---
        let parquet_path = dir.path().join("result.parquet");
        let parquet_sha256 = write_parquet(&parquet_path, &self.rows)?;

        // --- PDF ---
        let pdf_path = dir.path().join("report.pdf");
        let pdf_sha256 = write_pdf(
            &pdf_path,
            &bundle_id,
            &self.query_id,
            &self.actor_svid,
            self.executed_at,
            self.rows.len(),
        )?;

        // --- Manifest ---
        let partial = format!(
            "{}:{}:{}:{}",
            pdf_sha256, csv_sha256, parquet_sha256, bundle_id
        );
        let manifest_sha256 = hex::encode(Sha256::digest(partial.as_bytes()));

        // HSM signature stub: in production call eel-crypto-rs::hsm::sign_detached
        let hsm_signature = format!("stub:sha256:{}", &manifest_sha256[..16]);

        let manifest = BundleManifest {
            bundle_id: bundle_id.clone(),
            query_id: self.query_id.clone(),
            actor_svid: self.actor_svid.clone(),
            executed_at: self.executed_at,
            row_count: self.rows.len(),
            pdf_sha256,
            csv_sha256,
            parquet_sha256,
            manifest_sha256,
            tlog_anchor: None, // populated by tlog anchoring job
            hsm_signature,
        };

        let manifest_path = dir.path().join("manifest.json");
        let manifest_json = serde_json::to_vec_pretty(&manifest)?;
        std::fs::write(&manifest_path, &manifest_json)?;

        Ok(EvidenceBundle {
            manifest,
            pdf_path,
            csv_path,
            parquet_path,
            manifest_path,
            _dir: dir,
        })
    }
}

// ---------------------------------------------------------------------------
// CSV writer
// ---------------------------------------------------------------------------

fn write_csv(path: &PathBuf, rows: &[Row]) -> Result<String, BundleError> {
    let mut wtr = csv::Writer::from_path(path)?;
    let mut hasher = Sha256::new();

    if let Some(first) = rows.first() {
        if let Some(obj) = first.as_object() {
            let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
            wtr.write_record(&headers)?;
            let hdr_line = headers.join(",");
            hasher.update(hdr_line.as_bytes());

            for row in rows {
                if let Some(obj) = row.as_object() {
                    let record: Vec<String> = headers.iter().map(|k| obj[*k].to_string()).collect();
                    wtr.write_record(&record)?;
                    let line = record.join(",");
                    hasher.update(line.as_bytes());
                }
            }
        }
    }

    wtr.flush()?;
    Ok(hex::encode(hasher.finalize()))
}

// ---------------------------------------------------------------------------
// Parquet writer
// ---------------------------------------------------------------------------

fn write_parquet(path: &PathBuf, rows: &[Row]) -> Result<String, BundleError> {
    use arrow_array::{ArrayRef, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field as ArrowField, Schema};
    use parquet::arrow::ArrowWriter;
    use std::sync::Arc;

    // Collect all keys across all rows for the schema
    let mut keys: Vec<String> = vec![];
    for row in rows {
        if let Some(obj) = row.as_object() {
            for k in obj.keys() {
                if !keys.contains(k) {
                    keys.push(k.clone());
                }
            }
        }
    }

    if keys.is_empty() {
        // Write empty parquet file
        let schema = Arc::new(Schema::new(vec![ArrowField::new(
            "empty",
            DataType::Utf8,
            true,
        )]));
        let file = std::fs::File::create(path)?;
        let mut writer =
            ArrowWriter::try_new(file, schema.clone(), None).map_err(BundleError::Parquet)?;
        writer.close().map_err(BundleError::Parquet)?;
        return Ok(hex::encode(Sha256::digest(b"")));
    }

    let fields: Vec<ArrowField> = keys
        .iter()
        .map(|k| ArrowField::new(k.as_str(), DataType::Utf8, true))
        .collect();
    let schema = Arc::new(Schema::new(fields));

    let columns: Vec<ArrayRef> = keys
        .iter()
        .map(|k| {
            let values: Vec<Option<String>> = rows
                .iter()
                .map(|row| {
                    row.as_object()
                        .and_then(|o| o.get(k))
                        .map(|v| v.to_string())
                })
                .collect();
            Arc::new(StringArray::from(values)) as ArrayRef
        })
        .collect();

    let batch = RecordBatch::try_new(schema.clone(), columns)
        .map_err(|e| BundleError::Arrow(e.to_string()))?;

    let file = std::fs::File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, schema, None).map_err(BundleError::Parquet)?;
    writer.write(&batch).map_err(BundleError::Parquet)?;
    writer.close().map_err(BundleError::Parquet)?;

    let bytes = std::fs::read(path)?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

// ---------------------------------------------------------------------------
// PDF writer
// ---------------------------------------------------------------------------

fn write_pdf(
    path: &PathBuf,
    bundle_id: &str,
    query_id: &str,
    actor_svid: &str,
    executed_at: DateTime<Utc>,
    row_count: usize,
) -> Result<String, BundleError> {
    use printpdf::*;

    let (doc, page1, layer1) =
        PdfDocument::new("Telemetry Evidence Report", Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(page1).get_layer(layer1);

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| BundleError::Pdf(e.to_string()))?;

    let lines = vec![
        "EEL Telemetry Evidence Report".to_string(),
        format!("Bundle ID: {}", bundle_id),
        format!("Query ID: {}", query_id),
        format!("Actor SVID: {}", actor_svid),
        format!("Executed at: {}", executed_at.to_rfc3339()),
        format!("Row count: {}", row_count),
        String::new(),
        "This document is a court-admissible evidence bundle.".into(),
        "Verify the HSM signature and tlog anchor in manifest.json.".into(),
    ];

    let mut y = Mm(270.0);
    for line in &lines {
        layer.use_text(line, 12.0, Mm(15.0), y, &font);
        y -= Mm(8.0);
    }

    let bytes = doc
        .save_to_bytes()
        .map_err(|e| BundleError::Pdf(e.to_string()))?;
    std::fs::write(path, &bytes)?;

    Ok(hex::encode(Sha256::digest(&bytes)))
}
