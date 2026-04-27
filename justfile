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
