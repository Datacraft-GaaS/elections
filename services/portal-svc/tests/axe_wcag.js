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

/**
 * WCAG 2.2 AA axe-core test via Playwright (AC-M11-07, M11-A11Y-).
 * Run: node tests/axe_wcag.js
 *
 * Requires portal-svc Astro static site to be built and served locally.
 * Zero axe violations required per DoD.
 */
const { chromium } = require('playwright')
const { default: axe } = require('@axe-core/playwright')

const PAGES = [
  'http://localhost:4321/',
  'http://localhost:4321/transparency',
  'http://localhost:4321/history',
]

;(async () => {
  const browser = await chromium.launch()
  let totalViolations = 0

  for (const url of PAGES) {
    const page = await browser.newPage()
    await page.goto(url, { waitUntil: 'networkidle' })

    const results = await new axe(page).analyze()
    if (results.violations.length > 0) {
      console.error(`[FAIL] ${url} — ${results.violations.length} axe violation(s):`)
      for (const v of results.violations) {
        console.error(`  [${v.impact}] ${v.id}: ${v.description}`)
        for (const node of v.nodes) {
          console.error(`    > ${node.target.join(', ')}`)
        }
      }
      totalViolations += results.violations.length
    } else {
      console.log(`[PASS] ${url} — 0 violations`)
    }
    await page.close()
  }

  await browser.close()
  if (totalViolations > 0) {
    console.error(`\nTotal axe violations: ${totalViolations} (DoD requires 0)`)
    process.exit(1)
  }
  console.log('\nAll pages passed WCAG 2.2 AA axe checks.')
})()
