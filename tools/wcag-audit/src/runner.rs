//! Embedded Node.js runner script for axe-core evaluation via Playwright.

pub const RUNNER_SCRIPT: &str = r#"
// eel-wcag-runner.js - invoked by eel-wcag-audit
// Usage: node eel-wcag-runner.js <url> <level>
const { chromium } = require('playwright');
const axe = require('axe-core');

(async () => {
  const url = process.argv[2];
  const level = process.argv[3] || 'AA';
  if (!url) {
    console.error('Usage: node eel-wcag-runner.js <url> <level>');
    process.exit(1);
  }
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
    await page.addScriptTag({ content: axe.source });
    const results = await page.evaluate(async (lvl) => {
      const opts = lvl === 'AAA'
        ? { runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag2aaa', 'wcag22aa'] } }
        : { runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag22aa'] } };
      return await window.axe.run(document, opts);
    }, level);
    await browser.close();
    console.log(JSON.stringify(results));
  } catch (err) {
    if (browser) await browser.close().catch(() => {});
    console.error('eel-wcag-runner error: ' + err.message);
    process.exit(3);
  }
})();
"#;
