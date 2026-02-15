---
name: build-and-test
description: Run build, lint, and test commands for Pocket Ratings backend (Rust) and frontend (Nuxt). Use when the user asks to build, lint, test, or check code quality for either the backend or frontend.
---

# Build and Test Commands

## Backend (Rust)

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
