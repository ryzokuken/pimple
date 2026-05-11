<script lang="ts">
  import type { EventInstance } from "../ipc/types";

  type Props = {
    event: EventInstance;
    color: string;
    onSelect?: (e: EventInstance) => void;
  };
  const { event, color, onSelect }: Props = $props();
</script>

<button
  type="button"
  class="chip"
  style:background={color}
  onclick={(e) => {
    e.stopPropagation();
    onSelect?.(event);
  }}
  aria-label={event.summary || "Untitled event"}
>
  <span class="title">{event.summary || "Untitled"}</span>
</button>

<style>
  .chip {
    display: block;
    width: 100%;
    border: none;
    border-radius: 3px;
    padding: 1px 4px;
    color: white;
    font-size: 0.72rem;
    text-align: left;
    cursor: pointer;
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.12);
  }
  .chip:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  .title { display: block; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
</style>
