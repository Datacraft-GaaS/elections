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

test.describe("Verification widget", () => {
  test("shows PASS for untampered file", async ({ page }) => {
    // Use the built static site.
    await page.goto("/en/reports/public-fmr-fnmr-monthly/00000000-0000-7000-8000-000000000001/");

    // Get the declared SHA-256 for json format.
    const expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    // Create a file with empty bytes (matches SHA-256 of empty).
    const emptyFile = Buffer.from("");
    await page.setInputFiles("#verify-file", {
      name: "test.json",
      mimeType: "application/json",
      buffer: emptyFile,
    });

    await page.selectOption("#verify-format", "json");
    await page.click("#verify-btn");

    await expect(page.locator("#verify-result")).toContainText("PASS", { timeout: 5000 });
  });

  test("shows FAIL for tampered file", async ({ page }) => {
    await page.goto("/en/reports/public-fmr-fnmr-monthly/00000000-0000-7000-8000-000000000001/");

    const tamperedContent = Buffer.from("this is tampered content that does not match the declared hash");
    await page.setInputFiles("#verify-file", {
      name: "tampered.pdf",
      mimeType: "application/pdf",
      buffer: tamperedContent,
    });

    await page.selectOption("#verify-format", "pdf");
    await page.click("#verify-btn");

    await expect(page.locator("#verify-result")).toContainText("FAIL", { timeout: 5000 });
  });

  test("handles missing file gracefully", async ({ page }) => {
    await page.goto("/en/reports/public-fmr-fnmr-monthly/00000000-0000-7000-8000-000000000001/");
    await page.click("#verify-btn");
    await expect(page.locator("#verify-result")).toContainText("Please select a file");
  });
});
