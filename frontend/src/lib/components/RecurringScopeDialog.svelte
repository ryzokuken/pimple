<script lang="ts">
  import type { RecurringScope } from "../ipc/types";

  type Props = {
    /** "delete" or "edit" — drives the prompt copy. */
    verb: "delete" | "edit";
    onChoose: (scope: RecurringScope) => void;
    onCancel: () => void;
  };
  const { verb, onChoose, onCancel }: Props = $props();

  const headline = $derived(verb === "delete" ? "Delete recurring event" : "Edit recurring event");
  const body = $derived(
    verb === "delete"
      ? "This is a recurring event. Which occurrences would you like to delete?"
      : "This is a recurring event. Which occurrences would you like to edit?",
  );

  function onKey(e: KeyboardEvent): void {
    if (e.key === "Escape") onCancel();
  }
</script>

<svelte:window onkeydown={onKey} />

<div
  class="backdrop"
  role="presentation"
  onclick={onCancel}
  onkeydown={() => {}}
>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="scope-dialog-title"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key !== "Escape") e.stopPropagation(); }}
  >
    <h2 id="scope-dialog-title">{headline}</h2>
    <p>{body}</p>
    <div class="choices">
      <button
        type="button"
        onclick={() => onChoose("this_instance")}
        data-testid="scope-this-instance"
      >
        Only this instance
      </button>
      <button
        type="button"
        onclick={() => onChoose("this_and_future")}
        data-testid="scope-this-and-future"
      >
        This and all future occurrences
      </button>
      <button
        type="button"
        onclick={() => onChoose("all")}
        data-testid="scope-all"
      >
        All occurrences in the series
      </button>
    </div>
    <div class="actions">
      <button type="button" onclick={onCancel}>Cancel</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: grid; place-items: center;
    z-index: 110;
  }
  .modal {
    background: var(--bg); color: var(--fg);
    border: 1px solid var(--border); border-radius: 8px;
    padding: 1rem 1.25rem;
    width: min(420px, 92vw);
    display: flex; flex-direction: column; gap: 0.6rem;
  }
  h2 { margin: 0; font-size: 1.05rem; font-weight: 500; }
  p { margin: 0; color: var(--muted); font-size: 0.9rem; }
  .choices {
    display: flex; flex-direction: column; gap: 0.35rem;
  }
  .choices button {
    text-align: left;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.5rem 0.7rem;
    color: var(--fg);
    font: inherit;
    cursor: pointer;
  }
  .choices button:hover { background: color-mix(in oklab, var(--fg) 6%, transparent); }
  .choices button:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  .actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.25rem; }
  .actions button {
    background: transparent; border: 1px solid var(--border); border-radius: 6px;
    padding: 0.4rem 0.8rem; color: var(--fg); font: inherit; cursor: pointer;
  }
</style>
