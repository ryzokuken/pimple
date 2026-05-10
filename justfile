# Default recipe: list available recipes.
default:
    @just --list

# Start the app in dev mode. Runs `pnpm --dir frontend dev` (Vite, hot reload)
# alongside the Tauri shell. Requires `tauri-cli` (`cargo install tauri-cli`).
dev:
    cd crates/pimple-tauri && cargo tauri dev

# One-shot build: production frontend + release Rust binary.
build:
    pnpm --dir frontend install --frozen-lockfile
    pnpm --dir frontend build
    cargo build --release --workspace

# Full check matrix: format, lint, test, type-check, frontend tests, build.
# Mirrors what CI runs.
check:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --workspace
    pnpm --dir frontend check
    pnpm --dir frontend test
    pnpm --dir frontend build

# Playwright end-to-end smoke tests against the built frontend.
# First run downloads Chromium (~150 MB).
e2e:
    pnpm --dir frontend exec playwright install --with-deps chromium
    pnpm --dir frontend exec playwright test

# Format everything in place.
fmt:
    cargo fmt --all
    pnpm --dir frontend exec oxfmt src

# ─── TDD inner loops ─────────────────────────────────────────────────────────
# These are deliberately narrower than `check` so a red→green cycle stays under
# ~5 s on the core and ~3 s on the frontend. Pass a filter as the argument.

# Run a single pimple-core test (or a substring). Example: `just tc parse_single`.
tc filter="":
    cargo test -p pimple-core {{filter}}

# Run a single pimple-tauri integration test.
tt filter="":
    cargo test -p pimple-tauri {{filter}}

# Run a single frontend vitest by name pattern. Example: `just tf layoutWeek`.
tf filter="":
    pnpm --dir frontend exec vitest run -t "{{filter}}"

# Watch-mode vitest. Use when iterating on a frontend test.
tfw:
    pnpm --dir frontend exec vitest

# Clippy + tests on pimple-core only — the fastest red-green-clean cycle.
tdd-core filter="":
    cargo clippy -p pimple-core --all-targets --all-features -- -D warnings
    cargo test -p pimple-core {{filter}}

# Lint the Rust workspace (clippy + fmt-check).
lint-rust:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

# Lint the frontend (tsc + oxlint).
lint-fe:
    pnpm --dir frontend check
    pnpm --dir frontend lint

# Build the release binary only — no bundling, no platform installer.
# Faster than `just build` when you just want to confirm packaging compiles.
release:
    pnpm --dir frontend build
    cargo build --release --workspace

# Regenerate placeholder dev icons under crates/pimple-tauri/icons/.
# Real icons are tracked separately; these unblock local builds when icons/
# is empty (e.g., after a fresh clone, since icons/ is gitignored).
icons-dev:
    python3 scripts/gen_dev_icons.py
