<script lang="ts">
  import type { LaidOutEvent } from "../layout";

  type Props = {
    event: LaidOutEvent;
    color: string;
    onSelect?: (e: LaidOutEvent) => void;
  };
  const { event, color, onSelect }: Props = $props();

  const minutesPerHour = 60;
  const startHour = event.startMinute / minutesPerHour;
  const endHour = event.endMinute / minutesPerHour;
  const top = `${startHour * 48}px`;
  const height = `${Math.max(20, (endHour - startHour) * 48)}px`;
  const left = `${(event.column / event.columns) * 100}%`;
  const width = `${(1 / event.columns) * 100}%`;
</script>

<button
  type="button"
  class="block"
  style:top
  style:height
  style:left
  style:width
  style:background={color}
  onclick={() => onSelect?.(event)}
  aria-label={`${event.summary} from ${event.startMinute / 60}:00`}
>
  <span class="title">{event.summary}</span>
</button>

<style>
  .block {
    position: absolute;
    border: none;
    border-radius: 4px;
    color: white;
    font-size: 0.78rem;
    text-align: left;
    padding: 2px 4px;
    overflow: hidden;
    cursor: pointer;
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.15);
  }
  .block:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .title { display: block; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
</style>
