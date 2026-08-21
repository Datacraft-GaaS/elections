import type { APIRoute } from "astro";
import reports from "../../data/reports.json";

export const GET: APIRoute = ({ site }) => {
  const baseUrl = site?.toString().replace(/\/$/, "") ?? "https://reports.eel.internal";

  const items = [...reports]
    .sort((a, b) => b.published_at.localeCompare(a.published_at))
    .slice(0, 50)
    .map((r) => {
      const title = (r.title as Record<string, string>).en ?? r.slug;
      const link = `${baseUrl}/en/reports/${r.slug}/${r.run_id}/`;
      const pubDate = new Date(r.published_at).toUTCString();
      const fmts = (r.formats as Record<string, string | undefined>);
      const pdfUrl = fmts.pdf ?? "";
      return `
    <item>
      <title><![CDATA[${title}]]></title>
      <link>${link}</link>
      <guid isPermaLink="true">${link}</guid>
      <pubDate>${pubDate}</pubDate>
      <description><![CDATA[${r.report_class} report covering ${r.window_from.split("T")[0]} to ${r.window_to.split("T")[0]}]]></description>
      ${pdfUrl ? `<enclosure url="${pdfUrl}" type="application/pdf" length="0" />` : ""}
    </item>`;
    })
    .join("\n");

  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>ABIS Public Reports</title>
    <link>${baseUrl}/</link>
    <description>Newly published ABIS transparency reports</description>
    <language>en</language>
    <atom:link href="${baseUrl}/api/rss.xml" rel="self" type="application/rss+xml" />
    ${items}
  </channel>
</rss>`;

  return new Response(xml, {
    headers: {
      "Content-Type": "application/rss+xml; charset=utf-8",
      "Cache-Control": "public, max-age=3600",
    },
  });
};
