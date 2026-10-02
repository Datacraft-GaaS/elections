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

import { defineConfig } from 'astro/config'

export default defineConfig({
  site: 'https://results.eel.local',
  outDir: './dist',
  // Static build: every page pre-rendered from signed publication tarballs.
  // No server-side JS; LiveView handles real-time counts separately.
  output: 'static',
  // Strict CSP nonces injected at edge (ADR-0010); no external scripts.
  // build.inlineStylesheets: 'always' keeps the initial payload minimal (<200 KB per FR-M11-035).
  build: {
    inlineStylesheets: 'always',
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'sw', 'ki', 'luo', 'kln', 'mer', 'kam', 'som', 'orm', 'msa', 'ar'],
    routing: {
      prefixDefaultLocale: false,
    },
  },
})
