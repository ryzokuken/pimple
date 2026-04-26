import { Temporal } from "@js-temporal/polyfill";

import type { EventTime } from "../ipc/types";

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
