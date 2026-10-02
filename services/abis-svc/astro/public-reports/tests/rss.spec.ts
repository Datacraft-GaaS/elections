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
