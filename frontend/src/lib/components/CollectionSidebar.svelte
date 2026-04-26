<script lang="ts">
  import { collections } from "../stores";

  function toggle(id: string): void {
    collections.toggle(id);
  }
</script>

<nav class="collections" aria-label="Calendars">
  <h2>Calendars</h2>
  {#if collections.loading}
    <p class="muted">Loading…</p>
  {:else if collections.list.length === 0}
    <p class="muted">No calendars found.</p>
  {:else}
    <ul>
      {#each collections.list as c (c.id)}
        {@const cid = c.id as unknown as string}
        <li>
          <label>
            <input
              type="checkbox"
              checked={collections.visible.has(cid)}
              onchange={() => toggle(cid)}
            />
            <span class="swatch" style:background={c.color}></span>
            <span class="name">{c.display_name}</span>
          </label>
        </li>
      {/each}
    </ul>
  {/if}
</nav>

<style>
  .collections { padding: 0.75rem; }
  h2 {
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    margin: 0 0 0.5rem;
  }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.25rem; }
  label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.4rem;
    border-radius: 6px;
    cursor: pointer;
  }
  label:hover { background: color-mix(in oklab, var(--fg) 6%, transparent); }
  label:focus-within { outline: 2px solid var(--accent); outline-offset: 1px; }
  .swatch { width: 0.7rem; height: 0.7rem; border-radius: 50%; flex: 0 0 auto; }
  .muted { color: var(--muted); margin: 0.25rem 0; }
</style>
