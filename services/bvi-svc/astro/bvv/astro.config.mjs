import { defineConfig } from 'astro/config';

export default defineConfig({
  output: 'static',
  site: 'https://bvv.eel.io',
  compressHTML: true,
  build: {
    assets: '_assets'
  }
});
