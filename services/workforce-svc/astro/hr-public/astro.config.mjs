import { defineConfig } from 'astro/config'

export default defineConfig({
  site: 'https://apply.iebc.or.ke',
  outDir: './dist',
  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'sw'],
    routing: {
      prefixDefaultLocale: false,
    },
  },
})
