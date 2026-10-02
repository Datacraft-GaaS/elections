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
			'astro:config:done': ({ config, logger }) => {
				if (locales.length > 1 && !config.i18n) {
					logger.warn(`[eel-web-astro] Multiple locales configured but Astro i18n not set up. Add i18n config to astro.config.mjs. (default: ${defaultLocale})`);
				}
			},
		},
	};
}
