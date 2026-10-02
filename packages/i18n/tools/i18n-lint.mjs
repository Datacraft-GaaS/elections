#!/usr/bin/env node
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

// tools/i18n-lint.mjs
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { exit } from 'node:process';

const __dir = dirname(fileURLToPath(import.meta.url));
const root = join(__dir, '..');
const localesDir = join(root, 'locales');

const FILES = ['common', 'voter', 'results', 'grievance', 'errors', 'accessibility'];
const LOCALES = ['en', 'sw', 'ki', 'luo', 'luy', 'kln', 'kam', 'mer', 'guz', 'so', 'oro'];

let errors = 0;

// Load English as reference
const en = {};
for (const f of FILES) {
  const path = join(localesDir, 'en', `${f}.json`);
  en[f] = JSON.parse(readFileSync(path, 'utf8'));
}

// Check all other locales
for (const locale of LOCALES.filter(l => l !== 'en')) {
  for (const f of FILES) {
    const path = join(localesDir, locale, `${f}.json`);
    if (!existsSync(path)) {
      console.error(`MISSING FILE: locales/${locale}/${f}.json`);
      errors++;
      continue;
    }
    const data = JSON.parse(readFileSync(path, 'utf8'));
    for (const key of Object.keys(en[f])) {
      if (!(key in data)) {
        console.error(`MISSING KEY [${locale}/${f}]: ${key}`);
        errors++;
      }
    }
  }
}

if (errors > 0) {
  console.error(`\ni18n lint failed: ${errors} error(s)`);
  exit(1);
} else {
  console.log(`i18n lint passed: all ${LOCALES.length - 1} non-English locales complete for ${FILES.length} files.`);
}
