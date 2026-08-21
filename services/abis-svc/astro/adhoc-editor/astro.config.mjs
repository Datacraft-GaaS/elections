import { defineConfig } from 'astro/config';
import preact from '@astrojs/preact';
import mdx from '@astrojs/mdx';

export default defineConfig({
  output: 'static',
  integrations: [
    preact({ compat: false }),
    mdx(),
  ],
  i18n: {
    defaultLocale: 'en',
    locales: [
      'en', 'sw', 'fr', 'ar', 'so', 'om', 'ti', 'ha', 'yo', 'ig', 'am',
    ],
    routing: {
      prefixDefaultLocale: false,
    },
  },
});
