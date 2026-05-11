import { Temporal } from "@js-temporal/polyfill";
import { describe, expect, test } from "vitest";

import { ViewStore } from "../src/lib/stores/view.svelte";

function at(iso: string): Temporal.ZonedDateTime {
  return Temporal.ZonedDateTime.from(iso);
}

describe("ViewStore initial cursor", () => {
  test("Monday-start anchors cursor to the Monday of the given week", () => {
    // Wed Apr 22 2026, Berlin
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({ now: () => now, weekStart: "monday" });

    expect(view.cursor.day).toBe(20); // Mon Apr 20
    expect(view.cursor.dayOfWeek).toBe(1);
    expect(view.cursor.hour).toBe(0);
    expect(view.weekStart).toBe("monday");
  });

  test("Sunday-start anchors cursor to the Sunday of the same week", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]"); // Wed
    const view = new ViewStore({ now: () => now, weekStart: "sunday" });

    expect(view.cursor.day).toBe(19); // Sun Apr 19
    expect(view.cursor.dayOfWeek).toBe(7);
  });
});

describe("ViewStore.shift", () => {
  test("shift(+7) advances cursor by one week", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({ now: () => now, weekStart: "monday" });

    view.shift(7);

    expect(view.cursor.day).toBe(27); // Mon Apr 27
  });

  test("shift(-7) rewinds cursor by one week", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({ now: () => now, weekStart: "monday" });

    view.shift(-7);

    expect(view.cursor.day).toBe(13); // Mon Apr 13
  });
});

describe("ViewStore.setWeekStart", () => {
  test("flipping monday -> sunday re-anchors cursor to the same week's Sunday (one day earlier)", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]"); // Wed
    const view = new ViewStore({ now: () => now, weekStart: "monday" });
    expect(view.cursor.day).toBe(20); // Mon Apr 20

    view.setWeekStart("sunday");

    expect(view.cursor.day).toBe(19); // Sun Apr 19, NOT Apr 12
    expect(view.weekStart).toBe("sunday");
  });

  test("flipping sunday -> monday re-anchors cursor forward to the next Monday", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]"); // Wed
    const view = new ViewStore({ now: () => now, weekStart: "sunday" });
    expect(view.cursor.day).toBe(19); // Sun Apr 19

    view.setWeekStart("monday");

    expect(view.cursor.day).toBe(20); // Mon Apr 20 (forward), NOT Apr 13
    expect(view.weekStart).toBe("monday");
  });

  test("flipping after shifting away from today preserves the visible window", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({ now: () => now, weekStart: "monday" });
    view.shift(14); // two weeks forward -> Mon May 4
    expect(view.cursor.day).toBe(4);
    expect(view.cursor.month).toBe(5);

    view.setWeekStart("sunday");

    // Sun May 3 of the same week
    expect(view.cursor.day).toBe(3);
    expect(view.cursor.month).toBe(5);
  });
});

describe("ViewStore.goToday", () => {
  test("resets cursor to current week", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({ now: () => now, weekStart: "monday" });
    view.shift(28); // four weeks away
    expect(view.cursor.month).toBe(5);

    view.goToday();

    expect(view.cursor.day).toBe(20); // back to Mon Apr 20
  });
});

describe("ViewStore.mode + step", () => {
  test("step(forward=true) in week mode advances by 7 days", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({ now: () => now, weekStart: "monday" });
    view.step(true);
    expect(view.cursor.day).toBe(27);
  });

  test("step(forward=true) in month mode advances by 1 month", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({
      now: () => now,
      weekStart: "monday",
      mode: "month",
    });
    // Initial cursor lands at Mon Apr 20 (startOfWeek of Apr 22).
    view.step(true);
    expect(view.cursor.month).toBe(5);
    expect(view.cursor.day).toBe(20);
  });

  test("setMode(week) re-anchors cursor to startOfWeek", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({
      now: () => now,
      weekStart: "monday",
      mode: "month",
    });
    view.cursor = at("2026-04-22T10:00:00[Europe/Berlin]"); // Wed mid-month
    view.setMode("week");
    expect(view.cursor.dayOfWeek).toBe(1);
    expect(view.cursor.day).toBe(20);
  });

  test("range() returns 7 days in week mode", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({ now: () => now, weekStart: "monday" });
    const r = view.range();
    expect(r.days).toBe(7);
    expect(r.start.day).toBe(20);
  });

  test("range() returns 35 days for April 2026 in month mode", () => {
    const now = at("2026-04-22T10:00:00[Europe/Berlin]");
    const view = new ViewStore({
      now: () => now,
      weekStart: "monday",
      mode: "month",
    });
    const r = view.range();
    expect(r.days).toBe(35);
    expect(r.start.month).toBe(3);
    expect(r.start.day).toBe(30);
  });
});
