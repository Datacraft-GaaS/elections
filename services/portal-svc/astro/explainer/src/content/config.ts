import { defineCollection, z } from "astro:content";

const localisedText = z.object({
  en: z.string(),
  sw: z.string().optional(),
  ki: z.string().optional(),
  luo: z.string().optional(),
  kal: z.string().optional(),
  kam: z.string().optional(),
  guj: z.string().optional(),
  mer: z.string().optional(),
  mas: z.string().optional(),
  "ksl-glossary": z.string().optional(),
  som: z.string().optional(),
});

const referendumQuestionsCollection = defineCollection({
  type: "content",
  schema: z.object({
    id: z.string(),
    title: localisedText,
    summary: localisedText,
    full_text: localisedText,
    fact_check_annotations: z
      .array(
        z.object({
          claim: z.string(),
          verdict: z.enum(["true", "false", "misleading", "unverified"]),
          source_url: z.string().url().optional(),
        })
      )
      .default([]),
    ksl_video_url: z.string().url().optional(),
    tts_audio_url: z.string().url().optional(),
  }),
});

const explainerContentCollection = defineCollection({
  type: "content",
  schema: z.object({
    id: z.string(),
    title: localisedText,
    summary: localisedText,
    full_text: localisedText,
    ksl_video_url: z.string().url().optional(),
    tts_audio_url: z.string().url().optional(),
  }),
});

export const collections = {
  questions: referendumQuestionsCollection,
  explainers: explainerContentCollection,
};
