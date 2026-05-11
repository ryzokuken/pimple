import { expect, test } from "@playwright/test";

import { recordedCalls, setupTauriMock } from "./fixtures";

const APP_URL = "http://localhost:1420";

const PERSONAL_COLLECTION = {
  id: "personal",
  path: "/mock/vdir/personal",
  display_name: "Personal",
  color: "#3b82f6",
  visible: true,
};

const STANDUP_EVENT = {
  event_uid: "standup-1",
  collection_id: "personal",
  summary: "Daily standup",
  description: null,
  location: null,
  start: { kind: "utc", instant: "2026-04-25T15:30:00Z" },
  end: { kind: "utc", instant: "2026-04-25T16:00:00Z" },
  is_override: false,
  recurrence_id: { kind: "utc", instant: "2026-04-25T15:30:00Z" },
  raw_hash: "a".repeat(64),
  is_recurring: false,
};

test.describe("first-run flow", () => {
  test("FirstRunPicker is shown when no vdir is configured", async ({ page }) => {
    await setupTauriMock(page, { vdirRoot: null });
    await page.goto(APP_URL);

    await expect(
      page.getByRole("dialog", { name: /welcome to pimple/i }),
    ).toBeVisible();
    await expect(page.getByText(/week schedule/i)).toBeHidden();
  });

  test("picking a directory invokes set_vdir_root and reveals the main UI", async ({
    page,
  }) => {
    await setupTauriMock(page, { vdirRoot: null, pickerResult: "/picked" });
    await page.goto(APP_URL);

    await page.getByTestId("pick-vdir-root").click();

    await expect(
      page.getByRole("grid", { name: /week schedule/i }),
    ).toBeVisible();

    const calls = await recordedCalls(page);
    const setVdir = calls.find((c) => c.cmd === "set_vdir_root");
    expect(setVdir).toBeDefined();
    expect((setVdir!.args as { path: string }).path).toBe("/picked");
  });

  test("user cancelling the picker leaves the picker open", async ({ page }) => {
    await setupTauriMock(page, { vdirRoot: null, pickerResult: null });
    await page.goto(APP_URL);

    await page.getByTestId("pick-vdir-root").click();

    await expect(
      page.getByRole("dialog", { name: /welcome to pimple/i }),
    ).toBeVisible();
    const calls = await recordedCalls(page);
    expect(calls.find((c) => c.cmd === "set_vdir_root")).toBeUndefined();
  });
});

test.describe("week view", () => {
  test("loads with 7 columnheaders when vdir is configured", async ({ page }) => {
    await setupTauriMock(page, { collections: [PERSONAL_COLLECTION] });
    await page.goto(APP_URL);

    await expect(
      page.getByRole("grid", { name: /week schedule/i }),
    ).toBeVisible();
    await expect(page.getByRole("columnheader")).toHaveCount(7);
  });

  test("opens the create modal via + New and dismisses on Escape", async ({
    page,
  }) => {
    await setupTauriMock(page, { collections: [PERSONAL_COLLECTION] });
    await page.goto(APP_URL);

    await page.getByRole("button", { name: /\+ New/ }).click();
    const modal = page.getByRole("dialog", { name: /new event/i });
    await expect(modal).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(modal).toBeHidden();
  });

  test("clicking an event opens the edit modal pre-filled", async ({ page }) => {
    await setupTauriMock(page, {
      collections: [PERSONAL_COLLECTION],
      events: [STANDUP_EVENT],
    });
    await page.goto(APP_URL);

    await page.getByRole("button", { name: /daily standup/i }).first().click();

    const modal = page.getByRole("dialog", { name: /edit event/i });
    await expect(modal).toBeVisible();
    await expect(modal.getByRole("textbox").first()).toHaveValue("Daily standup");
    await expect(page.getByTestId("delete-event")).toBeVisible();
  });

  test("delete on a non-recurring event sends delete_event with scope=all", async ({
    page,
  }) => {
    await setupTauriMock(page, {
      collections: [PERSONAL_COLLECTION],
      events: [STANDUP_EVENT],
    });
    await page.goto(APP_URL);

    await page.getByRole("button", { name: /daily standup/i }).first().click();
    await page.getByTestId("delete-event").click();

    const calls = await recordedCalls(page);
    const del = calls.find((c) => c.cmd === "delete_event");
    expect(del).toBeDefined();
    const req = (del!.args as { request: { scope: string; uid: string } }).request;
    expect(req.scope).toBe("all");
    expect(req.uid).toBe("standup-1");
  });

  test("delete on a recurring event opens the scope dialog", async ({ page }) => {
    await setupTauriMock(page, {
      collections: [PERSONAL_COLLECTION],
      events: [{ ...STANDUP_EVENT, is_recurring: true }],
    });
    await page.goto(APP_URL);

    await page.getByRole("button", { name: /daily standup/i }).first().click();
    await page.getByTestId("delete-event").click();

    await expect(
      page.getByRole("dialog", { name: /delete recurring event/i }),
    ).toBeVisible();
    await expect(page.getByTestId("scope-this-instance")).toBeVisible();
    await expect(page.getByTestId("scope-this-and-future")).toBeVisible();
    await expect(page.getByTestId("scope-all")).toBeVisible();
  });
});

test.describe("month view", () => {
  test("Week/Month toggle swaps the grid and changes the label", async ({
    page,
  }) => {
    await setupTauriMock(page, { collections: [PERSONAL_COLLECTION] });
    await page.goto(APP_URL);

    await expect(
      page.getByRole("grid", { name: /week schedule/i }),
    ).toBeVisible();
    await page.getByTestId("mode-month").click();

    await expect(
      page.getByRole("grid", { name: /month schedule/i }),
    ).toBeVisible();
    await expect(
      page.getByRole("grid", { name: /week schedule/i }),
    ).toBeHidden();
  });
});
