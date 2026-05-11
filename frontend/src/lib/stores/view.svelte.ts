import { Temporal } from "@js-temporal/polyfill";

import { startOfWeek } from "../time/parse";
import type { WeekStart } from "../ipc/types";

interface ViewStoreInit {
  /** Wall-clock provider. Production passes `Temporal.Now.zonedDateTimeISO(tz)`;
   *  tests pin a frozen value so `goToday` is deterministic. */
  now: () => Temporal.ZonedDateTime;
  weekStart: WeekStart;
}

/**
 * Owns the visible-week cursor and current week-start preference, shared
 * across Navigator (dispatches shifts) and WeekGrid (reads cursor and
 * weekStart for rendering).
 */
export class ViewStore {
  cursor = $state<Temporal.ZonedDateTime>(undefined!);
  weekStart = $state<WeekStart>("monday");

  readonly #now: () => Temporal.ZonedDateTime;

  constructor(init: ViewStoreInit) {
    this.#now = init.now;
    this.weekStart = init.weekStart;
    this.cursor = startOfWeek(this.#now(), init.weekStart);
  }

  /** Advance (positive) or rewind (negative) the cursor by N days. */
  shift(deltaDays: number): void {
    this.cursor = this.cursor.add({ days: deltaDays });
  }

  /** Snap cursor to the current week (relative to weekStart). */
  goToday(): void {
    this.cursor = startOfWeek(this.#now(), this.weekStart);
  }

  /**
   * Switch week-start preference. Re-anchors the cursor to the equivalent
   * day in the new convention so the *visible week* stays the same — e.g.
   * flipping Monday → Sunday on a cursor of Mon Apr 20 yields Sun Apr 19,
   * not the prior Monday (Apr 13).
   *
   * Implementation: the safe pivot is a mid-week day (cursor + 3) which is
   * inside the same visible window regardless of which day-of-week the
   * cursor is currently anchored to. `startOfWeek` of that mid-week point
   * then resolves cleanly under the new convention.
   */
  setWeekStart(value: WeekStart): void {
    if (this.weekStart === value) return;
    const midweek = this.cursor.add({ days: 3 });
    this.weekStart = value;
    this.cursor = startOfWeek(midweek, value);
  }
}

const SYSTEM_TZ = Temporal.Now.timeZoneId();

/** The singleton view-store instance. Wall clock is the live system time. */
export const view = new ViewStore({
  now: () => Temporal.Now.zonedDateTimeISO(SYSTEM_TZ),
  weekStart: "monday",
});
