<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";

  import EventBlock from "./EventBlock.svelte";
  import { collections, events, view } from "../stores";
  import { layoutWeek, type LaidOutEvent } from "../layout";
  import { eventTimeToZoned } from "../time/parse";

  type Props = {
    onSelect?: (e: LaidOutEvent) => void;
  };
  const { onSelect }: Props = $props();

  const systemTz = Temporal.Now.timeZoneId();
  const HOURS = Array.from({ length: 24 }, (_, i) => i);
  const DAYS = Array.from({ length: 7 }, (_, i) => i);

  const nowZoned = $derived(Temporal.Now.zonedDateTimeISO(systemTz));
  const nowMinute = $derived(nowZoned.hour * 60 + nowZoned.minute);

  /** Today's index within the visible week (0..6), or -1 if today isn't in view. */
  const todayIndex = $derived.by(() => {
    const cursorDay = Math.floor(view.cursor.epochMilliseconds / 86_400_000);
    const nowDay = Math.floor(nowZoned.epochMilliseconds / 86_400_000);
    const offset = nowDay - cursorDay;
    return offset >= 0 && offset < 7 ? offset : -1;
  });

  const laidOut: LaidOutEvent[] = $derived.by(() => {
    const cursorDayMs = Math.floor(view.cursor.epochMilliseconds / 86_400_000);
    const items = events.instances.map((ei) => {
      const start = eventTimeToZoned(ei.start, systemTz);
      const end = eventTimeToZoned(ei.end, systemTz);
      const dayIndex = Math.max(
        0,
        Math.min(
          6,
          Math.floor(start.epochMilliseconds / 86_400_000) - cursorDayMs,
        ),
      );
      const startMinute = start.hour * 60 + start.minute;
      const endMinute = end.hour * 60 + end.minute;
      return { ...ei, dayIndex, startMinute, endMinute };
    });
    return layoutWeek(items);
  });

  function colorForCollection(cid: string): string {
    const found = collections.list.find((c) => (c.id as unknown as string) === cid);
    return found?.color ?? "#3b82f6";
  }

  function dayLabel(i: number): string {
    const d = view.cursor.add({ days: i });
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
          {#if d === todayIndex}
            <div class="now-line" style:top={`${(nowMinute / 60) * 48}px`} aria-hidden="true"></div>
          {/if}
          {#each laidOut.filter((e) => e.dayIndex === d) as ev (`${ev.event_uid}-${ev.startMinute}`)}
            <EventBlock
              event={ev}
              color={colorForCollection(ev.collection_id as unknown as string)}
              {onSelect}
            />
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
  .now-line {
    position: absolute;
    left: 0; right: 0;
    height: 0;
    border-top: 1px solid #ef4444;
    z-index: 1;
  }
</style>
