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

import { describe, it, expect } from 'vitest';
import { generateCspNonce, buildCsp, jsonLdScript, canonicalUrl } from './index.js';

describe('generateCspNonce', () => {
	it('returns a non-empty string', () => {
		const n = generateCspNonce();
		expect(typeof n).toBe('string');
		expect(n.length).toBeGreaterThan(0);
	});
	it('returns different values each call', () => {
		const a = generateCspNonce();
		const b = generateCspNonce();
		expect(a).not.toBe(b);
	});
});

describe('buildCsp', () => {
	it('contains nonce in script-src', () => {
		const csp = buildCsp('abc123');
		expect(csp).toContain("'nonce-abc123'");
		expect(csp).toContain('script-src');
	});
	it('contains frame-ancestors none', () => {
		const csp = buildCsp('x');
		expect(csp).toContain("frame-ancestors 'none'");
	});
});

describe('jsonLdScript', () => {
	it('includes @context', () => {
		const s = jsonLdScript({ '@type': 'Organization', name: 'Electoral Authority' });
		const parsed = JSON.parse(s) as Record<string, unknown>;
		expect(parsed['@context']).toBe('https://schema.org');
		expect(parsed['@type']).toBe('Organization');
	});
});

describe('canonicalUrl', () => {
	it('joins base and path correctly', () => {
		expect(canonicalUrl('https://eel.example', '/results')).toBe('https://eel.example/results');
		expect(canonicalUrl('https://eel.example/', '/voter/')).toBe('https://eel.example/voter/');
	});
});
