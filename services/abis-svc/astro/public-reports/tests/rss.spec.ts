import { test, expect } from "@playwright/test";

test.describe("RSS and Atom feeds", () => {
  test("RSS feed returns valid XML with items", async ({ request }) => {
    const resp = await request.get("/api/rss.xml");
    expect(resp.ok()).toBe(true);
    const body = await resp.text();
    expect(body).toContain("<?xml");
    expect(body).toContain("<rss");
    expect(body).toContain("<channel>");
    expect(body).toContain("<item>");
  });

  test("RSS feed has correct content-type", async ({ request }) => {
    const resp = await request.get("/api/rss.xml");
    expect(resp.headers()["content-type"]).toContain("rss+xml");
  });

  test("Atom feed returns valid XML with entries", async ({ request }) => {
    const resp = await request.get("/api/atom.xml");
    expect(resp.ok()).toBe(true);
    const body = await resp.text();
    expect(body).toContain("<?xml");
    expect(body).toContain("<feed");
    expect(body).toContain("<entry>");
  });

  test("Atom feed has tlog namespace extension", async ({ request }) => {
    const resp = await request.get("/api/atom.xml");
    const body = await resp.text();
    expect(body).toContain("eel:");
  });
});
