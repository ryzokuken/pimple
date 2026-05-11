<script lang="ts">
  import { onMount } from "svelte";

  import CollectionSidebar from "./lib/components/CollectionSidebar.svelte";
  import EventModal from "./lib/components/EventModal.svelte";
  import FirstRunPicker from "./lib/components/FirstRunPicker.svelte";
  import Layout from "./lib/components/Layout.svelte";
  import MonthGrid from "./lib/components/MonthGrid.svelte";
  import Navigator from "./lib/components/Navigator.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import WeekGrid from "./lib/components/WeekGrid.svelte";
  import type { EventInstance } from "./lib/ipc/types";
  import { config, view } from "./lib/stores";

  let createModalOpen = $state(false);
  let editingInstance = $state<EventInstance | null>(null);

  onMount(async () => {
    await config.load();
    view.setWeekStart(config.weekStart);
  });
</script>

{#if config.loaded && config.vdirRoot === null}
  <FirstRunPicker />
{:else if config.loaded}
  <Layout>
    {#snippet nav()}
      <div class="nav-wrap">
        <Navigator />
        <button type="button" class="new" onclick={() => (createModalOpen = true)}>+ New</button>
      </div>
    {/snippet}
    {#snippet sidebar()}<CollectionSidebar />{/snippet}
    {#snippet main()}
      {#if view.mode === "week"}
        <WeekGrid onSelect={(e) => (editingInstance = e)} />
      {:else}
        <MonthGrid onSelect={(e) => (editingInstance = e)} />
      {/if}
    {/snippet}
  </Layout>

  {#if createModalOpen}
    <EventModal onClose={() => (createModalOpen = false)} />
  {/if}
  {#if editingInstance}
    <EventModal
      event={editingInstance}
      onClose={() => (editingInstance = null)}
    />
  {/if}
{/if}

<Toasts />

<style>
  .nav-wrap { display: flex; align-items: center; }
  .new { margin-left: auto; margin-right: 1rem; padding: 0.35rem 0.7rem; border: 1px solid var(--border); border-radius: 6px; background: transparent; color: var(--fg); cursor: pointer; }
  .new:hover { background: color-mix(in oklab, var(--fg) 6%, transparent); }
  .new:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
</style>
