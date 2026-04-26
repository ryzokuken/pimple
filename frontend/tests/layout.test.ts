import { describe, expect, test } from "vitest";

import { layoutWeek } from "../src/lib/layout";

function ev(
  uid: string,
  day: number,
  start: number,
  end: number,
): Parameters<typeof layoutWeek>[0][number] {
  return {
    event_uid: uid,
    collection_id: "personal" as unknown as never,
    summary: uid,
    description: null,
    location: null,
    start: { kind: "utc", instant: "2026-04-25T15:30:00Z" },
    end: { kind: "utc", instant: "2026-04-25T16:30:00Z" },
    is_override: false,
    dayIndex: day,
    startMinute: start,
    endMinute: end,
  };
}

describe("layoutWeek", () => {
  test("non-overlapping events get column 0 / 1 col", () => {
    const out = layoutWeek([ev("a", 0, 60, 120), ev("b", 0, 180, 240)]);
    expect(out.every((e) => e.column === 0 && e.columns === 1)).toBe(true);
  });

  test("two overlapping events split into 2 columns", () => {
    const out = layoutWeek([ev("a", 0, 60, 180), ev("b", 0, 90, 210)]);
    expect(out.map((e) => e.columns)).toEqual([2, 2]);
    expect(out.map((e) => e.column).sort()).toEqual([0, 1]);
  });

  test("three-way overlap → 3 columns", () => {
    const out = layoutWeek([
      ev("a", 0, 60, 240),
      ev("b", 0, 90, 180),
      ev("c", 0, 120, 200),
    ]);
    expect(out.every((e) => e.columns === 3)).toBe(true);
  });

  test("events on different days don't share clusters", () => {
    const out = layoutWeek([ev("a", 0, 60, 180), ev("b", 1, 60, 180)]);
    expect(out.every((e) => e.columns === 1)).toBe(true);
  });
});
