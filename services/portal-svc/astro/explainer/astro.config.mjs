import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";
import mdx from "@astrojs/mdx";

export default defineConfig({
  integrations: [tailwind(), mdx()],
  i18n: {
    defaultLocale: "en",
    locales: [
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
    ],
    routing: {
      prefixDefaultLocale: false,
    },
  },
  site: "https://portal.eel.internal",
});
