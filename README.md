# Pocket Ratings

A small app for **personal or family use** to keep track of product
purchases: rate and review what you buy, where you bought it, and what
you paid.

- **Backend:** Rust (REST API + CLI), SQLite
- **Frontend:** Svelte

## What it does

- **Track purchases** — Record products you buy.
- **Rate and review** — Give products a rating and write a review.
- **Where and how much** — Store the place (store or location) where you
  bought each product and the price you paid.

## Structure

- `backend/` — Rust API and CLI
- `frontend/` — Svelte app
- `docs/` — Specification and API docs

## Prerequisites

- **Rust** (stable, 1.85+ for edition 2024)
- **Bun** (latest)
- **Node.js** 24 LTS

## Configuration

The backend is configured via environment variables. In the `backend`
directory, copy the example file to `.env` and edit the values:

```bash
cd backend
cp .env.example .env
```

Edit `.env` to set at least `JWT_SECRET` (required). You can change
`DB_PATH` and `BIND` if needed. The backend loads `.env` from its current
working directory, so run the backend from `backend/` (e.g. `cargo run`).
In production, set the variables in the environment instead of using a
file.

## Building for production

### Backend

```bash
cd backend
cargo build --release
```

The binary will be at `backend/target/release/pocketratings`.

### Frontend

```bash
cd frontend
bun install
bun run build
```

The built site output location depends on the Svelte setup (e.g. `frontend/build` for SvelteKit static build).

## Development

### Backend

```bash
cd backend
cargo build
cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings
cargo test

# Start the server
cargo run -- server start
```

The backend API will be available at `http://127.0.0.1:3099` (or the address configured in your `.env` file).

### Frontend

Once the backend server is running, start the frontend dev server:

```bash
cd frontend
bun install
bun run dev
```

The dev server will start at `http://localhost:3000`.

## Lint

```bash
# Backend (all targets: lib, bin, tests, examples)
cd backend && cargo clippy --all-targets --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings

# Frontend
cd frontend && bun run lint
```

## Tests

Run tests for both backend and frontend:

```bash
# Backend
cd backend && cargo test --release

# Frontend (Vitest: unit and component tests)
cd frontend && bun run test
```

## Pre-push hook

From the repo root, run `./scripts/pre-push.sh` to run backend (format, clippy, test) and frontend (lint, test) checks. To install as a git hook so it runs before every push:

```bash
ln -sf ../../scripts/pre-push.sh .git/hooks/pre-push
```

## License

GPL-3.0 — copyleft; derivatives and forks must remain open source. See
[LICENSE](LICENSE).
