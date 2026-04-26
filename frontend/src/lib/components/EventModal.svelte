<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";

  import { collections } from "../stores";
  import { createEvent } from "../ipc";
  import type { CollectionId, CreateEventRequest, EventTime } from "../ipc/types";

  type Props = { onClose: () => void };
  const { onClose }: Props = $props();

  const systemTz = Temporal.Now.timeZoneId();
  const todayIso = Temporal.Now.plainDateISO().toString();

  let summary = $state("");
  let description = $state("");
  let location = $state("");
  let allDay = $state(false);
  let startDate = $state(todayIso);
  let startTime = $state("09:00");
  let endDate = $state(todayIso);
  let endTime = $state("10:00");
  let collectionId = $state<string>("");
  let submitting = $state(false);
  let error = $state<string | null>(null);
  let titleInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (collectionId === "" && collections.list.length > 0) {
      collectionId = collections.list[0]!.id as unknown as string;
    }
  });

  $effect(() => {
    titleInput?.focus();
  });

  function eventTimeFor(date: string, time: string): EventTime {
    if (allDay) {
      return { kind: "all_day", date };
    }
    const dt = Temporal.PlainDateTime.from(`${date}T${time}:00`)
      .toZonedDateTime(systemTz);
    return { kind: "zoned", zoned: dt.toString() };
  }

  async function submit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    if (submitting) return;
    submitting = true;
    error = null;
    try {
      const req: CreateEventRequest = {
        collection_id: collectionId as unknown as CollectionId,
        summary,
        description: description || null,
        location: location || null,
        start: eventTimeFor(startDate, startTime),
        end: eventTimeFor(endDate, endTime),
        rrule: null,
      };
      await createEvent(req);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="backdrop" role="presentation" onclick={onClose}>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    aria-label="Create event"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <form onsubmit={submit}>
      <h2>New event</h2>

      <label class="row">
        <span>Title</span>
        <input bind:this={titleInput} bind:value={summary} required />
      </label>

      <label class="row checkbox">
        <input type="checkbox" bind:checked={allDay} />
        <span>All day</span>
      </label>

      <div class="row pair">
        <label>
          <span>Start</span>
          <input type="date" bind:value={startDate} required />
          {#if !allDay}
            <input type="time" bind:value={startTime} required />
          {/if}
        </label>
        <label>
          <span>End</span>
          <input type="date" bind:value={endDate} required />
          {#if !allDay}
            <input type="time" bind:value={endTime} required />
          {/if}
        </label>
      </div>

      <label class="row">
        <span>Calendar</span>
        <select bind:value={collectionId} required>
          {#each collections.list as c (c.id)}
            {@const cid = c.id as unknown as string}
            <option value={cid}>{c.display_name}</option>
          {/each}
        </select>
      </label>

      <label class="row">
        <span>Location</span>
        <input bind:value={location} />
      </label>

      <label class="row">
        <span>Description</span>
        <textarea bind:value={description} rows="3"></textarea>
      </label>

      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}

      <div class="actions">
        <button type="button" onclick={onClose}>Cancel</button>
        <button type="submit" disabled={submitting} class="primary">
          {submitting ? "Creating…" : "Create"}
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: grid; place-items: center;
    z-index: 100;
  }
  .modal {
    background: var(--bg); color: var(--fg);
    border: 1px solid var(--border); border-radius: 8px;
    padding: 1rem 1.25rem;
    width: min(440px, 92vw);
    display: flex; flex-direction: column; gap: 0.6rem;
  }
  form { display: contents; }
  h2 { margin: 0 0 0.5rem; font-size: 1.05rem; font-weight: 500; }
  .row { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: var(--muted); }
  .row > input, .row > select, .row > textarea { font: inherit; padding: 0.4rem 0.5rem; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--fg); }
  .row.checkbox { flex-direction: row; align-items: center; gap: 0.5rem; }
  .pair { flex-direction: row; gap: 0.5rem; }
  .pair > label { flex: 1; display: flex; flex-direction: column; gap: 0.25rem; }
  .actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
  button { background: transparent; border: 1px solid var(--border); border-radius: 6px; padding: 0.4rem 0.8rem; color: var(--fg); font: inherit; cursor: pointer; }
  button.primary { background: var(--accent); color: white; border-color: var(--accent); }
  button:disabled { opacity: 0.5; cursor: default; }
  .error { color: #dc2626; font-size: 0.85rem; margin: 0.25rem 0 0; }
</style>
