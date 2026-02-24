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

The built site output location depends on the Svelte setup (e.g.
`frontend/build` for SvelteKit static build).

## Development

**Important:** The backend server must be started before starting the frontend server, as the frontend depends on the backend API.

### Backend

Start the backend server first:

```bash
cd backend
cargo build
cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings
cargo test

# Start the server
cargo run -- server start
```

The backend API will be available at `http://127.0.0.1:3099` (or the
address configured in your `.env` file).

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

From the repo root, run `./scripts/pre-push.sh` to run backend (format,
clippy, test) and frontend (lint, test) checks. To install as a git hook
so it runs before every push:

```bash
ln -sf ../../scripts/pre-push.sh .git/hooks/pre-push
```

## Running with Docker / Podman

You can run the full stack (Caddy reverse proxy, backend API, static
frontend) with Docker or Podman Compose. The proxy routes `/api/v1/` to
the backend and everything else to the frontend; one entry point, no CORS.

**Prerequisites:** Docker with Compose, or Podman with Compose (e.g.
`podman compose` in Podman 4.1+).

1. Copy the root env example and set `JWT_SECRET`:
   ```bash
   cp .env.example .env
   # Edit .env and set JWT_SECRET to a long random string.
   ```

2. From the repo root, start the stack:
   ```bash
   docker compose up -d
   # or: podman compose up -d
   ```

3. Open the app at **http://localhost** (or https://yourdomain.com if you
   configured the Caddyfile with a domain for Let's Encrypt).

For production HTTPS, edit the **Caddyfile** at the repo root: replace
`http://localhost` with your domain (e.g. `https://pocketratings.example.com`).
Caddy will obtain and renew a certificate automatically.

### Running backend CLI commands

When the stack is running, the database lives inside the backend container.
Run CLI commands in that container so they use the same database:

```bash
docker compose exec backend /app/pocketratings <command> [options]
# or: podman compose exec backend /app/pocketratings <command> [options]
```

Examples (registration is CLI-only in v1):

```bash
docker compose exec backend /app/pocketratings user register \
  --name "Jane" --email jane@example.com --password secret
docker compose exec backend /app/pocketratings user list
docker compose exec backend /app/pocketratings category list
```

To run the CLI against a local database instead (e.g. from the repo
with `cargo run`), use the backend's own `.env` in `backend/` and run from
there; see **Configuration** and **Development** above.

### Backup and restore

The database is stored in a named volume `pocketratings_db`, mounted at
`/data` in the backend container. The database file is
`/data/pocketratings.db`.

**Back up the database**

From the repo root, with the stack running:

**Recommended: hot backup without stopping the server**

```bash
docker compose exec backend /app/pocketratings database backup
# or: podman compose exec backend /app/pocketratings database backup
```

This writes a snapshot to `/data/pocketratings.db.backup` inside the
container. Copy it out (replace `CONTAINER_ID` with the backend container
ID or name):

```bash
docker cp CONTAINER_ID:/data/pocketratings.db.backup backup-$(date +%Y%m%d-%H%M%S).db
# or: podman cp CONTAINER_ID:/data/pocketratings.db.backup backup-$(date +%Y%m%d-%H%M%S).db
```

**Alternative: raw copy** (simpler but not a guaranteed consistent
snapshot if the server is writing):

```bash
docker compose exec backend cat /data/pocketratings.db \
  > backup-$(date +%Y%m%d-%H%M%S).db
# or: podman compose exec backend cat /data/pocketratings.db \
#     > backup-$(date +%Y%m%d-%H%M%S).db
```

**Restore the database**

1. Stop the backend:
   ```bash
   docker compose stop backend
   # or: podman compose stop backend
   ```

2. Replace the database in the volume (use the backup file path you have):
   ```bash
   docker run --rm \
     -v pocketratings_pocketratings_db:/data -v "$(pwd)":/backup \
     alpine sh -c "cp /backup/backup-YYYYMMDD-HHMMSS.db /data/pocketratings.db \
       && chown 1000:1000 /data/pocketratings.db"
   ```
   For Podman use `podman run`. Replace `backup-YYYYMMDD-HHMMSS.db` with
   your backup filename. The volume name is `pocketratings_pocketratings_db`
   (Compose project prefix + volume name from compose.yaml).

3. Start the backend again:
   ```bash
   docker compose start backend
   # or: podman compose start backend
   ```

## License

GPL-3.0 — copyleft; derivatives and forks must remain open source. See
[LICENSE](LICENSE).
