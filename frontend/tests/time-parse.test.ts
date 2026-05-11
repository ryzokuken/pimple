import { Temporal } from "@js-temporal/polyfill";
import { describe, expect, test } from "vitest";

import {
  eventTimeToZoned,
  monthGridFor,
  rangeIso,
  startOfWeek,
} from "../src/lib/time/parse";

describe("eventTimeToZoned", () => {
  test("all-day", () => {
    const z = eventTimeToZoned(
      { kind: "all_day", date: "2026-04-25" },
      "Europe/Berlin",
    );
    expect(z.year).toBe(2026);
    expect(z.month).toBe(4);
    expect(z.day).toBe(25);
  });

  test("utc", () => {
    const z = eventTimeToZoned(
      { kind: "utc", instant: "2026-04-25T15:30:00Z" },
      "Europe/Berlin",
    );
    expect(z.timeZoneId).toBe("Europe/Berlin");
    // 15:30 UTC == 17:30 Berlin (DST in April)
    expect(z.hour).toBe(17);
  });

  test("zoned converts to display tz", () => {
    const z = eventTimeToZoned(
      {
        kind: "zoned",
        zoned: "2026-04-25T10:30:00-04:00[America/New_York]",
      },
      "UTC",
    );
    expect(z.timeZoneId).toBe("UTC");
    expect(z.hour).toBe(14);
    expect(z.minute).toBe(30);
  });
});

describe("startOfWeek", () => {
  test("Monday-start, Wednesday input", () => {
    const wed = Temporal.ZonedDateTime.from(
      "2026-04-22T10:00:00[Europe/Berlin]",
    );
    const sow = startOfWeek(wed, "monday");
    expect(sow.day).toBe(20); // Mon Apr 20
    expect(sow.hour).toBe(0);
  });

  test("Sunday-start, Wednesday input lands on prior Sunday", () => {
    const wed = Temporal.ZonedDateTime.from(
      "2026-04-22T10:00:00[Europe/Berlin]",
    );
    const sow = startOfWeek(wed, "sunday");
    expect(sow.day).toBe(19); // Sun Apr 19
    expect(sow.dayOfWeek).toBe(7); // ISO Sunday is 7
    expect(sow.hour).toBe(0);
  });

  test("Sunday-start when input is already Sunday is idempotent", () => {
    const sun = Temporal.ZonedDateTime.from(
      "2026-04-19T15:30:00[Europe/Berlin]",
    );
    const sow = startOfWeek(sun, "sunday");
    expect(sow.day).toBe(19);
    expect(sow.hour).toBe(0);
  });

  test("Monday-start when input is already Monday is idempotent", () => {
    const mon = Temporal.ZonedDateTime.from(
      "2026-04-20T15:30:00[Europe/Berlin]",
    );
    const sow = startOfWeek(mon, "monday");
    expect(sow.day).toBe(20);
    expect(sow.hour).toBe(0);
  });

  test("Sunday-start when input is Saturday lands on prior Sunday", () => {
    const sat = Temporal.ZonedDateTime.from(
      "2026-04-25T10:00:00[Europe/Berlin]",
    );
    const sow = startOfWeek(sat, "sunday");
    expect(sow.day).toBe(19); // 6 days back
  });
});

describe("monthGridFor", () => {
  test("April 2026 (5-week month, Wed-first, Monday-start) yields 35 days from Mon Mar 30", () => {
    const apr = Temporal.ZonedDateTime.from("2026-04-15T12:00:00[Europe/Berlin]");
    const grid = monthGridFor(apr, "monday");
    expect(grid.days).toBe(35);
    expect(grid.start.month).toBe(3);
    expect(grid.start.day).toBe(30);
    expect(grid.start.dayOfWeek).toBe(1);
  });

  test("January 2023 (6-week month, Sunday-first, Monday-start) yields 42 days from Mon Dec 26", () => {
    const jan = Temporal.ZonedDateTime.from("2023-01-15T12:00:00[Europe/Berlin]");
    const grid = monthGridFor(jan, "monday");
    expect(grid.days).toBe(42);
    expect(grid.start.year).toBe(2022);
    expect(grid.start.month).toBe(12);
    expect(grid.start.day).toBe(26);
  });

  test("April 2026 with Sunday-start yields a Sunday gridStart", () => {
    const apr = Temporal.ZonedDateTime.from("2026-04-15T12:00:00[Europe/Berlin]");
    const grid = monthGridFor(apr, "sunday");
    expect(grid.start.dayOfWeek).toBe(7);
    // Apr 1 is Wed; the prior Sunday is Mar 29.
    expect(grid.start.month).toBe(3);
    expect(grid.start.day).toBe(29);
  });

  test("gridStart is at midnight in the input's zone", () => {
    const z = Temporal.ZonedDateTime.from("2026-04-15T18:30:45[Europe/Berlin]");
    const grid = monthGridFor(z, "monday");
    expect(grid.start.hour).toBe(0);
    expect(grid.start.minute).toBe(0);
    expect(grid.start.second).toBe(0);
    expect(grid.start.timeZoneId).toBe("Europe/Berlin");
  });
});

describe("rangeIso", () => {
  test("seven-day range serializes start/end", () => {
    const start = Temporal.ZonedDateTime.from(
      "2026-04-20T00:00:00[Europe/Berlin]",
    );
    const r = rangeIso(start, 7);
    expect(r.start).toContain("2026-04");
    // Berlin midnight is UTC+2 in April, so end is 2026-04-26T22:00:00Z (= Apr 27 00:00 Berlin)
    expect(r.end).toContain("2026-04-26");
  });
});
