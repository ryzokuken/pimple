<script lang="ts">
  import type { Temporal } from "@js-temporal/polyfill";

  import { collections, events, view } from "../stores";
  import { rangeIso } from "../time/parse";
  import type { ViewMode } from "../stores/view.svelte";

  $effect(() => {
    const r = view.range();
    const iso = rangeIso(r.start, r.days);
    void events.setView(iso.start, iso.end, collections.visibleIds());
  });

  const label = $derived.by(() => {
    if (view.mode === "week") {
      const end = view.cursor.add({ days: 6 });
      const fmt = (d: Temporal.ZonedDateTime) =>
        d.toLocaleString("en", { month: "short", day: "numeric" });
      return `${fmt(view.cursor)} – ${fmt(end)}, ${view.cursor.year}`;
    }
    return view.cursor.toLocaleString("en", { month: "long", year: "numeric" });
  });

  function setMode(m: ViewMode): void {
    view.setMode(m);
  }
</script>

<header class="navigator" aria-label="Schedule navigator">
  <div class="left">
    <button type="button" onclick={() => view.step(false)} aria-label="Previous">‹</button>
    <button type="button" onclick={() => view.goToday()}>Today</button>
    <button type="button" onclick={() => view.step(true)} aria-label="Next">›</button>
  </div>
  <h1 class="range">{label}</h1>
  <div class="mode-toggle" role="group" aria-label="View mode">
    <button
      type="button"
      class:active={view.mode === "week"}
      onclick={() => setMode("week")}
      data-testid="mode-week"
    >Week</button>
    <button
      type="button"
      class:active={view.mode === "month"}
      onclick={() => setMode("month")}
      data-testid="mode-month"
    >Month</button>
  </div>
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
  .mode-toggle { display: flex; gap: 0.25rem; margin-left: auto; }
  .mode-toggle .active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
</style>
