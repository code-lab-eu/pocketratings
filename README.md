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
- `quadlet/` — Podman Quadlet units for running the stack as systemd user units

## Prerequisites

- **Rust** (stable, 1.85+)
- **Bun** (latest)
- **Node.js** 26

## Configuration

The backend is configured via environment variables. In the `backend`
directory, copy the example file to `.env` and edit the values:

```bash
cd backend
cp .env.example .env
```

Edit `.env` to suit your setup. Run the backend from `backend/` so it finds
`.env`. In production, set the variables in the environment instead of using a
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
cd backend && cargo clippy --all-targets --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings
cd frontend && bun run lint
```

## Tests

Run tests for both backend and frontend:

```bash
cd backend && cargo test --release
cd frontend && bun run test
```

## Pre-push hook

From the repo root, run `./scripts/pre-push.sh` to run backend (format,
clippy, test, coverage) and frontend (lint, test) checks. To install as
a git hook so it runs before every push:

```bash
ln -sf ../../scripts/pre-push.sh .git/hooks/pre-push
```

Backend coverage (run by the pre-push script) requires
`cargo install cargo-llvm-cov`.

## Running with Docker / Podman

You can run the full stack (Caddy reverse proxy, backend API, static frontend)
with Docker or Podman Compose, or with Podman Quadlet as systemd user units. The
proxy routes `/api/v1/` to the backend and everything else to the frontend; one
entry point, no CORS. Both setups build the images from the repository and read
the same `Caddyfile` and `.env` at the repo root.

### Compose

**Prerequisites:** Docker with Compose, or Podman with Compose (e.g.
`podman-compose`).

1. Copy the root env example and edit the values:
   ```bash
   cp .env.example .env
   ```

2. From the repo root, start the stack:
   ```bash
   docker compose up -d
   # or: podman-compose -f compose.yaml up -d
   ```

3. Open the app at **http://localhost** (or https://yourdomain.com if you
   configured the Caddyfile with a domain for Let's Encrypt).

For production HTTPS, edit the **Caddyfile** at the repo root: replace the site
address `:80` with your domain (e.g. `https://pocketratings.example.com`). Caddy
will obtain and renew a certificate automatically.

### Podman Quadlet

**Prerequisites:** Linux with systemd and Podman 6.1 or newer.

The unit files are in `quadlet/`. The proxy publishes HTTP on port 3080, which a
rootless container can use; terminate TLS in front of it, or change the
Caddyfile and the published ports as described below.

1. Copy the root env example to `.env` and edit the values. Set `APP_BASE_URL`
   to the address the app is served from, e.g. `http://localhost:3080`:
   ```bash
   cp .env.example .env
   ```

2. Link the unit directory into the Podman configuration of your user and load
   the units:
   ```bash
   mkdir -p ~/.config/containers/systemd
   ln -s "$PWD/quadlet" ~/.config/containers/systemd/pocketratings
   systemctl --user daemon-reload
   ```
   Quadlet follows the link and resolves the relative paths in the units
   (`../backend`, `../frontend`, `../Caddyfile`, `../.env`) against the
   repository. `podman quadlet install` copies the units without the repository,
   so do not use it here.

3. Start the stack. The first start builds both images and takes several
   minutes:
   ```bash
   systemctl --user start pocketratings
   ```

4. Open the app at **http://localhost:3080**.

The stack starts at boot when systemd starts your user instance at boot:
`loginctl enable-linger "$USER"`.

**Settings of the host.** Put host-specific settings in drop-in files instead of
editing the units. A drop-in for the proxy goes in
`~/.config/containers/systemd/pocketratings.container.d/` and one for the
backend in `pocketratings-backend.container.d/` next to it; the file name must
end in `.conf`. An empty `PublishPort=` line removes the default port, for
example to listen on localhost only:

```ini
[Container]
PublishPort=
PublishPort=127.0.0.1:3080:80
```

An `Environment=` line in a backend drop-in takes precedence over the same
variable in `.env`. Run `systemctl --user daemon-reload` after changing a
drop-in, then restart its unit, e.g. `systemctl --user restart pocketratings`.

**Update.** Fetch the new version and restart the backend and the frontend; each
start of their units rebuilds the image from the repository:

