#!/usr/bin/env node
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
