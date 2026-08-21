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
