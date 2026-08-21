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
		const s = jsonLdScript({ '@type': 'Organization', name: 'IEBC' });
		const parsed = JSON.parse(s) as Record<string, unknown>;
		expect(parsed['@context']).toBe('https://schema.org');
		expect(parsed['@type']).toBe('Organization');
	});
});

describe('canonicalUrl', () => {
	it('joins base and path correctly', () => {
		expect(canonicalUrl('https://iebc.or.ke', '/results')).toBe('https://iebc.or.ke/results');
		expect(canonicalUrl('https://iebc.or.ke/', '/voter/')).toBe('https://iebc.or.ke/voter/');
	});
});
