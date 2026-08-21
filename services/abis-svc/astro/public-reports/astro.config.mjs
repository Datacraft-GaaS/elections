import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";

const LOCALES = [
  "en",
  "sw",
  "ki",
  "luo",
  "kal",
  "kam",
  "guj",
  "mer",
  "mas",
  "ksl-glossary",
  "som",
];

export default defineConfig({
  site: process.env.SITE_URL ?? "https://reports.eel.internal",
  integrations: [
    tailwind(),
    mdx(),
    sitemap({
      i18n: {
        defaultLocale: "en",
        locales: Object.fromEntries(LOCALES.map((l) => [l, l])),
      },
    }),
  ],
  i18n: {
    defaultLocale: "en",
    locales: LOCALES,
    routing: {
      prefixDefaultLocale: false,
    },
  },
  output: "static",
});
