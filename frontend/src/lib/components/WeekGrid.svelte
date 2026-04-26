<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";

  import { events } from "../stores";

  const systemTz = Temporal.Now.timeZoneId();
  const HOURS = Array.from({ length: 24 }, (_, i) => i);
  const DAYS = Array.from({ length: 7 }, (_, i) => i);

  // Compute days from the first instance's start, falling back to today.
  const weekStart = $derived.by<Temporal.ZonedDateTime>(() => {
    const first = events.instances[0];
    const ref = first
      ? Temporal.Instant.from(eventInstanceStartIso(first)).toZonedDateTimeISO(systemTz)
      : Temporal.Now.zonedDateTimeISO(systemTz);
    const offset = (ref.dayOfWeek - 1 + 7) % 7; // Monday-based; configurable later
    return ref
      .subtract({ days: offset })
      .with({ hour: 0, minute: 0, second: 0, millisecond: 0, microsecond: 0, nanosecond: 0 });
  });

  function eventInstanceStartIso(ei: import("../ipc/types").EventInstance): string {
    const t = ei.start;
    switch (t.kind) {
      case "all_day": return Temporal.PlainDate.from(t.date).toZonedDateTime(systemTz).toInstant().toString();
      case "floating": return Temporal.PlainDateTime.from(t.datetime).toZonedDateTime(systemTz).toInstant().toString();
      case "utc": return t.instant;
      case "zoned": return Temporal.ZonedDateTime.from(t.zoned).toInstant().toString();
    }
  }

  function dayLabel(i: number): string {
    const d = weekStart.add({ days: i });
    return d.toLocaleString("en", { weekday: "short", day: "numeric" });
  }
</script>

<div class="week" role="grid" aria-label="Week schedule">
  <div class="header" role="row">
    <div class="hour-gutter" aria-hidden="true"></div>
    {#each DAYS as d (d)}
      <div class="day-head" role="columnheader">{dayLabel(d)}</div>
    {/each}
  </div>
  <div class="body">
    <div class="hours" aria-hidden="true">
      {#each HOURS as h (h)}
        <div class="hour-label">{String(h).padStart(2, "0")}:00</div>
      {/each}
    </div>
    <div class="grid">
      {#each DAYS as d (d)}
        <div class="day-col" role="gridcell" data-day={d}>
          {#each HOURS as h (h)}
            <div class="hour-cell" data-hour={h}></div>
          {/each}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .week { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .header { display: grid; grid-template-columns: 60px repeat(7, 1fr); border-bottom: 1px solid var(--border); }
  .hour-gutter { border-right: 1px solid var(--border); }
  .day-head {
    padding: 0.5rem 0.25rem;
    font-size: 0.85rem;
    color: var(--muted);
    border-right: 1px solid var(--border);
  }
  .day-head:last-child { border-right: none; }
  .body { flex: 1; min-height: 0; overflow: auto; display: grid; grid-template-columns: 60px 1fr; }
  .hours { display: grid; grid-template-rows: repeat(24, 48px); }
  .hour-label {
    padding: 0 0.5rem;
    font-size: 0.75rem;
    color: var(--muted);
    border-right: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
  }
  .grid { display: grid; grid-template-columns: repeat(7, 1fr); }
  .day-col {
    display: grid;
    grid-template-rows: repeat(24, 48px);
    border-right: 1px solid var(--border);
    position: relative;
  }
  .day-col:last-child { border-right: none; }
  .hour-cell { border-bottom: 1px solid var(--border); }
</style>
