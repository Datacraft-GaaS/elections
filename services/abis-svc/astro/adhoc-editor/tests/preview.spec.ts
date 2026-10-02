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

test.describe('Preview functionality', () => {
  test('editor page loads for a report id', async ({ page }) => {
    await page.goto('/adhoc/editor?id=test-report-id');
    await expect(page.locator('h1')).toContainText('Editor');
  });

  test('editor has SQL textarea', async ({ page }) => {
    await page.goto('/adhoc/editor?id=test-report-id');
    await expect(page.locator('[data-testid="sql-editor"]')).toBeVisible();
  });
});
