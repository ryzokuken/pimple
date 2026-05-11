# TDD workflow for pimple

A pocket guide to running tests fast enough to iterate red→green→clean on a single test in seconds, not minutes.

## Inner loops

The justfile ships fine-grained recipes that wrap the cheapest possible invocation:

| Recipe | What it runs | Typical cold/warm |
|---|---|---|
| `just tc <filter>` | `cargo test -p pimple-core <filter>` | 20 s / 1 s |
| `just tt <filter>` | `cargo test -p pimple-tauri <filter>` | 30 s / 2 s |
| `just tf <filter>` | `pnpm exec vitest run -t "<filter>"` | 3 s / 1 s |
| `just tfw` | `vitest` watch mode | persistent |
| `just tdd-core <filter>` | clippy + tc, both `-p pimple-core` | 25 s / 2 s |
| `just lint-rust` | fmt-check + clippy strict | 30 s / 5 s |
| `just lint-fe` | tsc + oxlint | 5 s / 3 s |

The `<filter>` is a **test-function-name substring** — e.g., `just tc parses_zoned` matches `parses_zoned_event`. It is **not** a file name match. To target a single file, use `cargo test -p pimple-core --test parse_single`.

## Outer loops

| Recipe | Use |
|---|---|
| `just check` | Full CI matrix locally. Run before committing. ~2 min cold. |
| `just e2e` | Playwright. First run downloads Chromium (~150 MB). |
| `just release` | Production frontend + release Rust binary. Confirms packaging compiles. |
| `just build` | Same as `release` but mirrors the CI script. |

## Red → green pattern

1. **Add a failing test first**.
   - Rust: append to an existing `tests/<topic>.rs` or create a new one. Name the function so the filter is precise (`parses_dst_transition`, not `test_one`).
   - Frontend: append to `tests/<topic>.test.ts` using `describe + test` from vitest.
2. **Confirm red**.
   - `just tc <name>` should exit non-zero. If it passes, the test isn't actually checking what you think.
3. **Minimum code to go green**.
   - Resist refactoring during the green step. Make the test pass with the *smallest possible* change. Refactor as a separate commit.
4. **Confirm green**.
   - `just tc <name>` exits zero.
5. **Lint clean**.
   - `just lint-rust` for backend, `just lint-fe` for frontend. The workspace lints are strict (`unwrap_used = "deny"`, `panic = "deny"`, etc.) — fix before commit, not after.
6. **Commit**.
   - One TDD cycle = one logical commit. Subject in lowercase imperative (`core: add raw_ics patch path`). Body explains *why* the change is needed if it's not obvious from the diff.

## Concurrency-aware tests

`watcher.rs` integration tests touch a real filesystem and use 150 ms debouncing. If you run them in parallel with other watcher tests on the same `tempdir`, races appear. Two options:

- Single test at a time: `just tc watcher_picks_up_new_file`.
- Use `--test-threads=1` if running multiple: `cargo test -p pimple-core --test watcher -- --test-threads=1`.

For non-watcher tests, parallelism is fine.

## Frontend tip: keep Vitest in watch mode

Run `just tfw` in a side terminal. It re-runs only affected tests on save, giving instant feedback. Use `just tf` for one-shot CI-shaped runs in scripts/CI.

## What "green" includes

Before a commit lands:

- `cargo fmt --all -- --check` — zero diffs
- `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings
- `cargo test --workspace` — every test passes
- `pnpm check` — `tsc --noEmit` clean
- `pnpm test` — vitest clean
- `pnpm build` — Vite production build succeeds

`just check` runs all of the above in one invocation; if it exits 0, you're safe to commit.

## When you hit a clippy lint you can't quickly silence

The workspace pins `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `print_stdout`, `print_stderr` at `deny` or `warn-with-deny-on-pedantic`. Genuine intentional uses of `expect` need an explicit:

```rust
#[expect(
    clippy::expect_used,
    reason = "system zone failure is unrecoverable here"
)]
```

Don't reach for `#[allow]` (workspace denies `allow_attributes`). The required `reason` makes future-you accountable. If you can't articulate why the panic is acceptable, the lint is right and the code needs a `Result` instead.

## When you need a watcher test to wait

The watcher debounces at 150 ms. After mutating a file in a test, sleep at least 200 ms before asserting the index reflects the change. Use `tokio::time::sleep(Duration::from_millis(300)).await` to leave headroom. Existing examples: `tests/watcher.rs`.

## The shape of a good pimple test

- Names a single observable behavior (`parses_zoned_event`, not `test_parse`).
- Uses a `tempdir` for any FS interaction; never touches `$HOME`.
- Compares structurally, not by stringifying — for `Event`, assert each field rather than `format!("{:?}", event)`.
- Includes the spec section in a comment when testing a standards-correctness behavior (`// RFC 5545 §3.6.1: DTSTART of DATE type with no DTEND defaults to one day`).
- Lives in `tests/` (integration), not in-line in `src/` modules, unless the test specifically needs `crate::private_helper` access.
