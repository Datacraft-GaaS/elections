import { test, expect } from "@playwright/test";
import AxeBuilder from "axe-playwright";

test.describe("accessibility: index page", () => {
  test("has zero axe violations (WCAG 2.2 AAA)", async ({ page }) => {
    await page.goto("/");

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();

    expect(results.violations).toHaveLength(0);
  });

  test("skip-to-main-content link is present and focusable", async ({ page }) => {
    await page.goto("/");
    await page.keyboard.press("Tab");
    const skipLink = page.getByText("Skip to main content");
    await expect(skipLink).toBeFocused();
  });

  test("all images have non-empty alt text", async ({ page }) => {
    await page.goto("/");
    const images = page.locator("img");
    const count = await images.count();
    for (let i = 0; i < count; i++) {
      const alt = await images.nth(i).getAttribute("alt");
      expect(alt, `img[${i}] missing alt`).not.toBeNull();
      expect(alt?.trim(), `img[${i}] empty alt`).not.toBe("");
    }
  });

  test("page title is set", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle(/Referendum Explainer/);
  });
});
