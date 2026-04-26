import { expect, test } from "@playwright/test";

test("loads the app and shows the week grid", async ({ page }) => {
  await page.goto("http://localhost:1420");
  await expect(page.getByRole("grid", { name: /Week schedule/i })).toBeVisible();
  await expect(page.getByRole("columnheader")).toHaveCount(7);
});

test("opens the create-event modal and dismisses on Escape", async ({ page }) => {
  await page.goto("http://localhost:1420");
  await page.getByRole("button", { name: /\+ New/ }).click();
  const modal = page.getByRole("dialog", { name: /Create event/i });
  await expect(modal).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(modal).toBeHidden();
});
