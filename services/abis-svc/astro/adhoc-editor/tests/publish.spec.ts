import { test, expect } from '@playwright/test';

test.describe('Publish flow', () => {
  test('editor page has save button', async ({ page }) => {
    await page.goto('/adhoc/editor?id=test-id');
    await expect(page.locator('[data-testid="save-button"]')).toBeVisible();
  });

  test('save button is labelled correctly', async ({ page }) => {
    await page.goto('/adhoc/editor?id=test-id');
    const btn = page.locator('[data-testid="save-button"]');
    await expect(btn).toContainText('Save version');
  });
});
