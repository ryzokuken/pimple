# pimple — design (v0.1)

**Status**: approved 2026-04-24
**Scope**: local-first calendar client for the vdir format, built as a Tauri 2 app on Rust + Svelte 5.

## 1. Background

pimple is a free software personal information management (PIM) client. The v0.1 scope is a calendar-only client: read and create events in a local vdir, kept in sync with CalDAV servers by [pimsync](https://pimsync.whynothugo.nl/) running as a separate tool. Later versions will add edit/delete, then tasks and contacts, then mobile.

Guiding values:

- **Minimalism** — no feature exists unless a real user need forces it. Reduce code we maintain and test ourselves.
- **Standards-first** — vdir, iCalendar (RFC 5545), CalDAV (RFC 4791), RFC 9557 for datetime serialization.
- **Local / privacy-conscious** — no network code in pimple itself. All sync is pimsync's job.
- **Cross-platform** — Linux, macOS, Windows desktop in v0.1. Mobile preserved as a future target; architecture decisions do not foreclose it.
- **Compiled and efficient top to bottom** — Rust on the backend, Svelte 5 compiled output on the frontend. No runtime VM overhead anywhere in the hot path.

Prior iteration: [vdird](https://github.com/ryzokuken/vdird) explored this space in Deno + TypeScript. Its fixtures and its struggles with iCalendar's DATE-TIME variants directly inform the design below.

## 2. Scope

### In scope for v0.1

- Read events from a vdir, auto-discovering collections (subdirectories).
- Single polished week view. One primary UI, deeply considered.
- Per-collection color and visibility toggle via a sidebar.
- Create new events via a form modal (title, time range or all-day, description, location, collection, RRULE).
- Live UI updates when pimsync (or any other writer) changes the vdir.
- Collection metadata honored: `displayname` and `color` files in the vdir.
- All iCalendar datetime forms modeled explicitly: DATE (all-day), DATE-TIME floating, DATE-TIME UTC, DATE-TIME zoned.
- VTIMEZONE-aware display: events render in the user's system time zone, computed from the event's source zone.
- RRULE expansion on read. Events with recurrences appear at the correct times in the week view.

### Deferred to v0.2

- Edit events; delete events.
- Recurring-event edit semantics (this instance / this and future / all).
- Month view.
- First-run vdir-root picker.

### Deferred to v0.3+

- VTODO (tasks), vCard (contacts).
- Natural-language quick event entry.
- Reminders and desktop notifications.
- Mobile platforms via Tauri 2.

### Explicit non-goals

- No CalDAV sync logic inside pimple. pimsync handles all network I/O.
- No writing to vdir collection metadata (`color`, `displayname`) — those belong to the user or to pimsync.
- No invocation or supervision of pimsync by pimple. pimple is a pure vdir consumer.
- No natural-language event parsing in v0.1.

## 3. Architecture

Cargo workspace with two Rust members plus a separate frontend:

```
pimple/
├── Cargo.toml              # workspace
├── crates/
│   ├── pimple-core/        # library
│   └── pimple-tauri/       # binary
└── frontend/               # Svelte 5 + TS + Vite
```

### Layer responsibilities

**`pimple-core`** — pure Rust library with no Tauri dependency. Owns all vdir layout knowledge, iCalendar parsing and serialization, the in-memory event index, the filesystem watcher, and the atomic write path. Exposes a small async API and a broadcast channel of index-change events. Reusable by a future CLI, alternate shell, or integration harness.

**`pimple-tauri`** — thin binary crate. Wires `pimple-core` to a Tauri window. A handful of `#[tauri::command]` functions; a task that forwards core broadcast events as Tauri events to the frontend.

**`frontend/`** — Svelte 5 app. Owns all UI: week grid, sidebar, modal, settings. Talks to the backend through a single typed IPC module. Reactive stores subscribed to Tauri events keep the UI in sync with the index.

### Data flow (push-based)

```
filesystem change
  → notify watcher
  → pimple-core re-parses file
  → index update
  → broadcast::Sender<IndexChange>
  → pimple-tauri forwarder task
  → Tauri event "events_changed"
  → Svelte events store
  → components re-render
```

Writes from pimple go through the same pipeline. The writer writes an `.ics` file; the watcher observes it; the index updates; the UI re-renders. The write path never touches the index directly. The filesystem is the single source of truth.

### IPC surface

Commands (`#[tauri::command]`):

- `list_collections() -> Vec<Collection>`
- `events_in_range(start, end, visible_collections) -> Vec<EventInstance>` — range arguments are RFC 9557 strings
- `create_event(CreateEventRequest) -> Result<String, Error>` — returns the new event's UID
- `set_collection_visible(id, visible)`
- `get_config() -> AppConfig`, `set_config(AppConfig)`

Events emitted by the backend:

- `events_changed` — payload is the change kind plus affected UIDs. Frontend re-queries for full data.

All boundary types derive `serde::{Serialize, Deserialize}` and generate matching TypeScript definitions via `ts-rs`.

## 4. Data model

```rust
/// Persisted event definition. One per iCalendar UID.
struct Event {
    uid: String,
    collection_id: CollectionId,
    summary: String,
    description: Option<String>,
    location: Option<String>,
    start: EventTime,
    end: EventTime,
    rrule: Option<RRule>,
    exdates: Vec<EventTime>,
    overrides: Vec<OverrideInstance>,
    created_at: jiff::Timestamp,
    modified_at: jiff::Timestamp,
    raw_ics: String,   // original file text; preserved for round-trip
}

/// All four iCalendar DATE-TIME forms modeled explicitly.
enum EventTime {
    AllDay(jiff::civil::Date),
    Floating(jiff::civil::DateTime),
    Utc(jiff::Timestamp),
    Zoned(jiff::Zoned),
}

/// A single concrete occurrence. Derived on query; never persisted.
struct EventInstance {
    event_uid: String,
    instance_start: EventTime,
    instance_end: EventTime,
    is_override: bool,
}

/// A calendar, mapped to one subdirectory of the vdir root.
struct Collection {
    id: CollectionId,          // directory name; stable identifier
    path: PathBuf,
    display_name: String,      // from `displayname` file, falling back to directory name
    color: String,             // CSS hex (`#RRGGBB`), from `color` file or auto-assigned from a fixed palette
    visible: bool,             // app-local state; never written back to the vdir
}
```

### Round-trip preservation

`raw_ics` holds the original file content. On future edit, we parse, mutate, and patch the parsed data back into the raw, so unknown fields (VALARM, X-vendor properties, attachments) are preserved. Dropping unknown fields on write is a common data-loss bug in half-finished calendar clients; pimple refuses to do that even while limited to create-only in v0.1.

### Multi-VEVENT per file

A single `.ics` file may contain a master VEVENT plus override VEVENTs (sharing a UID, differing by RECURRENCE-ID) representing edited instances of a recurring event. The parser collapses these into one `Event` with a populated `overrides` list. One UID = one file = one `Event`.

## 5. `pimple-core` internals

Six modules, each small and independently testable.

### `vdir::layout`

Pure filesystem-structure knowledge. `enumerate_collections(root) -> Vec<Collection>` and `read_collection_metadata(path) -> Collection` (reads `displayname`, `color`, falls back to defaults). No event-content parsing.

### `ical::parse`

Wraps the `icalendar` crate. Converts file text to `Event` or a parse error. Handles the DATE / DATE-TIME discrimination that `EventTime` models. Groups multi-VEVENT files by UID and attaches overrides to the master. Preserves `raw_ics` as the input text. Works in chrono types internally (forced by `icalendar`'s API) and converts to jiff at its output boundary.

### `ical::expand`

Wraps the `rrule` crate. Given an `Event` and a date range, yields `EventInstance`s. Applies EXDATE subtraction and override replacement. Non-recurring events take a one-instance fast path. Works in chrono internally and converts at the boundary.

### `index`

The in-memory event store. Effectively `HashMap<Uid, Event>` plus a secondary index by collection. API:

- `events_in_range(start, end, visible_collections) -> Vec<EventInstance>`
- `subscribe() -> broadcast::Receiver<IndexChange>`, where `IndexChange = Added(String) | Updated(String) | Removed(String) | FullReload` (the string is the affected event's UID)

The index is rebuilt by the watcher, never mutated directly by callers. Single writer, many readers; behind an `RwLock`.

### `watcher`

Owns a `notify` watcher over the vdir root. Translates raw filesystem events into `IndexChange` events after re-parsing the affected file. Debounces (a single logical write often fires several FS events). Collection directory add/remove emits `FullReload`.

### `write`

The create path. `create_event(request) -> Result<String>` (returning the new UID):

1. Generate a UUID for the UID.
2. Build a minimal well-formed VCALENDAR + VEVENT string with the RFC 5545 required fields plus the requested optional ones.
3. Write to `<collection_path>/<uid>.ics` via temp-file + rename, atomic on the same filesystem.
4. The watcher observes the new file. The index update flows through the normal pipeline.

The write path does not touch the index directly. External writes (pimsync, text editor) and internal writes take the same code path through the watcher.

### Concurrency

Async API on tokio (runtime owned by the Tauri layer). Index behind an `RwLock`; reads are cheap and frequent, writes from the watcher are short and rare. Broadcast channel for change subscriptions.

### Datetime handling

Public types use `jiff`. Internal parser and expander use chrono, forced by upstream crate APIs. Conversion is confined to two files: `ical::parse::from_chrono` and `ical::expand::to_jiff`. Everything outside those two files speaks jiff only. The IPC boundary serializes jiff types as RFC 9557 strings.

## 6. `pimple-tauri` bridge layer

Thin by design.

- `tauri.conf.json` — one window, strict CSP, no unused plugins.
- `#[tauri::command]` functions matching the IPC surface above.
- A Tokio task on app startup that subscribes to the core's broadcast channel and re-emits changes as Tauri events.
- A `tauri::State<PimpleCore>` holding the instantiated core.

Window lifecycle in v0.1: one window; no tray, no background mode. Closing exits the app. If the user wants pimsync to keep running, that is pimsync's concern.

The frontend has no direct filesystem capability — `@tauri-apps/plugin-fs` is not in the allowlist. All FS access is gated through Rust commands.

## 7. Frontend structure

Svelte 5 + TypeScript + Vite.

```
frontend/src/
├── lib/
│   ├── ipc.ts                    # typed Tauri command wrappers
│   ├── stores/
│   │   ├── events.svelte.ts
│   │   ├── collections.svelte.ts
│   │   └── config.svelte.ts
│   ├── time/
│   │   └── parse.ts              # RFC 9557 parsing; native Temporal where available, polyfill otherwise
│   └── components/
│       ├── WeekGrid.svelte
│       ├── EventBlock.svelte
│       ├── CollectionSidebar.svelte
│       ├── EventModal.svelte
│       └── Navigator.svelte
├── App.svelte
├── app.css
└── main.ts
```

### Stores

Svelte 5 runes. `events.svelte.ts` exposes `$state`-backed visible instances and an `init()` that subscribes to the Tauri `events_changed` event and re-queries the current visible range when it fires. Queries are debounced so a burst of filesystem changes yields one re-render.

### Week grid rendering

CSS Grid: seven day columns by 24 hour rows. Events position via computed `grid-row-start` and `grid-row-end`. Overlapping events per day are resolved by a column-split algorithm — sort by start time, group overlap clusters, assign parallel columns within each cluster, render with `grid-column-start` adjustments inside the day's sub-grid. This is the one non-trivial UI primitive, and it is unit-tested directly.

### Temporal in the frontend

Native `Temporal` is shipping unflagged in Chromium 144; Tauri's webview on each desktop picks it up as those platforms ship. Until then, bundle `@js-temporal/polyfill`. RFC 9557 strings parse via `Temporal.ZonedDateTime.from()`. Both sides of the IPC boundary share the Temporal model.

### Design direction

One restrained design language across all platforms. Respects system light/dark and accent color via CSS custom properties and `prefers-color-scheme`. No attempt to mimic native widgets — Tauri webviews cannot. Quiet, readable, keyboard-first. Accessibility is baseline: full keyboard navigation through grid and events, ARIA roles, focus management on modal open/close, `prefers-reduced-motion` respected.

Specific visual direction is deliberately open at spec time; it is revisited once there is running code to iterate on.

## 8. Error handling

`pimple-core` uses `thiserror` for typed library errors. `pimple-tauri` converts core errors to a serializable IPC error type.

Three classes:

1. **Parse errors on a specific file** — log via `tracing`, continue loading other files, surface a non-blocking UI notification listing affected filenames. One broken `.ics` does not take down the calendar.
2. **Filesystem errors on the vdir root** — fatal at startup (no vdir, nothing to show); recoverable mid-session (temporarily unmounted volume: retry with backoff, then surface).
3. **Write errors** — atomic temp-file-rename makes partial writes impossible, so failures reduce to permission or disk-space issues. Surfaced directly in the create modal.

Fail loudly on programmer errors (invalid UID in a write request, nonexistent collection ID). Fail gracefully on user-world errors (malformed external data, transient filesystem issues). No silent exception swallowing anywhere.

## 9. Testing strategy

`pimple-core`:

- Unit tests for vdir layout, iCalendar parsing, and RRULE expansion. Fixtures lifted from `vdird` plus added cases: all-day, floating, zoned, UTC, DST transitions, recurring with override, multi-VEVENT file.
- Property tests via `proptest` for the chrono↔jiff conversion layer. Round-trip any valid datetime and assert equality.
- No mocking. Real `tempdir` filesystems.

`pimple-tauri`:

- Integration tests boot the core against a `tempdir` vdir, invoke commands, and assert responses.
- `tauri::test::MockRuntime` for window-less testing.

Frontend:

- Vitest + `@testing-library/svelte` for component tests. Direct unit tests for the week-grid overlap algorithm.
- Playwright E2E against a built app. Smoke test: start, see events, create one, see it appear, restart, see it persist.

CI, zero-warnings policy from day one:

- `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --check`.
- `oxlint`, `vitest`, `tsc --noEmit`.
- `cargo deny check` for supply-chain.

## 10. Delivery phasing

- **v0.1** — read, week view, multi-collection sidebar, create-event form modal. This spec.
- **v0.2** — edit event, delete event (with recurring-instance semantics), month view, first-run vdir-root picker.
- **v0.3+** — VTODO, vCard, natural-language quick entry, reminders and notifications, mobile via Tauri 2.

The v0.1 architecture is sized for v0.2 without refactor. The `write` path is symmetric for edit and delete. `EventTime` accommodates the full recurrence-editing data model. `raw_ics` preserves unknown fields through round-trips.

## 11. Open questions

- **First-run vdir discovery** — v0.1 accepts the vdir root via config file or CLI flag; a file picker comes in v0.2.
- **Window geometry persistence** — deferred to v0.2.
- **Visual design specifics** — deliberately not pinned here; iterated on during implementation.
- **License** — GPL or AGPL, matching the project's ethos. Finalized before first public release.

## 12. References

- [vdir format](https://vdirsyncer.pimutils.org/en/stable/vdir.html)
- [pimsync](https://pimsync.whynothugo.nl/)
- [RFC 5545 (iCalendar)](https://www.rfc-editor.org/rfc/rfc5545)
- [RFC 9557 (IXDTF)](https://www.rfc-editor.org/rfc/rfc9557)
- [Temporal proposal](https://tc39.es/proposal-temporal/)
- [jiff](https://github.com/BurntSushi/jiff)
- [temporal_rs](https://github.com/boa-dev/temporal)
- Prior iteration: [vdird](https://github.com/ryzokuken/vdird)
