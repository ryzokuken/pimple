<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";

  import { collections } from "../stores";
  import { createEvent, deleteEvent, updateEvent } from "../ipc";
  import type {
    CollectionId,
    CreateEventRequest,
    DeleteEventRequest,
    EventInstance,
    EventTime,
    RecurringScope,
    UpdateEventRequest,
  } from "../ipc/types";
  import { eventTimeToZoned } from "../time/parse";
  import RecurringScopeDialog from "./RecurringScopeDialog.svelte";

  type Props = {
    onClose: () => void;
    /** When present, the modal opens in edit mode pre-filled from this instance. */
    event?: EventInstance | null;
  };
  const { onClose, event = null }: Props = $props();

  const mode = $derived(event === null ? "create" : "edit");

  const systemTz = Temporal.Now.timeZoneId();
  const todayIso = Temporal.Now.plainDateISO().toString();

  function eventTimeToFormParts(t: EventTime): {
    date: string;
    time: string;
    allDay: boolean;
  } {
    if (t.kind === "all_day") {
      return { date: t.date, time: "00:00", allDay: true };
    }
    const z = eventTimeToZoned(t, systemTz);
    const date = z.toPlainDate().toString();
    const time = `${String(z.hour).padStart(2, "0")}:${String(z.minute).padStart(2, "0")}`;
    return { date, time, allDay: false };
  }

  // Initial form state — pre-filled from `event` when present.
  const initStart = event
    ? eventTimeToFormParts(event.start)
    : { date: todayIso, time: "09:00", allDay: false };
  const initEnd = event
    ? eventTimeToFormParts(event.end)
    : { date: todayIso, time: "10:00", allDay: false };

  let summary = $state(event?.summary ?? "");
  let description = $state(event?.description ?? "");
  let location = $state(event?.location ?? "");
  let allDay = $state(initStart.allDay);
  let startDate = $state(initStart.date);
  let startTime = $state(initStart.time);
  let endDate = $state(initEnd.date);
  let endTime = $state(initEnd.time);
  let collectionId = $state<string>(
    event ? (event.collection_id as unknown as string) : "",
  );
  let submitting = $state(false);
  let error = $state<string | null>(null);
  let titleInput = $state<HTMLInputElement | null>(null);
  let scopeDialog = $state<null | "save" | "delete">(null);

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
    const dt = Temporal.PlainDateTime.from(`${date}T${time}:00`).toZonedDateTime(
      systemTz,
    );
    return { kind: "zoned", zoned: dt.toString() };
  }

  async function performSave(scope: RecurringScope): Promise<void> {
    if (!event) return;
    submitting = true;
    error = null;
    try {
      const req: UpdateEventRequest = {
        uid: event.event_uid,
        collection_id: event.collection_id,
        expected_raw_hash: event.raw_hash,
        scope,
        occurrence: scope === "all" ? null : event.recurrence_id,
        summary,
        description: description || null,
        location: location || null,
        start: eventTimeFor(startDate, startTime),
        end: eventTimeFor(endDate, endTime),
        rrule: null,
      };
      await updateEvent(req);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      submitting = false;
      scopeDialog = null;
    }
  }

  async function performDelete(scope: RecurringScope): Promise<void> {
    if (!event) return;
    submitting = true;
    error = null;
    try {
      const req: DeleteEventRequest = {
        uid: event.event_uid,
        collection_id: event.collection_id,
        expected_raw_hash: event.raw_hash,
        scope,
        occurrence: scope === "all" ? null : event.recurrence_id,
      };
      await deleteEvent(req);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      submitting = false;
      scopeDialog = null;
    }
  }

  async function submit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    if (submitting) return;

    if (mode === "create") {
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
      return;
    }

    // Edit mode
    if (event && event.is_recurring) {
      scopeDialog = "save";
    } else {
      await performSave("all");
    }
  }

  function onDeleteClick(): void {
    if (!event) return;
    if (event.is_recurring) {
      scopeDialog = "delete";
    } else {
      void performDelete("all");
    }
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === "Escape" && scopeDialog === null) onClose();
  }

  const title = $derived(mode === "create" ? "New event" : "Edit event");
  const submitLabel = $derived(
    mode === "create"
      ? submitting ? "Creating…" : "Create"
      : submitting ? "Saving…" : "Save",
  );
</script>

<svelte:window onkeydown={onKey} />

<div class="backdrop" role="presentation" onclick={onClose} onkeydown={() => {}}>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key !== "Escape") e.stopPropagation(); }}
  >
    <form onsubmit={submit}>
      <h2>{title}</h2>

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

      {#if mode === "create"}
        <label class="row">
          <span>Calendar</span>
          <select bind:value={collectionId} required>
            {#each collections.list as c (c.id)}
              {@const cid = c.id as unknown as string}
              <option value={cid}>{c.display_name}</option>
            {/each}
          </select>
        </label>
      {/if}

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
        {#if mode === "edit"}
          <button
            type="button"
            class="danger"
            disabled={submitting}
            onclick={onDeleteClick}
            data-testid="delete-event"
          >
            Delete
          </button>
        {/if}
        <button type="button" onclick={onClose}>Cancel</button>
        <button type="submit" disabled={submitting} class="primary" data-testid="submit-event">
          {submitLabel}
        </button>
      </div>
    </form>
  </div>
</div>

{#if scopeDialog === "save"}
  <RecurringScopeDialog
    verb="edit"
    onChoose={(scope) => performSave(scope)}
    onCancel={() => (scopeDialog = null)}
  />
{:else if scopeDialog === "delete"}
  <RecurringScopeDialog
    verb="delete"
    onChoose={(scope) => performDelete(scope)}
    onCancel={() => (scopeDialog = null)}
  />
{/if}

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
  button.danger { color: #dc2626; border-color: #dc2626; margin-right: auto; }
  button.danger:hover { background: color-mix(in oklab, #dc2626 12%, transparent); }
  button:disabled { opacity: 0.5; cursor: default; }
  .error { color: #dc2626; font-size: 0.85rem; margin: 0.25rem 0 0; }
</style>
