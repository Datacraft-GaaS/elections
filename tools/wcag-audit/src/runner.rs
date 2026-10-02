// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
