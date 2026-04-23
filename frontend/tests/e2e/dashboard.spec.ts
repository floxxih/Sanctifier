import { test, expect } from "@playwright/test";
import path from "path";

test.describe("Results Dashboard", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/dashboard");
  });

  test("should display initial empty state", async ({ page }) => {
    await expect(page.getByText("Load a report to view findings.")).toBeVisible();
    await expect(page.getByRole("button", { name: "Parse JSON" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Export PDF" })).toBeDisabled();
  });

  test("should load and parse JSON report", async ({ page }) => {
    const filePath = path.join(__dirname, "fixtures", "sample-report.json");
    
    // Upload the JSON file
    const fileChooserPromise = page.waitForEvent("filechooser");
    await page.getByText("Upload JSON").click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(filePath);

    // Click Parse JSON
    await page.getByRole("button", { name: "Parse JSON" }).click();

    // Check if findings are displayed
    await expect(page.getByText("Findings")).toHaveCount(2); // Tab and Header
    await expect(page.getByText("initialize")).toBeVisible();
    await expect(page.getByText("transfer")).toBeVisible();
    await expect(page.getByText("add_balance")).toBeVisible();

    // Check Sanctity Score and Summary Chart
    await expect(page.getByText("Sanctity Score")).toBeVisible();
    await expect(page.getByText("Findings Summary")).toBeVisible();

    // Export PDF should now be enabled
    await expect(page.getByRole("button", { name: "Export PDF" })).toBeEnabled();
  });

  test("should filter findings by severity", async ({ page }) => {
    // Load data first
    const filePath = path.join(__dirname, "fixtures", "sample-report.json");
    const fileChooserPromise = page.waitForEvent("filechooser");
    await page.getByText("Upload JSON").click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(filePath);
    await page.getByRole("button", { name: "Parse JSON" }).click();

    // Select 'Critical' filter
    await page.getByRole("button", { name: "critical" }).click();

    // Should show critical findings (initialize - AUTH_GAP is critical)
    await expect(page.getByText("initialize")).toBeVisible();
    // Should NOT show medium/high findings (if we had specific count tests we'd check them)
    
    // Select 'Low' filter
    await page.getByRole("button", { name: "low" }).click();
    // Should show no findings for this sample
    await expect(page.getByText("No findings match the selected severity.")).toBeVisible();
  });

  test("should switch between Findings and Call Graph tabs", async ({ page }) => {
    // Load data
    const filePath = path.join(__dirname, "fixtures", "sample-report.json");
    const fileChooserPromise = page.waitForEvent("filechooser");
    await page.getByText("Upload JSON").click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(filePath);
    await page.getByRole("button", { name: "Parse JSON" }).click();

    // Go to Call Graph tab
    await page.getByRole("tab", { name: "Call Graph" }).click();
    
    // Check if Call Graph is visible
    await expect(page.getByTestId("call-graph-container")).toBeVisible();
    
    // Go back to Findings tab
    await page.getByRole("tab", { name: "Findings" }).click();
    await expect(page.getByText("Filter by Severity")).toBeVisible();
  });

  test("should handle invalid JSON input", async ({ page }) => {
    const textarea = page.getByPlaceholder(/size_warnings/);
    await textarea.fill("{ invalid json }");
    await page.getByRole("button", { name: "Parse JSON" }).click();

    await expect(page.getByText(/Invalid JSON/)).toBeVisible();
    await expect(page.getByText("Load a report to view findings.")).toBeVisible();
  });
});
