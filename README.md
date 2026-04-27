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

```sh
git clone https://github.com/ryzokuken/pimple.git
cd pimple

# Install frontend dependencies (security-conscious by default: no install scripts).
pnpm --dir frontend install

# Build everything in release mode.
cargo build --release --workspace

# Run in dev mode against a vdir at ~/.calendars (set via the running app's
# IPC; in v0.1 there is no first-run picker yet — point it via the
# set_vdir_root command, or wait for v0.2).
cargo run -p pimple-tauri
```

For an iterative dev loop with hot-reload of the frontend:

```sh
cargo install tauri-cli --version '^2'
cd crates/pimple-tauri && cargo tauri dev
```

## Test

The Rust core, the Tauri bridge, and the frontend each have their own test
suites. None of them depend on the others to run.

```sh
# Rust: unit, integration, and property tests for pimple-core and pimple-tauri.
cargo test --workspace

# Lints and formatting (zero warnings policy).
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Frontend: type-check, unit tests (Vitest), and a production build.
pnpm --dir frontend check
pnpm --dir frontend test
pnpm --dir frontend build

# End-to-end smoke tests (Playwright; first run downloads Chromium).
pnpm --dir frontend exec playwright install chromium
pnpm --dir frontend exec playwright test
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
