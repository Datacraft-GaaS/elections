#!/usr/bin/env node
// tools/pseudo-gen.mjs
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dir = dirname(fileURLToPath(import.meta.url));
const root = join(__dir, '..');
const enDir = join(root, 'locales', 'en');
const outDir = join(root, 'locales', 'pseudo');

const FILES = ['common', 'voter', 'results', 'grievance', 'errors', 'accessibility'];

function accent(str) {
  return str
    .replace(/a/g, 'ä').replace(/e/g, 'ë').replace(/i/g, 'ï')
    .replace(/o/g, 'ö').replace(/u/g, 'ü')
    .replace(/A/g, 'Ä').replace(/E/g, 'Ë').replace(/I/g, 'Ï')
    .replace(/O/g, 'Ö').replace(/U/g, 'Ü');
}

function pseudoValue(v) {
  if (typeof v !== 'string') return v;
  // Preserve ICU placeholders {x}, keep them intact
  const parts = v.split(/(\{[^}]+\})/g);
  const expanded = parts.map(p => p.startsWith('{') ? p : accent(p)).join('');
  return `[!!! ${expanded} !!!]`;
}

mkdirSync(outDir, { recursive: true });

for (const file of FILES) {
  const src = JSON.parse(readFileSync(join(enDir, `${file}.json`), 'utf8'));
  const pseudo = Object.fromEntries(Object.entries(src).map(([k, v]) => [k, pseudoValue(v)]));
  writeFileSync(join(outDir, `${file}.json`), JSON.stringify(pseudo, null, 2) + '\n');
}

console.log('Pseudo locale written to locales/pseudo/');
