import { Temporal } from "@js-temporal/polyfill";

import type { EventTime, WeekStart } from "../ipc/types";

/**
 * Convert an `EventTime` from the IPC boundary into a Temporal type the UI
 * can position on the week grid.
 */
export function eventTimeToZoned(
  t: EventTime,
  systemTz: string,
): Temporal.ZonedDateTime {
  switch (t.kind) {
    case "all_day":
      return Temporal.PlainDate.from(t.date).toZonedDateTime(systemTz);
    case "floating":
      return Temporal.PlainDateTime.from(t.datetime).toZonedDateTime(systemTz);
    case "utc":
      return Temporal.Instant.from(t.instant).toZonedDateTimeISO(systemTz);
    case "zoned":
      // jiff serializes Zoned as RFC 9557: e.g. 2026-04-25T10:00:00-04:00[America/New_York]
      return Temporal.ZonedDateTime.from(t.zoned).withTimeZone(systemTz);
  }
}

export function nowZoned(systemTz: string): Temporal.ZonedDateTime {
  return Temporal.Now.zonedDateTimeISO(systemTz);
}

export function startOfWeek(
  dt: Temporal.ZonedDateTime,
  weekStart: "monday" | "sunday",
): Temporal.ZonedDateTime {
  const startDow = weekStart === "monday" ? 1 : 7;
  const offset = (dt.dayOfWeek - startDow + 7) % 7;
  return dt
    .subtract({ days: offset })
    .with({ hour: 0, minute: 0, second: 0, millisecond: 0, microsecond: 0, nanosecond: 0 });
}

export function rangeIso(
  start: Temporal.ZonedDateTime,
  durationDays: number,
): { start: string; end: string } {
  return {
    start: start.toInstant().toString(),
    end: start.add({ days: durationDays }).toInstant().toString(),
  };
}

/**
 * Visible-day range for a month grid containing `anyDayInMonth`. The grid
 * starts at the most recent `weekStart` on or before the 1st of the month
 * and ends at the first `weekStart` strictly after the last of the month —
 * i.e., always a whole number of weeks (35 or 42 days depending on month
 * shape and week-start convention).
 */
export function monthGridFor(
  anyDayInMonth: Temporal.ZonedDateTime,
  weekStart: WeekStart,
): { start: Temporal.ZonedDateTime; days: number } {
  const firstOfMonth = anyDayInMonth
    .with({ day: 1 })
    .with({
      hour: 0,
      minute: 0,
      second: 0,
      millisecond: 0,
      microsecond: 0,
      nanosecond: 0,
    });
  const gridStart = startOfWeek(firstOfMonth, weekStart);
  const lastOfMonth = firstOfMonth.add({ days: firstOfMonth.daysInMonth - 1 });
  const afterLast = startOfWeek(lastOfMonth, weekStart).add({ days: 7 });
  const days = Math.round(
    (afterLast.epochMilliseconds - gridStart.epochMilliseconds) /
      86_400_000,
  );
  return { start: gridStart, days };
}
