import { Temporal } from "@js-temporal/polyfill";

import { monthGridFor, startOfWeek } from "../time/parse";
import type { WeekStart } from "../ipc/types";

export type ViewMode = "week" | "month";

interface ViewStoreInit {
  /** Wall-clock provider. Production passes `Temporal.Now.zonedDateTimeISO(tz)`;
   *  tests pin a frozen value so `goToday` is deterministic. */
  now: () => Temporal.ZonedDateTime;
  weekStart: WeekStart;
  mode?: ViewMode;
}

/**
 * Owns the visible cursor, the current week-start preference, and the
 * week-vs-month view mode. Shared by Navigator (dispatches shifts),
 * WeekGrid, and MonthGrid (read cursor and mode for rendering).
 */
export class ViewStore {
  cursor = $state<Temporal.ZonedDateTime>(undefined!);
  weekStart = $state<WeekStart>("monday");
  mode = $state<ViewMode>("week");

  readonly #now: () => Temporal.ZonedDateTime;

  constructor(init: ViewStoreInit) {
    this.#now = init.now;
    this.weekStart = init.weekStart;
    this.mode = init.mode ?? "week";
    this.cursor = startOfWeek(this.#now(), init.weekStart);
  }

  /** Advance (positive) or rewind (negative) the cursor by N days. */
  shift(deltaDays: number): void {
    this.cursor = this.cursor.add({ days: deltaDays });
  }

  /** Step by one logical unit forward (+1) or back (-1) — a week in week
   *  mode, a month in month mode. */
  step(forward: boolean): void {
    if (this.mode === "week") {
      this.shift(forward ? 7 : -7);
    } else {
      this.cursor = this.cursor.add({ months: forward ? 1 : -1 });
    }
  }

  /** Snap cursor to today (week mode: current week; month mode: current month). */
  goToday(): void {
    if (this.mode === "week") {
      this.cursor = startOfWeek(this.#now(), this.weekStart);
    } else {
      this.cursor = this.#now().with({
        hour: 0,
        minute: 0,
        second: 0,
        millisecond: 0,
        microsecond: 0,
        nanosecond: 0,
      });
    }
  }

  /**
   * Switch week-start preference. Re-anchors the cursor to the equivalent
   * day in the new convention so the *visible week* stays the same — e.g.
   * flipping Monday → Sunday on a cursor of Mon Apr 20 yields Sun Apr 19,
   * not the prior Monday (Apr 13).
   */
  setWeekStart(value: WeekStart): void {
    if (this.weekStart === value) return;
    const midweek = this.cursor.add({ days: 3 });
    this.weekStart = value;
    if (this.mode === "week") {
      this.cursor = startOfWeek(midweek, value);
    }
    // In month mode the cursor anchors to a specific date within the month;
    // weekStart only affects the grid edges, not the cursor.
  }

  /** Switch view mode. Keeps the cursor's date but re-anchors it to a
   *  meaningful representative for the new mode. */
  setMode(value: ViewMode): void {
    if (this.mode === value) return;
    this.mode = value;
    if (value === "week") {
      this.cursor = startOfWeek(this.cursor, this.weekStart);
    }
    // Month mode keeps cursor as-is (any day inside the month).
  }

  /**
   * Date range to fetch events for, based on current mode. Reactive — Svelte
   * 5 method calls on classes with $state fields re-run when the underlying
   * state changes, so callers can pass `view.range()` into a $derived chain.
   */
  range(): { start: Temporal.ZonedDateTime; days: number } {
    if (this.mode === "week") {
      return { start: this.cursor, days: 7 };
    }
    return monthGridFor(this.cursor, this.weekStart);
  }
}

const SYSTEM_TZ = Temporal.Now.timeZoneId();

/** The singleton view-store instance. Wall clock is the live system time. */
export const view = new ViewStore({
  now: () => Temporal.Now.zonedDateTimeISO(SYSTEM_TZ),
  weekStart: "monday",
});
