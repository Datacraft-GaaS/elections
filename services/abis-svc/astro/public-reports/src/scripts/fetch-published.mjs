#!/usr/bin/env node
/**
 * Copyright (c) 2026 Nyimbi Odero
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Fetches published public reports from the reports API and writes
 * them to src/data/reports.json for the static build.
 *
 * Usage:
 *   node src/scripts/fetch-published.mjs
 *
 * Environment variables:
 *   REPORTS_API_URL  - Base URL for the reports API (default: http://localhost:8080)
 *   REPORTS_TOKEN    - Bearer token for auth
 */

import { writeFileSync, mkdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUTPUT = join(__dirname, "../data/reports.json");

const BASE_URL = process.env.REPORTS_API_URL ?? "http://localhost:8080";
const TOKEN = process.env.REPORTS_TOKEN ?? "";

async function fetchPublished() {
  const headers = TOKEN ? { Authorization: `Bearer ${TOKEN}` } : {};
  const url = `${BASE_URL}/api/v1/reports/definitions?class=public&active_only=true`;

  console.log(`Fetching from ${url}`);
  const defsResp = await fetch(url, { headers });
  if (!defsResp.ok) {
    throw new Error(`Failed to fetch definitions: ${defsResp.status} ${defsResp.statusText}`);
  }
  const defs = await defsResp.json();

  const reports = [];
  for (const def of defs) {
    const runsUrl = `${BASE_URL}/api/v1/reports/runs?slug=${def.slug}&status=complete&limit=5`;
    const runsResp = await fetch(runsUrl, { headers });
    if (!runsResp.ok) continue;
    const runs = await runsResp.json();

    for (const run of runs) {
      const artifactsUrl = `${BASE_URL}/api/v1/reports/runs/${run.run_id}/artifacts`;
      const artResp = await fetch(artifactsUrl, { headers });
      if (!artResp.ok) continue;
      const artifacts = await artResp.json();

      const formats = {};
      const sha256 = {};
      for (const art of artifacts) {
        if (["pdf", "csv", "json", "html"].includes(art.format)) {
          formats[art.format] = `${BASE_URL}/api/v1/artifacts/${art.artifact_id}/download`;
          sha256[art.format] = art.sha256_hex;
        }
      }

      reports.push({
        slug: run.slug,
        run_id: run.run_id,
        title: { en: def.title },
        report_class: def.report_class,
        window_from: run.window_from,
        window_to: run.window_to,
        formats,
        sha256,
        tlog_leaf_index: run.tlog_leaf_index ?? null,
        tlog_root_hash: run.tlog_root_hash ?? null,
        published_at: run.completed_at ?? run.requested_at,
        template_version: def.template_version,
      });
    }
  }

  mkdirSync(dirname(OUTPUT), { recursive: true });
  writeFileSync(OUTPUT, JSON.stringify(reports, null, 2));
  console.log(`Wrote ${reports.length} report(s) to ${OUTPUT}`);
}

fetchPublished().catch((err) => {
  console.error("fetch-published failed:", err.message);
  console.error("The build will use the committed placeholder reports.json.");
  process.exit(0); // Do not fail the build on fetch errors.
});
