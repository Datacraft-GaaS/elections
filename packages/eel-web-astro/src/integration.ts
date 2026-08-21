/**
 * Astro integration for eel-web-astro.
 *
 * Provides:
 *   - Injects CSP nonce into Astro middleware
 *   - Adds sitemap and RSS configuration
 *   - Wires up i18n routing
 */
import type { AstroIntegration } from 'astro';

export interface EelWebOptions {
	/** Base URL for canonical links and sitemap. */
	siteUrl: string;
	/** Supported locales. Default: all 11 eel locales. */
	locales?: string[];
	/** Default locale. Default: 'en'. */
	defaultLocale?: string;
	/** Enable RSS feed. Default: true. */
	rss?: boolean;
	/** Enable sitemap. Default: true. */
	sitemap?: boolean;
}

export default function eelWebAstro(options: EelWebOptions): AstroIntegration {
	const {
		siteUrl,
		locales = ['en', 'sw', 'ki', 'luo', 'luy', 'kln', 'kam', 'mer', 'guz', 'so', 'oro'],
		defaultLocale = 'en',
		rss: _rss = true,
		sitemap: _sitemap = true,
	} = options;

	return {
		name: '@eel/web-astro',
		hooks: {
			'astro:config:setup': ({ injectRoute, logger }) => {
				logger.info(`eel-web-astro: site=${siteUrl}, locales=${locales.join(',')}`);
				injectRoute({
					pattern: '/_eel/health',
					entrypoint: '@eel/web-astro/health',
				});
			},
			'astro:config:done': ({ config }) => {
				if (locales.length > 1 && !config.i18n) {
					console.warn(`[eel-web-astro] Multiple locales configured but Astro i18n not set up. Add i18n config to astro.config.mjs. (default: ${defaultLocale})`);
				}
			},
		},
	};
}
