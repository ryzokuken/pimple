<script lang="ts">
  import { config } from "../stores";
</script>

<div class="first-run" role="dialog" aria-modal="true" aria-labelledby="first-run-title">
  <div class="card">
    <h1 id="first-run-title">Welcome to pimple</h1>
    <p>
      Pimple reads your calendar from a folder of <code>.ics</code> files in
      <a href="https://vdirsyncer.pimutils.org/en/stable/vdir.html">vdir</a>
      format. Choose the root directory of your vdir to get started — for
      example,
      <code>~/.calendars</code>.
    </p>
    <button
      type="button"
      class="primary"
      onclick={() => config.pickAndApplyVdirRoot()}
      disabled={config.pickerOpen}
      data-testid="pick-vdir-root"
    >
      {config.pickerOpen ? "Opening picker…" : "Choose directory"}
    </button>
    {#if config.error}
      <p class="error" role="alert">{config.error}</p>
    {/if}
    <p class="muted">
      You can change this later by editing
      <code>$XDG_CONFIG_HOME/pimple/config.toml</code>.
    </p>
  </div>
</div>

<style>
  .first-run {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    background: var(--bg);
    z-index: 200;
  }
  .card {
    max-width: 420px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  h1 {
    font-size: 1.2rem;
    margin: 0;
  }
  p {
    margin: 0;
    color: var(--fg);
    line-height: 1.5;
  }
  .muted {
    color: var(--muted);
    font-size: 0.85rem;
  }
  .error {
    color: #dc2626;
    font-size: 0.9rem;
  }
  button {
    align-self: flex-start;
    background: var(--accent);
    color: white;
    border: 1px solid var(--accent);
    border-radius: 6px;
    padding: 0.5rem 0.9rem;
    font: inherit;
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.92em;
    background: color-mix(in oklab, var(--fg) 8%, transparent);
    padding: 0 0.2em;
    border-radius: 3px;
  }
</style>
