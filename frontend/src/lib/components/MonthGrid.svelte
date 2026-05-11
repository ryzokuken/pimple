<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";

  import EventChip from "./EventChip.svelte";
  import { collections, events, view } from "../stores";
  import type { EventInstance } from "../ipc/types";
  import { eventTimeToZoned, monthGridFor } from "../time/parse";

  type Props = {
    onSelect?: (e: EventInstance) => void;
  };
  const { onSelect }: Props = $props();

  const systemTz = Temporal.Now.timeZoneId();

  /** All days in the visible month grid. */
  const days = $derived.by(() => {
    const grid = monthGridFor(view.cursor, view.weekStart);
    return Array.from({ length: grid.days }, (_, i) => grid.start.add({ days: i }));
  });

  /** Number of weeks shown (5 or 6). */
  const weekCount = $derived(days.length / 7);

  /** Month being viewed — cells outside this month get dimmed. */
  const focusMonth = $derived(view.cursor.month);

  /** Today, in the system tz, as an epoch-day integer for cheap equality. */
  const todayEpochDay = $derived(
    Math.floor(
      Temporal.Now.zonedDateTimeISO(systemTz).epochMilliseconds / 86_400_000,
    ),
  );

  /** Bucket event instances by their start's epoch-day. */
  const bucketByDay = $derived.by(() => {
    const map: Map<number, EventInstance[]> = new Map();
    for (const ei of events.instances) {
      const start = eventTimeToZoned(ei.start, systemTz);
      const day = Math.floor(start.epochMilliseconds / 86_400_000);
      const list = map.get(day) ?? [];
      list.push(ei);
      map.set(day, list);
    }
    // Sort each bucket by start minute so chips appear chronologically.
    for (const list of map.values()) {
      list.sort((a, b) => {
        const ams = eventTimeToZoned(a.start, systemTz).epochMilliseconds;
        const bms = eventTimeToZoned(b.start, systemTz).epochMilliseconds;
        return ams - bms;
      });
    }
    return map;
  });

  function chipsFor(day: Temporal.ZonedDateTime): {
    visible: EventInstance[];
    overflow: number;
  } {
    const key = Math.floor(day.epochMilliseconds / 86_400_000);
    const all = bucketByDay.get(key) ?? [];
    const cap = 3;
    return {
      visible: all.slice(0, cap),
      overflow: Math.max(0, all.length - cap),
    };
  }

  function colorForCollection(cid: string): string {
    const found = collections.list.find((c) => (c.id as unknown as string) === cid);
    return found?.color ?? "#3b82f6";
  }

  function weekdayLabel(i: number): string {
    return days[i]!.toLocaleString("en", { weekday: "short" });
  }
</script>

<div class="month" role="grid" aria-label="Month schedule" style:--week-count={weekCount}>
  <div class="header" role="row">
    {#each Array.from({ length: 7 }, (_, i) => i) as i (i)}
      <div class="day-head" role="columnheader">{weekdayLabel(i)}</div>
    {/each}
  </div>
  <div class="body">
    {#each days as day (day.epochMilliseconds)}
      {@const inMonth = day.month === focusMonth}
      {@const isToday = Math.floor(day.epochMilliseconds / 86_400_000) === todayEpochDay}
      {@const { visible, overflow } = chipsFor(day)}
      <div
        class="cell"
        class:out-of-month={!inMonth}
        class:today={isToday}
        role="gridcell"
      >
        <div class="num">{day.day}</div>
        <div class="chips">
          {#each visible as ev (`${ev.event_uid}-${eventTimeToZoned(ev.start, systemTz).epochMilliseconds}`)}
            <EventChip
              event={ev}
              color={colorForCollection(ev.collection_id as unknown as string)}
              {onSelect}
            />
          {/each}
          {#if overflow > 0}
            <div class="more" aria-label={`${overflow} more events`}>+ {overflow} more</div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .month { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .header {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    border-bottom: 1px solid var(--border);
  }
  .day-head {
    padding: 0.4rem 0.5rem;
    font-size: 0.8rem;
    color: var(--muted);
    border-right: 1px solid var(--border);
  }
  .day-head:last-child { border-right: none; }
  .body {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    grid-auto-rows: 1fr;
    min-height: 0;
    overflow: auto;
  }
  .cell {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.25rem;
    border-right: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    min-height: 90px;
  }
  .cell:nth-child(7n) { border-right: none; }
  .cell.out-of-month { background: color-mix(in oklab, var(--bg) 92%, transparent); color: var(--muted); }
  .cell.today .num { color: var(--accent); font-weight: 600; }
  .num { font-size: 0.85rem; }
  .chips { display: flex; flex-direction: column; gap: 0.1rem; }
  .more { font-size: 0.72rem; color: var(--muted); padding: 0 0.25rem; }
</style>
