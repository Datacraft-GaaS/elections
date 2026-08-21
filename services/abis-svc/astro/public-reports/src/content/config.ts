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
