<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";

  import { collections, events } from "../stores";
  import { rangeIso, startOfWeek } from "../time/parse";

  type Props = { weekStart?: "monday" | "sunday" };
  const { weekStart = "monday" }: Props = $props();

  const systemTz = Temporal.Now.timeZoneId();

  let cursor = $state<Temporal.ZonedDateTime>(
    startOfWeek(Temporal.Now.zonedDateTimeISO(systemTz), weekStart),
  );

  $effect(() => {
    const r = rangeIso(cursor, 7);
    void events.setView(r.start, r.end, collections.visibleIds());
  });

  function shift(deltaDays: number): void {
    cursor = cursor.add({ days: deltaDays });
  }

  function goToday(): void {
    cursor = startOfWeek(Temporal.Now.zonedDateTimeISO(systemTz), weekStart);
  }

  const label = $derived.by(() => {
    const end = cursor.add({ days: 6 });
    const fmt = (d: Temporal.ZonedDateTime) =>
      `${d.toLocaleString("en", { month: "short", day: "numeric" })}`;
    return `${fmt(cursor)} – ${fmt(end)}, ${cursor.year}`;
  });
</script>

<header class="navigator" aria-label="Week navigator">
  <div class="left">
    <button type="button" onclick={() => shift(-7)} aria-label="Previous week">‹</button>
    <button type="button" onclick={goToday}>Today</button>
    <button type="button" onclick={() => shift(7)} aria-label="Next week">›</button>
  </div>
  <h1 class="range">{label}</h1>
</header>

<style>
  .navigator {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--border);
  }
  .left {
    display: flex;
    gap: 0.25rem;
  }
  button {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.35rem 0.7rem;
    color: var(--fg);
    font: inherit;
    cursor: pointer;
  }
  button:hover { background: color-mix(in oklab, var(--fg) 6%, transparent); }
  button:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  .range {
    font-size: 1rem;
    font-weight: 500;
    margin: 0;
  }
</style>
