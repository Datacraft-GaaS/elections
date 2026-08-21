// RSS 2.0 feed of published result snapshots (FR-M11-013, AC-M11-11).
// No PII in feed items (FR-M11-026).

import type { APIRoute } from 'astro'

function escapeXml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')
}

export const GET: APIRoute = async ({ site }) => {
  const siteUrl = site?.toString() ?? 'https://results.eel.local'

  // Production: load published snapshots from signed tarball loader.
  const items: Array<{ title: string; link: string; pubDate: string; description: string }> = []

  const rss = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Election Results Feed</title>
    <link>${escapeXml(siteUrl)}</link>
    <description>Official election result snapshots from the eel portal.</description>
    <language>en</language>
    <atom:link href="${escapeXml(siteUrl + '/rss.xml')}" rel="self" type="application/rss+xml" />
    ${items
      .map(
        (item) => `
    <item>
      <title>${escapeXml(item.title)}</title>
      <link>${escapeXml(item.link)}</link>
      <pubDate>${item.pubDate}</pubDate>
      <description>${escapeXml(item.description)}</description>
      <guid isPermaLink="true">${escapeXml(item.link)}</guid>
    </item>`,
      )
      .join('')}
  </channel>
</rss>`

  return new Response(rss, {
    headers: {
      'Content-Type': 'application/rss+xml; charset=utf-8',
      'Cache-Control': 'public, max-age=60, s-maxage=60',
    },
  })
}
