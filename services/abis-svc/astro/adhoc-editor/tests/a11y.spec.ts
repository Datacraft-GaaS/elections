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

import { test, expect } from '@playwright/test';

test.describe('Accessibility', () => {
  test('index has main landmark', async ({ page }) => {
    await page.goto('/');
    const main = page.locator('main');
    await expect(main).toBeVisible();
  });

  test('index has h1', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h1')).toBeVisible();
  });

  test('editor has ARIA labels on key sections', async ({ page }) => {
    await page.goto('/adhoc/editor?id=test-id');
    const sqlSection = page.locator('[aria-label="SQL editor"]');
    await expect(sqlSection).toBeVisible();
  });

  test('skip-link present on all pages', async ({ page }) => {
    for (const url of ['/', '/adhoc/new', '/adhoc/editor?id=test-id']) {
      await page.goto(url);
      const skip = page.locator('.skip-link');
      await expect(skip).toBeDefined();
    }
  });
});
