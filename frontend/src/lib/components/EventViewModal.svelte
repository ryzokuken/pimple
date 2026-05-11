<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";

  import { deleteEvent } from "../ipc";
  import type {
    DeleteEventRequest,
    EventInstance,
    RecurringScope,
  } from "../ipc/types";
  import { eventTimeToZoned } from "../time/parse";
  import RecurringScopeDialog from "./RecurringScopeDialog.svelte";

  type Props = {
    event: EventInstance;
    onClose: () => void;
  };
  const { event, onClose }: Props = $props();

  const systemTz = Temporal.Now.timeZoneId();
  let scopeDialog = $state(false);
  let deleting = $state(false);
  let error = $state<string | null>(null);

  const startLabel = $derived.by(() => {
    const z = eventTimeToZoned(event.start, systemTz);
    return z.toLocaleString("en", {
      weekday: "short",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  });
  const endLabel = $derived.by(() => {
    const z = eventTimeToZoned(event.end, systemTz);
    return z.toLocaleString("en", { hour: "2-digit", minute: "2-digit" });
  });

  async function performDelete(scope: RecurringScope): Promise<void> {
    if (deleting) return;
    deleting = true;
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
      deleting = false;
      scopeDialog = false;
    }
  }

  function onDeleteClick(): void {
    if (event.is_recurring) {
      scopeDialog = true;
    } else {
      void performDelete("all");
    }
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === "Escape" && !scopeDialog) onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="backdrop" role="presentation" onclick={onClose} onkeydown={() => {}}>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="event-view-title"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key !== "Escape") e.stopPropagation(); }}
  >
    <h2 id="event-view-title">{event.summary || "Untitled event"}</h2>
    <p class="time">{startLabel} – {endLabel}</p>
    {#if event.location}
      <p class="loc">{event.location}</p>
    {/if}
    {#if event.description}
      <p class="desc">{event.description}</p>
    {/if}
    {#if event.is_recurring}
      <p class="muted">This event is part of a recurring series.</p>
    {/if}

    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}

    <div class="actions">
      <button type="button" onclick={onClose}>Close</button>
      <button
        type="button"
        class="danger"
        disabled={deleting}
        onclick={onDeleteClick}
        data-testid="delete-event"
      >
        {deleting ? "Deleting…" : "Delete"}
      </button>
    </div>
  </div>
</div>

{#if scopeDialog}
  <RecurringScopeDialog
    verb="delete"
    onChoose={(scope) => performDelete(scope)}
    onCancel={() => (scopeDialog = false)}
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
  h2 { margin: 0; font-size: 1.1rem; font-weight: 500; }
  .time { margin: 0; font-size: 0.95rem; color: var(--fg); }
  .loc, .desc { margin: 0; font-size: 0.9rem; color: var(--fg); }
  .desc { white-space: pre-wrap; }
  .muted { margin: 0; color: var(--muted); font-size: 0.85rem; }
  .error { color: #dc2626; font-size: 0.85rem; margin: 0.25rem 0 0; }
  .actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
  .actions button {
    background: transparent; border: 1px solid var(--border); border-radius: 6px;
    padding: 0.4rem 0.8rem; color: var(--fg); font: inherit; cursor: pointer;
  }
  .actions button:hover { background: color-mix(in oklab, var(--fg) 6%, transparent); }
  .actions button.danger { color: #dc2626; border-color: #dc2626; }
  .actions button.danger:hover { background: color-mix(in oklab, #dc2626 12%, transparent); }
  .actions button:disabled { opacity: 0.5; cursor: default; }
</style>
