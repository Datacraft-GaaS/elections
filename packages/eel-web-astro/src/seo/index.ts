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
 * SEO helpers for eel-web-astro.
 * Provides canonical URL, Open Graph, and structured data utilities.
 */

export interface SeoProps {
	title: string;
	description: string;
	canonical?: string;
	locale?: string;
	ogImage?: string;
	ogType?: 'website' | 'article';
	noIndex?: boolean;
	jsonLd?: Record<string, unknown>;
}

/** Build a CSP nonce (16 random bytes, base64-encoded). Call once per request. */
export function generateCspNonce(): string {
	if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
		const bytes = new Uint8Array(16);
		crypto.getRandomValues(bytes);
		return btoa(String.fromCharCode(...bytes));
	}
	return Math.random().toString(36).slice(2) + Math.random().toString(36).slice(2);
}

/** Build a Content-Security-Policy header value. */
export function buildCsp(nonce: string, extra: Record<string, string> = {}): string {
	const directives: Record<string, string> = {
		'default-src': "'self'",
		'script-src': `'self' 'nonce-${nonce}'`,
		'style-src': `'self' 'nonce-${nonce}' https://fonts.googleapis.com`,
		'font-src': "'self' https://fonts.gstatic.com",
		'img-src': "'self' data:",
		'connect-src': "'self'",
		'frame-ancestors': "'none'",
		'form-action': "'self'",
		'base-uri': "'self'",
		...extra,
	};
	return Object.entries(directives)
		.map(([k, v]) => `${k} ${v}`)
		.join('; ');
}

/** Render JSON-LD script tag content as a string. */
export function jsonLdScript(data: Record<string, unknown>): string {
	return JSON.stringify({ '@context': 'https://schema.org', ...data });
}

/** Construct a canonical URL from base URL + path. */
export function canonicalUrl(base: string, path: string): string {
	return `${base.replace(/\/$/, '')}/${path.replace(/^\//, '')}`;
}
