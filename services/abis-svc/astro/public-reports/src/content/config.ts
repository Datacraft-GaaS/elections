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

import { defineCollection, z } from "astro:content";

const publishedReports = defineCollection({
  type: "data",
  schema: z.object({
    slug: z.string(),
    title: z.record(z.string(), z.string()), // locale -> title
    report_class: z.enum([
      "operational",
      "statistical",
      "regulatory",
      "legal",
      "public",
      "financial",
      "executive",
      "ad_hoc",
    ]),
    window_from: z.string(), // ISO 8601
    window_to: z.string(),
    formats: z.object({
      pdf: z.string().url().optional(),
      csv: z.string().url().optional(),
      json: z.string().url().optional(),
      html: z.string().url().optional(),
    }),
    sha256: z.object({
      pdf: z.string().optional(),
      csv: z.string().optional(),
      json: z.string().optional(),
      html: z.string().optional(),
    }),
    tlog_leaf_index: z.number().optional(),
    tlog_root_hash: z.string().optional(), // hex
    cosign_sig: z.string().optional(), // base64
    published_at: z.string(),
    generator_svid: z.string().optional(),
    template_version: z.string().optional(),
  }),
});

export const collections = {
  reports: publishedReports,
};
