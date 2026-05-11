<script lang="ts">
  import type { Temporal } from "@js-temporal/polyfill";

  import { collections, events, view } from "../stores";
  import { rangeIso } from "../time/parse";

  $effect(() => {
    const r = rangeIso(view.cursor, 7);
    void events.setView(r.start, r.end, collections.visibleIds());
  });

  const label = $derived.by(() => {
    const end = view.cursor.add({ days: 6 });
    const fmt = (d: Temporal.ZonedDateTime) =>
      d.toLocaleString("en", { month: "short", day: "numeric" });
    return `${fmt(view.cursor)} – ${fmt(end)}, ${view.cursor.year}`;
  });
</script>

<header class="navigator" aria-label="Week navigator">
  <div class="left">
    <button type="button" onclick={() => view.shift(-7)} aria-label="Previous week">‹</button>
    <button type="button" onclick={() => view.goToday()}>Today</button>
    <button type="button" onclick={() => view.shift(7)} aria-label="Next week">›</button>
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
