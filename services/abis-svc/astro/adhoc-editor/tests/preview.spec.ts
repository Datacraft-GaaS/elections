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
