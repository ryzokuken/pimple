# pimple

A local-first calendar client for the [vdir](https://vdirsyncer.pimutils.org/en/stable/vdir.html)
format. Built on Tauri 2 with a Rust core and a Svelte 5 frontend.

pimple reads and writes events on disk; sync to CalDAV is left to
[pimsync](https://pimsync.whynothugo.nl/) running as a separate tool. There is
no network code in pimple itself.

## Status

Alpha. v0.2 ships a polished week and month view, a first-run vdir picker,
multi-collection support, event creation, and full edit / delete with
recurring-event semantics (*this instance / this and future / all*). Tasks,
contacts, reminders, natural-language entry, and mobile remain out of scope
until later versions. See
[`docs/superpowers/specs/2026-05-11-pimple-v0.2.md`](docs/superpowers/specs/2026-05-11-pimple-v0.2.md)
for the v0.2 design.

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

just icons-dev       # one-time after a fresh clone: generates placeholder
                     # icons under crates/pimple-tauri/icons/ (gitignored)
just dev             # frontend (Vite) + Tauri shell with hot reload
just build           # production frontend + release binary + native bundle
just check           # full check matrix: fmt, clippy, tests, type-check, build
just e2e             # Playwright end-to-end smoke tests
just fmt             # format Rust and frontend code in place
```

On first launch the app opens a native folder picker; choose your vdir root
(e.g. `~/.calendars`) and the choice persists to:

- Linux: `$XDG_CONFIG_HOME/pimple/config.toml`
- macOS: `~/Library/Application Support/pimple/config.toml`
- Windows: `%APPDATA%\pimple\config.toml`

To change settings (week start, etc.) edit that file and restart the app —
v0.2 has no settings UI yet.

If you'd rather not use `just`, the equivalents are:

```sh
pnpm --dir frontend install               # frontend deps; ignore-scripts is on by default
cd crates/pimple-tauri && cargo tauri dev # dev mode
cargo build --release --workspace         # release Rust binary
pnpm --dir frontend build                 # production frontend bundle
```

## TDD workflow

For inner-loop iteration the justfile ships fine-grained recipes that wrap
the cheapest possible invocation. See [`docs/dev/tdd.md`](docs/dev/tdd.md)
for the red→green→clean pattern.

```sh
just tc <filter>     # cargo test -p pimple-core (substring matches test name)
just tt <filter>     # cargo test -p pimple-tauri
just tf <filter>     # pnpm vitest run -t "<filter>"
just tfw             # vitest watch mode
just tdd-core <name> # clippy + tc, tight cycle on the core
just lint-rust       # fmt-check + clippy strict
just lint-fe         # tsc --noEmit + oxlint
just release         # production frontend + release Rust binary
```

## Test

The Rust core, the Tauri bridge, and the frontend each have their own test
suites. None of them depend on the others. v0.2 totals: **106 Rust + 48 vitest
+ 9 Playwright = 163 tests**.

```sh
just check   # everything CI runs (excludes Playwright)
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
pnpm --dir frontend exec playwright test
```

## Project layout

```
crates/
  pimple-core/    Rust library: vdir I/O, iCalendar parse / expand /
                  patch / build, event index, filesystem watcher, atomic
                  write path (create / update / delete).
  pimple-tauri/   Tauri 2 binary: IPC commands, change-event forwarding,
                  window-geometry persistence, dialog plugin wiring.
frontend/         Svelte 5 + TypeScript. Reactive stores subscribed to
                  Tauri events; week- and month-grid renderers; unified
                  EventModal for create / edit / delete with a
                  recurring-scope dialog.
docs/superpowers/
  specs/          Design documents (v0.1, v0.2).
  plans/          Implementation plans.
docs/dev/
  tdd.md          TDD workflow reference.
scripts/
  gen_dev_icons.py  Placeholder-icon generator (used by `just icons-dev`).
```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

## Contributing

Issues and patches welcome. Before committing, run the full check matrix
above. The repository's CI mirrors it.
