/**
 * i18n scaffolding for eel-web-astro.
 *
 * Wraps packages/i18n/ locale catalogues and provides:
 *   - getLocale(): infer locale from Astro.request URL or cookie
 *   - t(key, params?): translate a key in the current locale
 *   - supportedLocales: list of supported locale codes
 */

export const supportedLocales = ['en', 'sw', 'ki', 'luo', 'luy', 'kln', 'kam', 'mer', 'guz', 'so', 'oro'] as const;
export type Locale = (typeof supportedLocales)[number];

export const defaultLocale: Locale = 'en';

export function isLocale(code: string): code is Locale {
	return (supportedLocales as readonly string[]).includes(code);
}

/** Extract locale from a URL pathname like /sw/voter or /en/ */
export function localeFromPath(pathname: string): Locale {
	const segment = pathname.split('/').find(Boolean) ?? '';
	return isLocale(segment) ? segment : defaultLocale;
}

/** Extract locale from Accept-Language header (first matching supported locale). */
export function localeFromHeader(acceptLanguage: string): Locale {
	for (const part of acceptLanguage.split(',')) {
		const code = part.trim().split(';')[0]!.trim().slice(0, 2).toLowerCase();
		if (isLocale(code)) return code;
	}
	return defaultLocale;
}

export type MessageCatalogue = Record<string, string>;

/** Lazy-load a message catalogue for a given locale and file. */
export async function loadCatalogue(locale: Locale, file: string): Promise<MessageCatalogue> {
	try {
		const mod = await import(`@eel/i18n/locales/${locale}/${file}.json`, { assert: { type: 'json' } });
		return mod.default as MessageCatalogue;
	} catch {
		return {};
	}
}

/** Interpolate ICU-like placeholders in a message string. */
export function interpolate(message: string, params: Record<string, string | number> = {}): string {
	return message.replace(/\{(\w+)\}/g, (_, key: string) => String(params[key] ?? `{${key}}`));
}

export type TranslateFn = (key: string, params?: Record<string, string | number>) => string;

/** Build a translate function for a given catalogue. */
export function makeTranslator(catalogue: MessageCatalogue, fallback: MessageCatalogue = {}): TranslateFn {
	return (key, params) => {
		const raw = catalogue[key] ?? fallback[key] ?? key;
		return interpolate(raw, params);
	};
}
