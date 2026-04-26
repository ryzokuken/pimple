<script lang="ts">
  import CollectionSidebar from "./lib/components/CollectionSidebar.svelte";
  import Layout from "./lib/components/Layout.svelte";
  import Navigator from "./lib/components/Navigator.svelte";
  import WeekGrid from "./lib/components/WeekGrid.svelte";
</script>

<Layout>
  {#snippet nav()}<Navigator />{/snippet}
  {#snippet sidebar()}<CollectionSidebar />{/snippet}
  {#snippet main()}<WeekGrid />{/snippet}
</Layout>
