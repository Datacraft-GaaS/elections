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

use std::time::Instant;

use chrono::Utc;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::Result;
use crate::render::{
    csv::CsvRenderer, html::HtmlRenderer, json::JsonRenderer, parquet::ParquetRenderer,
    pdf::PdfRenderer, png::PngRenderer, xlsx::XlsxRenderer, RenderOutput, Renderer, TemplateBundle,
};
use crate::{anchor, extractor, repository, signer};

const POLL_INTERVAL_MS: u64 = 500;
const BATCH_SIZE: i64 = 10;

/// Run one pass: picks queued runs and processes each through the full pipeline.
pub async fn run_pipeline_pass(
    pool: &PgPool,
    read_pool: &PgPool,
    http_client: &reqwest::Client,
    cancel: CancellationToken,
) {
    let runs = match repository::fetch_queued_runs(pool, BATCH_SIZE).await {
        Ok(r) => r,
        Err(e) => {
            error!(error = %e, "failed to fetch queued runs");
            return;
        }
    };

    for run in runs {
        if cancel.is_cancelled() {
            break;
        }
        let run_id = run.run_id;
        if let Err(e) = process_run(pool, read_pool, http_client, &run, cancel.clone()).await {
            error!(run_id = %run_id, error = %e, "run failed");
            let _ = repository::fail_run(pool, run_id, &e.to_string()).await;
        }
    }
}

/// Continuous pipeline loop.
pub async fn pipeline_loop(
    pool: PgPool,
    read_pool: PgPool,
    http_client: reqwest::Client,
    cancel: CancellationToken,
) {
    info!("pipeline loop started");
    while !cancel.is_cancelled() {
        run_pipeline_pass(&pool, &read_pool, &http_client, cancel.clone()).await;
        tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
    info!("pipeline loop stopped");
}

async fn process_run(
    pool: &PgPool,
    read_pool: &PgPool,
    http_client: &reqwest::Client,
    run: &crate::domain::ReportRun,
    cancel: CancellationToken,
) -> Result<()> {
    let run_id = run.run_id;
    let wall_start = Instant::now();

    let def = repository::fetch_definition_by_slug(pool, &run.slug)
        .await?
        .ok_or_else(|| crate::error::ReportError::NotFound(format!("definition {}", run.slug)))?;

    // queued -> extracting
    if !repository::advance_run_status(pool, run_id, "queued", "extracting").await? {
        warn!(run_id = %run_id, "CAS queued->extracting failed, skipping");
        return Ok(());
    }
    emit_event("eel.abis.report_run_started.v1", run_id, &run.slug);

    let sql_ref = def.sql_template_ref.as_deref().unwrap_or_default();
    let (rows, rows_extracted, extraction_ms) = extractor::extract(
        read_pool,
        run_id,
        sql_ref,
        run.window_from,
        run.window_to,
        None,
        cancel.clone(),
    )
    .await?;

    repository::update_run_extraction_stats(pool, run_id, rows_extracted, extraction_ms).await?;

    // extracting -> rendering
    repository::advance_run_status(pool, run_id, "extracting", "rendering").await?;

    let meta = crate::render::RunMeta {
        slug: run.slug.clone(),
        title: def.title.clone(),
        template_version: def.template_version.clone(),
        window_from: run.window_from,
        window_to: run.window_to,
        generated_at: Utc::now(),
    };
    let template = TemplateBundle::default();

    let mut artifact_ids: Vec<Uuid> = Vec::new();

    for fmt_str in &def.renderer_pipeline {
        if cancel.is_cancelled() {
            return Err(crate::error::ReportError::Cancelled);
        }
        let output = render_format(fmt_str, &rows, &meta, &template)?;
        let artifact_id = Uuid::now_v7();
        let blob_ref = format!("blobs/{run_id}/{fmt_str}");
        let row_count = match fmt_str.as_str() {
            "csv" | "parquet" | "xlsx" | "json" => Some(rows_extracted),
            _ => None,
        };

        repository::insert_artifact(
            pool,
            artifact_id,
            run_id,
            fmt_str,
            &blob_ref,
            &output.sha256,
            output.size_bytes,
            None,
            row_count,
        )
        .await?;

        emit_event("eel.abis.report_artifact_rendered.v1", run_id, fmt_str);
        artifact_ids.push(artifact_id);
    }

    // rendering -> signing (if required)
    if def.requires_hsm_sign {
        repository::advance_run_status(pool, run_id, "rendering", "signing").await?;

        for artifact_id in &artifact_ids {
            let art = repository::fetch_artifact(pool, *artifact_id)
                .await?
                .ok_or_else(|| crate::error::ReportError::NotFound(artifact_id.to_string()))?;
            let sig = signer::sign_artifact(http_client, &art.sha256, "declaration").await?;
            repository::update_artifact_signature(pool, *artifact_id, &sig, "declaration").await?;
        }
        emit_event("eel.abis.report_signed.v1", run_id, &run.slug);

        // signing -> anchoring (if required)
        if def.requires_tlog_anchor {
            repository::advance_run_status(pool, run_id, "signing", "anchoring").await?;

            for artifact_id in &artifact_ids {
                let art = repository::fetch_artifact(pool, *artifact_id)
                    .await?
                    .ok_or_else(|| crate::error::ReportError::NotFound(artifact_id.to_string()))?;
                let (leaf_index, root_hash, proof, sth_size, cosigs) =
                    anchor::anchor_artifact(http_client, *artifact_id, &art.sha256).await?;
                let anchor_id = Uuid::now_v7();
                repository::insert_tlog_anchor(
                    pool,
                    anchor_id,
                    run_id,
                    *artifact_id,
                    leaf_index,
                    &root_hash,
                    &proof,
                    sth_size,
                    cosigs,
                )
                .await?;
            }
            emit_event("eel.abis.report_anchored.v1", run_id, &run.slug);

            // anchoring -> delivering
            repository::advance_run_status(pool, run_id, "anchoring", "delivering").await?;
        } else {
            // signing -> delivering
            repository::advance_run_status(pool, run_id, "signing", "delivering").await?;
        }
    } else {
        // rendering -> delivering
        repository::advance_run_status(pool, run_id, "rendering", "delivering").await?;
    }

    // delivering -> complete
    repository::advance_run_status(pool, run_id, "delivering", "complete").await?;

    let total_ms = wall_start.elapsed().as_millis() as i32;
    sqlx::query("UPDATE eel_report_runs SET total_ms = $1 WHERE run_id = $2")
        .bind(total_ms)
        .bind(run_id)
        .execute(pool)
        .await?;

    emit_event("eel.abis.report_completed.v1", run_id, &run.slug);
    info!(run_id = %run_id, total_ms, "run complete");
    Ok(())
}

fn render_format(
    fmt: &str,
    rows: &crate::render::Rows,
    meta: &crate::render::RunMeta,
    template: &TemplateBundle,
) -> Result<RenderOutput> {
    match fmt {
        "json" => JsonRenderer::render(rows, meta, template),
        "csv" => CsvRenderer::render(rows, meta, template),
        "parquet" => ParquetRenderer::render(rows, meta, template),
        "pdf" => PdfRenderer::render(rows, meta, template),
        "html" => HtmlRenderer::render(rows, meta, template),
        "xlsx" => XlsxRenderer::render(rows, meta, template),
        "png" => PngRenderer::render(rows, meta, template),
        other => Err(crate::error::ReportError::Renderer(format!(
            "unknown format: {other}"
        ))),
    }
}

fn emit_event(event_type: &str, run_id: Uuid, context: &str) {
    info!(event_type, run_id = %run_id, context, "cloudevent");
}
