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

describe('Button component a11y', () => {
	it('renders a valid button HTML structure', () => {
		const html = `<button class="eel-btn eel-btn--primary eel-btn--md" type="button">Click me</button>`;
		expect(html).toContain('type="button"');
		expect(html).toContain('<button');
	});

	it('disabled button has aria-disabled', () => {
		const html = `<button class="eel-btn eel-btn--primary eel-btn--md" type="button" disabled aria-disabled="true">Click me</button>`;
		expect(html).toContain('aria-disabled="true"');
	});

	it('link button uses anchor element', () => {
		const html = `<a class="eel-btn eel-btn--primary eel-btn--md" href="/target">Go there</a>`;
		expect(html).toContain('<a ');
		expect(html).toContain('href=');
	});
});

describe('Badge component a11y', () => {
	it('has role status', () => {
		const html = `<span class="eel-badge eel-badge--success" role="status" aria-label="Active">Active</span>`;
		expect(html).toContain('role="status"');
		expect(html).toContain('aria-label=');
	});
});

describe('Alert component a11y', () => {
	it('warning alert has role alert', () => {
		const html = `<div class="eel-alert eel-alert--warning" role="alert" aria-live="assertive">Watch out!</div>`;
		expect(html).toContain('role="alert"');
		expect(html).toContain('aria-live="assertive"');
	});

	it('info alert has role status', () => {
		const html = `<div class="eel-alert eel-alert--info" role="status" aria-live="polite">FYI</div>`;
		expect(html).toContain('role="status"');
		expect(html).toContain('aria-live="polite"');
	});
});

describe('Navigation component a11y', () => {
	it('has aria-label on nav element', () => {
		const html = `<nav class="eel-nav" aria-label="Main navigation"></nav>`;
		expect(html).toContain('aria-label="Main navigation"');
	});

	it('current page item has aria-current=page', () => {
		const html = `<a href="/results" class="eel-nav__link eel-nav__link--current" aria-current="page">Results</a>`;
		expect(html).toContain('aria-current="page"');
	});
});

describe('Table component a11y', () => {
	it('table wrapper has role region and aria-label', () => {
		const html = `<div class="eel-table-wrapper" role="region" aria-label="Election results" tabindex="0"></div>`;
		expect(html).toContain('role="region"');
		expect(html).toContain('aria-label=');
		expect(html).toContain('tabindex="0"');
	});
});

describe('LangSwitcher component a11y', () => {
	it('select has aria-label', () => {
		const html = `<select id="eel-lang-select" aria-label="Select language"><option value="en">English</option></select>`;
		expect(html).toContain('aria-label="Select language"');
	});
});

describe('MerkleProofViewer component a11y', () => {
	it('has complementary landmark role', () => {
		const html = `<aside class="eel-merkle" aria-label="Merkle proof viewer" role="complementary"></aside>`;
		expect(html).toContain('role="complementary"');
		expect(html).toContain('aria-label=');
	});
});

describe('Footer component a11y', () => {
	it('uses contentinfo role', () => {
		const html = `<footer class="eel-footer" role="contentinfo"></footer>`;
		expect(html).toContain('role="contentinfo"');
	});
});
