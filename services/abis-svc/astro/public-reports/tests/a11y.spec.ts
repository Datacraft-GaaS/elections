import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test.describe("Accessibility (axe)", () => {
  test("index page has no critical axe violations", async ({ page }) => {
    await page.goto("/");
    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa"])
      .analyze();
    const critical = results.violations.filter((v) => v.impact === "critical");
    expect(critical, `Critical violations: ${JSON.stringify(critical, null, 2)}`).toHaveLength(0);
  });

  test("report detail page has no critical axe violations", async ({ page }) => {
    await page.goto("/en/reports/public-fmr-fnmr-monthly/00000000-0000-7000-8000-000000000001/");
    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa"])
      .analyze();
    const critical = results.violations.filter((v) => v.impact === "critical");
    expect(critical, `Critical violations: ${JSON.stringify(critical, null, 2)}`).toHaveLength(0);
  });
});
