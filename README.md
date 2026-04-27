# pimple

A local-first calendar client for the [vdir](https://vdirsyncer.pimutils.org/en/stable/vdir.html)
format. Built on Tauri 2 with a Rust core and a Svelte 5 frontend.

pimple reads and writes events on disk; sync to CalDAV is left to
[pimsync](https://pimsync.whynothugo.nl/) running as a separate tool. There is
no network code in pimple itself.

## Status

Alpha. Functional v0.1 covers a polished week view, multi-collection support,
and event creation. Edit, delete, recurring-event editing, month view, tasks,
contacts, and mobile are out of scope until later versions. See
[`docs/superpowers/specs/2026-04-24-pimple-design.md`](docs/superpowers/specs/2026-04-24-pimple-design.md)
for the v0.1 design and what's coming.

## Prerequisites

- Rust 1.85+ (for edition 2024). Install via [rustup](https://rustup.rs).
- Node 22 LTS and [pnpm](https://pnpm.io). The frontend uses Vite 8 and
  Svelte 5.
- Tauri 2 system dependencies for your platform. See the
  [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) page —
  Linux needs `webkit2gtk-4.1`, `librsvg2`, and `build-essential`; macOS needs
  Xcode command-line tools; Windows needs WebView2.
- A vdir on disk to point pimple at. If you don't have one, install
  `pimsync` and configure a CalDAV pair, or create an empty directory tree
  (`mkdir -p ~/.calendars/personal`) for testing.

## Build and run

The repository ships a [`justfile`](justfile) with the common entry points.
Install [`just`](https://github.com/casey/just) (`cargo install just`) and
[`tauri-cli`](https://v2.tauri.app/reference/cli/) (`cargo install tauri-cli`).

```sh
git clone https://github.com/ryzokuken/pimple.git
cd pimple

just dev      # frontend (Vite) + Tauri shell with hot reload
just build    # production frontend + release binary
just check    # full check matrix: fmt, clippy, tests, type-check, build
just e2e      # Playwright end-to-end smoke tests
just fmt      # format Rust and frontend code in place
```

If you'd rather not use `just`, the equivalents are:

```sh
pnpm --dir frontend install               # frontend deps; ignore-scripts is on by default
cd crates/pimple-tauri && cargo tauri dev # dev mode
cargo build --release --workspace         # release Rust binary
pnpm --dir frontend build                 # production frontend bundle
```

In v0.1 there is no first-run vdir picker. Until that lands in v0.2, point
the running app at your vdir from the webview console:

```js
window.__TAURI__.core.invoke('set_vdir_root', { path: '/home/you/.calendars' });
```

## Test

The Rust core, the Tauri bridge, and the frontend each have their own test
suites. None of them depend on the others.

```sh
just check   # everything CI runs
just e2e     # Playwright (first run downloads Chromium)
```

Or by hand:

```sh
cargo test --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

pnpm --dir frontend check
pnpm --dir frontend test
pnpm --dir frontend build
```

## Project layout

```
crates/
  pimple-core/   Rust library: vdir I/O, iCalendar parse/expand, event index,
                 filesystem watcher, atomic write path.
  pimple-tauri/  Tauri 2 binary: IPC commands, change-event forwarding.
frontend/        Svelte 5 + TypeScript. Reactive stores subscribed to Tauri
                 events; week-grid renderer with overlap-aware layout.
docs/superpowers/
  specs/         Design documents.
  plans/         Implementation plans.
```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

## Contributing

Issues and patches welcome. Before committing, run the full check matrix
above. The repository's CI mirrors it.
