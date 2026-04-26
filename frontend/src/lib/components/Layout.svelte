<script lang="ts">
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";

  import { collections, events } from "../stores";

  type Props = {
    sidebar: Snippet;
    main: Snippet;
    nav: Snippet;
  };
  const { sidebar, main, nav }: Props = $props();

  onMount(async () => {
    await events.init();
    await collections.load();
    return () => events.destroy();
  });
</script>

<div class="app">
  {@render nav()}
  <div class="body">
    <aside class="sidebar">{@render sidebar()}</aside>
    <section class="main">{@render main()}</section>
  </div>
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100%; }
  .body { display: grid; grid-template-columns: 220px 1fr; flex: 1; min-height: 0; }
  .sidebar { border-right: 1px solid var(--border); overflow: auto; }
  .main { overflow: auto; }
</style>
