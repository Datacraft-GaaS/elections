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
