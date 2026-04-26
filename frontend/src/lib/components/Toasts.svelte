<script lang="ts">
  import { onMount } from "svelte";

  import { diagnostics } from "../stores";

  onMount(async () => {
    await diagnostics.init();
    return () => diagnostics.destroy();
  });
</script>

<div class="toasts" aria-live="polite" aria-atomic="false">
  {#each diagnostics.list as d (d.id)}
    <div class="toast" role="status">
      <span>{d.message}</span>
      <button type="button" aria-label="Dismiss" onclick={() => diagnostics.dismiss(d.id)}>×</button>
    </div>
  {/each}
</div>

<style>
  .toasts { position: fixed; right: 1rem; bottom: 1rem; display: flex; flex-direction: column; gap: 0.5rem; z-index: 50; }
  .toast { display: flex; align-items: center; gap: 0.5rem; background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 0.4rem 0.6rem; font-size: 0.85rem; max-width: 360px; box-shadow: 0 1px 2px rgba(0,0,0,0.1); }
  .toast button { background: transparent; border: none; color: var(--muted); cursor: pointer; font-size: 1rem; line-height: 1; }
</style>