```bash
git pull
systemctl --user daemon-reload
systemctl --user restart pocketratings-backend pocketratings-frontend
```

Stop the stack with `systemctl --user stop 'pocketratings*'`.

### Running backend CLI commands

When the stack is running, the database lives inside the backend container.
Run CLI commands in that container so they use the same database:

```bash
docker compose exec backend /app/pocketratings <command> [options]
# or: podman-compose -f compose.yaml exec -T backend /app/pocketratings <command> [options]
# or (Quadlet): podman exec pocketratings-backend /app/pocketratings <command> [options]
```

Examples (registration is CLI-only in v1); with Quadlet, replace
`docker compose exec backend` with `podman exec pocketratings-backend`:

```bash
docker compose exec backend /app/pocketratings user register \
  --name "Jane" --email jane@example.com --password secret
docker compose exec backend /app/pocketratings user list
docker compose exec backend /app/pocketratings user set-password \
  --email jane@example.com --password newsecret
docker compose exec backend /app/pocketratings user reset-link \
  --email jane@example.com
docker compose exec backend /app/pocketratings category list
```

`user reset-link` prints a one-time password reset link (valid for 12 hours,
invalidated once used) built from `APP_BASE_URL`. Send it to the user over a
secure channel; they set their own password without you learning it. Add
`--output json` to get `{ "url", "expires_at" }` instead.

To run the CLI against a local database instead (e.g. from the repo
with `cargo run`), use the backend's own `.env` in `backend/` and run from
there; see **Configuration** and **Development** above.

### Backup and restore

The database is stored in a named volume, `pocketratings_pocketratings_db` with
Compose and `pocketratings-db` with Quadlet, mounted at `/data` in the backend
container. The database file is `/data/pocketratings.db`.

**Back up the database**

From the repo root, with the stack running:

**Recommended: hot backup without stopping the server**

```bash
docker compose exec backend /app/pocketratings database backup
# or: podman-compose -f compose.yaml exec -T backend /app/pocketratings database backup
# or (Quadlet): podman exec pocketratings-backend /app/pocketratings database backup
```

This writes a snapshot to `/data/pocketratings.db.backup` inside the container.
Copy it out (replace `CONTAINER_ID` with the backend container ID or name; with
Quadlet the name is `pocketratings-backend`):

```bash
docker cp CONTAINER_ID:/data/pocketratings.db.backup backup-$(date +%Y%m%d-%H%M%S).db
# or: podman cp CONTAINER_ID:/data/pocketratings.db.backup backup-$(date +%Y%m%d-%H%M%S).db
```

**Alternative: raw copy** (simpler but not a guaranteed consistent
snapshot if the server is writing):

```bash
docker compose exec backend cat /data/pocketratings.db \
  > backup-$(date +%Y%m%d-%H%M%S).db
# or: podman-compose -f compose.yaml exec -T backend cat /data/pocketratings.db \
#     > backup-$(date +%Y%m%d-%H%M%S).db
# or (Quadlet): podman exec pocketratings-backend cat /data/pocketratings.db \
#     > backup-$(date +%Y%m%d-%H%M%S).db
```

**Restore the database**

1. Stop the backend:
   ```bash
   docker compose stop backend
   # or: podman-compose -f compose.yaml stop backend
   # or (Quadlet, also stops the proxy): systemctl --user stop pocketratings-backend
   ```

2. Replace the database in the volume (use the backup file path you have):
   ```bash
   docker run --rm \
     -v pocketratings_pocketratings_db:/data -v "$(pwd)":/backup \
     alpine sh -c "cp /backup/backup-YYYYMMDD-HHMMSS.db /data/pocketratings.db \
       && chown 999:999 /data/pocketratings.db"
   ```
UID and GID 999 belong to the user `app` that runs the backend. For Podman use
`podman run`. Replace `backup-YYYYMMDD-HHMMSS.db` with your backup filename. The
volume name is `pocketratings_pocketratings_db`, or `pocketratings-db` with
Quadlet.

3. Start the backend again:
   ```bash
   docker compose start backend
   # or: podman-compose -f compose.yaml start backend
   # or (Quadlet): systemctl --user start pocketratings
   ```

## License

GPL-3.0 — copyleft; derivatives and forks must remain open source. See
[LICENSE](LICENSE).
