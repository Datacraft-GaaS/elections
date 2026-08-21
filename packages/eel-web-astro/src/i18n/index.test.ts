import { describe, it, expect } from 'vitest';
import {
	localeFromPath,
	localeFromHeader,
	interpolate,
	makeTranslator,
	isLocale,
	supportedLocales,
	defaultLocale,
} from './index.js';

describe('isLocale', () => {
	it('returns true for valid locales', () => {
		for (const l of supportedLocales) {
			expect(isLocale(l)).toBe(true);
		}
	});
	it('returns false for invalid locales', () => {
		expect(isLocale('zz')).toBe(false);
		expect(isLocale('fr')).toBe(false);
		expect(isLocale('')).toBe(false);
	});
});

describe('localeFromPath', () => {
	it('extracts locale from path', () => {
		expect(localeFromPath('/sw/voter')).toBe('sw');
		expect(localeFromPath('/en/results')).toBe('en');
		expect(localeFromPath('/ki/')).toBe('ki');
	});
	it('returns defaultLocale for unknown or empty path', () => {
		expect(localeFromPath('/')).toBe(defaultLocale);
		expect(localeFromPath('/fr/page')).toBe(defaultLocale);
		expect(localeFromPath('')).toBe(defaultLocale);
	});
});

describe('localeFromHeader', () => {
	it('picks first matching locale', () => {
		expect(localeFromHeader('sw, en;q=0.9')).toBe('sw');
		expect(localeFromHeader('fr, en;q=0.8')).toBe('en');
		expect(localeFromHeader('de')).toBe(defaultLocale);
	});
});

describe('interpolate', () => {
	it('replaces placeholders', () => {
		expect(interpolate('Hello {name}!', { name: 'Alice' })).toBe('Hello Alice!');
		expect(interpolate('{count} items', { count: 42 })).toBe('42 items');
	});
	it('leaves unknown placeholders intact', () => {
		expect(interpolate('Hello {name}', {})).toBe('Hello {name}');
	});
});

describe('makeTranslator', () => {
	const catalogue = { 'app.name': 'IEMS Portal', 'nav.home': 'Home' };
	const t = makeTranslator(catalogue);

	it('translates known keys', () => {
		expect(t('app.name')).toBe('IEMS Portal');
	});
	it('returns key for unknown keys', () => {
		expect(t('missing.key')).toBe('missing.key');
	});
	it('falls back to fallback catalogue', () => {
		const t2 = makeTranslator({}, catalogue);
		expect(t2('nav.home')).toBe('Home');
	});
	it('interpolates params', () => {
		const t3 = makeTranslator({ 'greeting': 'Hello {name}' });
		expect(t3('greeting', { name: 'Bob' })).toBe('Hello Bob');
	});
});
