---
name: backend-build-and-test
description: Run build, lint, and test for the Pocket Ratings backend (Rust). Use when the user asks to build, lint, or test the backend, or verify backend before submit.
---

# Backend Build and Test

Use this skill when running backend build, lint, or test. Run commands from the repo root with `cd backend` or set `working-directory: backend` in CI.

**For full quality control** (format, then lint, then test with strict Clippy), use the **backend-quality-control** skill instead.

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

```bash
cd backend
cargo test --release
```

## All checks (build + lint + test)

```bash
cd backend
cargo build --release && \
cargo clippy --all-targets --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings && \
cargo test --release
```
