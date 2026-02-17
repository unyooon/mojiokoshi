import { test, expect } from "@playwright/test";
import { mockTauriBackend } from "./helpers";

test.describe("App without Tauri backend", () => {
  test("shows MojiOkoshi title on startup", async ({ page }) => {
    await page.goto("/");
    const heading = page.getByRole("heading", { name: "MojiOkoshi" });
    await expect(heading).toBeVisible();
  });

  test("shows error state when Tauri backend is unavailable", async ({ page }) => {
    await page.goto("/");
    // Without __TAURI_INTERNALS__, invoke throws immediately and the
    // catch block in App.tsx sets the status to "Error: ..."
    await expect(page.getByText(/^Error:/)).toBeVisible();
  });
});

test.describe("App with mocked Tauri backend", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriBackend(page);
  });

  test("renders MeetingControls with Start Recording button", async ({ page }) => {
    await page.goto("/");
    const startButton = page.getByRole("button", { name: "Start Recording" });
    await expect(startButton).toBeVisible();
  });

  test("renders InsightsPanel with AI Insights heading and tabs", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("button", { name: "Start Recording" })).toBeVisible();

    const heading = page.getByRole("heading", { name: "AI Insights" });
    await expect(heading).toBeVisible();

    // Verify key tab labels from InsightsPanel sections
    await expect(page.getByRole("button", { name: "サマリー" })).toBeVisible();
    await expect(page.getByRole("button", { name: "キーワード" })).toBeVisible();
    await expect(page.getByRole("button", { name: "アクション" })).toBeVisible();
    await expect(page.getByRole("button", { name: "決定事項" })).toBeVisible();
  });

  test("does not apply dark class to html element by default", async ({ page }) => {
    // With "system" theme the dark class depends on OS preference.
    // Playwright uses a Chromium instance that defaults to light mode,
    // so the <html> element should NOT have the "dark" class.
    await page.goto("/");
    await expect(page.getByRole("button", { name: "Start Recording" })).toBeVisible();

    const htmlEl = page.locator("html");
    await expect(htmlEl).not.toHaveClass(/dark/);
  });

  test("Cmd+, keyboard shortcut opens Settings dialog", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("button", { name: "Start Recording" })).toBeVisible();

    // Press Cmd+, (Meta+Comma)
    await page.keyboard.press("Meta+,");

    // The SettingsDialog renders a <dialog> with an <h3> that reads "Settings"
    const settingsHeading = page.getByRole("heading", { name: "Settings" });
    await expect(settingsHeading).toBeVisible();
  });
});
