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
