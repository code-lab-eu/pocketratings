# Pocket Ratings

A small app for **personal or family use** to keep track of product
purchases: rate and review what you buy, where you bought it, and what
you paid.

- **Backend:** Rust (REST API + CLI), SQLite
- **Frontend:** Nuxt 4

## What it does

- **Track purchases** — Record products you buy.
- **Rate and review** — Give products a rating and write a review.
- **Where and how much** — Store the place (store or location) where you
  bought each product and the price you paid.

## Structure

- `backend/` — Rust API and CLI
- `frontend/` — Nuxt 4 app
- `docs/` — Specification and API docs

## Prerequisites

- **Rust** (stable, 1.85+ for edition 2024)
- **Bun** (latest)
- **Node.js** 24 LTS (required by Bun)

The same build/lint/test commands are used in CI and are documented in
[.cursor/skills/build-and-test/SKILL.md](.cursor/skills/build-and-test/SKILL.md).

## Building

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

The built site will be in `frontend/.output`.

## Development

### Backend

```bash
cd backend
cargo build
cargo clippy --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings
cargo test --release
```

### Frontend

```bash
cd frontend
bun install
bun run dev
```

The dev server will start at `http://localhost:3000`.

## Lint

Same commands as in CI (see [.github/workflows/ci.yml](.github/workflows/ci.yml)):

```bash
# Backend
cd backend && cargo clippy --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings

# Frontend
cd frontend && bun run lint
```

## Tests

Same commands as in CI:

```bash
# Backend
cd backend && cargo test --release

# Frontend
cd frontend && bun run test
```

## License

GPL-3.0 — copyleft; derivatives and forks must remain open source. See
[LICENSE](LICENSE).
