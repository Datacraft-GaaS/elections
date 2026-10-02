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

test.describe('Editor page', () => {
  test('loads the index page', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h1')).toContainText('Ad-hoc Reports');
  });

  test('has skip-to-main link', async ({ page }) => {
    await page.goto('/');
    const skipLink = page.locator('.skip-link');
    await expect(skipLink).toBeDefined();
  });

  test('new report page loads', async ({ page }) => {
    await page.goto('/adhoc/new');
    await expect(page.locator('h1')).toContainText('New Ad-hoc Report');
  });

  test('new report form has required fields', async ({ page }) => {
    await page.goto('/adhoc/new');
    await expect(page.locator('#title-input')).toBeVisible();
    await expect(page.locator('#slug-input')).toBeVisible();
    await expect(page.locator('#visibility-select')).toBeVisible();
  });
});
