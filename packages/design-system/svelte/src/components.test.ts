import { describe, it, expect } from 'vitest';

describe('Svelte design system', () => {
	it('exports version', async () => {
		const { version } = await import('./index.js');
		expect(version).toBe('0.1.0');
	});
});
