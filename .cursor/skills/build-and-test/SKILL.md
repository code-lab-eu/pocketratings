---
name: build-and-test
description: Run build, lint, and test commands for Pocket Ratings backend (Rust) and frontend (Nuxt). Use when the user asks to build, lint, test, or check code quality for either the backend or frontend.
---

# Build and Test Commands

## Backend (Rust)

**If `cargo` fails with "rustup could not choose a version" / "no default is configured":** run `rustup default stable` (with network permission) first, then retry the cargo command. Do not report the failure as a code issue or ask the user to run locally—apply the fix and re-run.

### Build
```bash
cd backend
cargo build --release
```

### Lint
Use strict pedantic mode with all warnings enabled:
```bash
cd backend
cargo clippy --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings
```

### Test
```bash
cd backend
cargo test --release
```

### All checks (build + lint + test)
```bash
cd backend
cargo build --release && \
cargo clippy --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings && \
cargo test --release
```

## Frontend (Nuxt)

### Install dependencies
```bash
cd frontend
bun install
```

### Build
```bash
cd frontend
bun run build
```

### Lint
```bash
cd frontend
bun run lint
```

### Test
```bash
cd frontend
bun run test
```

### Dev server
```bash
cd frontend
bun run dev
```

## Notes

- Always use `bun` (not `npm`) for frontend commands
- Backend linting uses strict pedantic mode - all warnings are errors
- Release builds are used for CI consistency
- Working directory must be set correctly (`cd backend` or `cd frontend`)

## Running backend commands when cargo is not available

If `cargo test`, `cargo build`, or `cargo clippy` fails with "rustup could not choose a version" or "no default is configured":

1. Run **first** (with network permission): `rustup default stable`
2. Then re-run the same cargo command (e.g. `cargo build` or `cargo test`).

Do not treat this as a code bug or suggest the user run commands locally—run `rustup default stable` and retry. The error message gives the fix.
