<script lang="ts">
  import CollectionSidebar from "./lib/components/CollectionSidebar.svelte";
  import EventModal from "./lib/components/EventModal.svelte";
  import Layout from "./lib/components/Layout.svelte";
  import Navigator from "./lib/components/Navigator.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import WeekGrid from "./lib/components/WeekGrid.svelte";

  let modalOpen = $state(false);
</script>

<Layout>
  {#snippet nav()}
    <div class="nav-wrap">
      <Navigator />
      <button type="button" class="new" onclick={() => (modalOpen = true)}>+ New</button>
    </div>
  {/snippet}
  {#snippet sidebar()}<CollectionSidebar />{/snippet}
  {#snippet main()}<WeekGrid />{/snippet}
</Layout>

{#if modalOpen}
  <EventModal onClose={() => (modalOpen = false)} />
{/if}

<Toasts />

<style>
  .nav-wrap { display: flex; align-items: center; }
  .new { margin-left: auto; margin-right: 1rem; padding: 0.35rem 0.7rem; border: 1px solid var(--border); border-radius: 6px; background: transparent; color: var(--fg); cursor: pointer; }
  .new:hover { background: color-mix(in oklab, var(--fg) 6%, transparent); }
  .new:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
</style>
