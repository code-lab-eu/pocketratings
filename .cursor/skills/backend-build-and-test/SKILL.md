---
name: backend-build-and-test
description: Run build, lint, and test for the Pocket Ratings backend (Rust). Use when the user asks to build, lint, or test the backend, or verify backend before submit.
---

# Backend Build and Test

Use this skill when running backend build, lint, or test. Run commands from the repo root with `cd backend` or set `working-directory: backend` in CI.

**For full quality control** (format, then lint, then test with strict Clippy), use the **backend-quality-control** skill instead.

## Consistency expectations

When adding or changing backend code, follow existing patterns so build and lint stay clean and the codebase stays consistent.

- **CLI modules** (`backend/src/cli/`): Use full-word module names (e.g. `database`, not `db_cmd`). No `_cmd` or two-letter shorthand. Add a module alias in `mod.rs`: `use crate::cli::<module> as <module>_cli`.
- **Clap types**: Define `*Args`, `*Cmd`, and `*Opts` in `cli/mod.rs` with the other entity types — not in the handler module. Use full words for both the variant and the CLI subcommand (e.g. `Database(DatabaseArgs)` so the command is `database`).
- **Match arms in `cli::run`**: Use the type name directly (e.g. `DatabaseCmd::Backup`) and call the handler via the alias (e.g. `database_cli::backup(...)`). Do not use inline `modulename::TypeName` in the match.
- **Database access**: Queries belong in the `db` crate (`backend/src/db/`). CLI handlers call `db::*` functions; they do not use `sqlx::query` (or similar) directly.
- **Handler signatures**: Pass decomposed arguments (e.g. `output: Option<&str>`) into handler functions, not whole opts structs, matching how other entity handlers are called from `mod.rs`.
- **Module docs**: Use the same style as existing CLI modules (e.g. `//! Database subcommands (backup).`). In doc comments, put identifiers and env names in backticks (e.g. `` `DB_PATH` ``) for `doc_markdown` compliance.

**If `cargo` fails with "rustup could not choose a version" or "no default is configured":** run `rustup default stable` (with network permission) first, then retry. Do not treat it as a code bug.

## Build

```bash
cd backend
cargo build --release
```

## Lint

Strict pedantic mode; all warnings are errors. Run Clippy on **all targets** (lib, bin, tests, examples):

```bash
cd backend
cargo clippy --all-targets --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings
```

## Test

Skip the long-running `server_start_and_stop_via_cli` test (starts/stops the server; can exceed timeouts):

```bash
cd backend
cargo test --release -- --skip server_start_and_stop_via_cli
```

## All checks (build + lint + test)

```bash
cd backend
cargo build --release && \
cargo clippy --all-targets --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings && \
cargo test --release -- --skip server_start_and_stop_via_cli
```
