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

import type { APIRoute } from "astro";
import reports from "../../data/reports.json";

export const GET: APIRoute = ({ site }) => {
  const baseUrl = site?.toString().replace(/\/$/, "") ?? "https://reports.eel.internal";

  const entries = [...reports]
    .sort((a, b) => b.published_at.localeCompare(a.published_at))
    .slice(0, 50)
    .map((r) => {
      const title = (r.title as Record<string, string>).en ?? r.slug;
      const link = `${baseUrl}/en/reports/${r.slug}/${r.run_id}/`;
      const updated = new Date(r.published_at).toISOString();
      const tlogAnchor = r.tlog_leaf_index != null
        ? `<eel:tlog_leaf_index>${r.tlog_leaf_index}</eel:tlog_leaf_index>
         <eel:tlog_root_hash>${r.tlog_root_hash ?? ""}</eel:tlog_root_hash>`
        : "";
      return `
  <entry>
    <title type="text"><![CDATA[${title}]]></title>
    <link href="${link}" rel="alternate" type="text/html" />
    <id>${link}</id>
    <updated>${updated}</updated>
    <summary type="text">${r.report_class} report for window ${r.window_from.split("T")[0]} to ${r.window_to.split("T")[0]}</summary>
    ${tlogAnchor}
  </entry>`;
    })
    .join("\n");

  const now = new Date().toISOString();
  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"
      xmlns:eel="https://reports.eel.internal/ns/1.0">
  <title>ABIS Public Reports</title>
  <link href="${baseUrl}/" rel="alternate" />
  <link href="${baseUrl}/api/atom.xml" rel="self" />
  <id>${baseUrl}/api/atom.xml</id>
  <updated>${now}</updated>
  <subtitle>Newly published ABIS transparency reports with transparency-log anchors</subtitle>
  ${entries}
</feed>`;

  return new Response(xml, {
    headers: {
      "Content-Type": "application/atom+xml; charset=utf-8",
      "Cache-Control": "public, max-age=3600",
    },
  });
};
