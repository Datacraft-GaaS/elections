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
 * Client-side Markdown to HTML rendering with basic sanitization.
 * Uses marked for parsing; sanitization strips dangerous tags/attributes.
 */

import { marked } from 'marked';

const ALLOWED_TAGS = new Set([
  'p', 'br', 'strong', 'em', 'code', 'pre', 'ul', 'ol', 'li',
  'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'blockquote', 'hr',
  'table', 'thead', 'tbody', 'tr', 'th', 'td', 'a', 'span', 'div',
]);

const DANGEROUS_PATTERNS = [
  /javascript:/gi,
  /on\w+=/gi,
  /data:/gi,
  /<script/gi,
  /<iframe/gi,
  /<object/gi,
  /<embed/gi,
];

export function sanitizeHtml(html: string): string {
  let safe = html;
  for (const pattern of DANGEROUS_PATTERNS) {
    safe = safe.replace(pattern, '');
  }
  return safe;
}

export function renderMarkdown(md: string): string {
  const html = marked.parse(md, { async: false }) as string;
  return sanitizeHtml(html);
}
