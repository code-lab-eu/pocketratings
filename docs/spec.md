# Pocket Ratings — Specification

## Goals

- **Audience**: Single user or a family. No multi-tenant or public deployment.
- **Purpose**: Keep a personal record of product purchases: what you bought, where, for how much, and how you’d rate and review it.
- **Outcomes**: Track products and categories; record purchases (product, location, quantity, unit price in EUR); allow multiple reviews per (user, product) over time; soft-delete and retain history.
- **Non-goals for v1**: No social features, no public reviews, no multi-currency (EUR only). No barcode scanning (browser support is limited—e.g. Barcode Detection API works on Chrome Android but not Safari iOS; consider later with feature detection and fallback).

---

## User flows

**Account**

- **Register**: In v1, registration is **CLI-only** (not exposed in the REST API). User provides name, email, password → account created (password hashed with Argon2).
- **Login**: User provides email and password → session/token for API and web app (only unauthenticated API endpoint; all others return 403 if not authenticated).
- **Delete**: User can be soft-deleted or removed (CLI only). Delete is only allowed if the user has no purchases or reviews.

**Categories**

- **Create**: User creates a category (name; optional parent for hierarchy).
- **List**: User sees categories (tree or flat).
- **Update / soft-delete**: User can rename a category. Soft-delete is only allowed if the category has no child categories and no products (move or delete children and products first).

**Locations (stores)**

- **Create**: User adds a location (name).
- **List**: User sees all locations.
- **Update / soft-delete**: User can rename or soft-delete a location. Soft-delete is only allowed if the location has no purchases.

**Products**

- **Create**: User adds a product (name, brand, category).
- **List**: User sees products (filter by category, search by name/brand).
- **Update / soft-delete**: User can edit product or soft-delete it.

**Purchases**

- **Record**: User records a purchase: product, location, quantity, unit price (EUR), date. In v1 the current user is always recorded as the purchaser.
- **List**: User sees purchases (e.g. by product, by location, by date range, or “my purchases”).

- **Update / soft-delete**: User can edit a purchase (product, location, quantity, price, date) or soft-delete it. Edit and delete only for their own purchases.

**Reviews**

- **Write**: User adds a review for a product (rating 1–5, optional text). Multiple reviews per (user, product) allowed over time.
- **List**: User sees reviews (by product, or “my reviews”).
- **Update / soft-delete**: User can edit or soft-delete a review.

**Reading / dashboard (optional for v1)**

- View “recent purchases”, “recent reviews”, or a product’s purchase and review history. Detail can be defined when designing the API and UI.

---

## Frontend (web app)

The main use case for the web app is **in-store decision making**: the user is in a shop (e.g. supermarket) facing a large selection (wines, coffees, cheeses, sausages). They take out their phone, open the app, and want to quickly see which products in a category they already tried and how they rated them, so they can decide what to buy (or avoid).

**Implications:**

- **Lookup-first, not data-entry-first** — Browsing categories and searching products, plus at-a-glance ratings, must be the default. Adding/editing categories, products, purchases, and reviews is secondary and lives behind a menu.
- **Mobile-first** — One-handed use, large touch targets, minimal chrome, fast load and interaction.
- **Fast and simple** — Few taps to "category → list of products with my rating"; search with quick feedback.

**Information architecture**

| Priority     | Area                    | Description |
|-------------|-------------------------|-------------|
| **Primary** | Category browse         | List categories (flat or tree from `GET /api/v1/categories` with optional `parent_id`). Tap category → products in that category. |
| **Primary** | Search                  | Full-text product search by name/brand (`GET /api/v1/products?q=...`). Results show the user's rating when available. |
| **Primary** | Product list with ratings | For a chosen category (or search), show products with **my** rating and optional one-line review. Merge `GET /api/v1/products?category_id=X` (or `?q=`) with `GET /api/v1/reviews` (my reviews) in the frontend; key by `product_id`. |
| **Primary** | Product detail          | Tap product → full review(s), optional purchase history. Uses `GET /api/v1/products/:id`, `GET /api/v1/reviews?product_id=:id`. |
| **Secondary** | Auth                  | Login (`POST /api/v1/auth/login`); store JWT (e.g. localStorage); handle `X-New-Token` refresh. Registration remains CLI-only. |
| **Secondary** | Management            | Single entry point (e.g. hamburger or "More" menu) for: Categories CRUD, Locations CRUD, Products CRUD, Purchases, Reviews. All existing REST endpoints. |

The home screen is **category list + search**. No dashboard or "recent activity" on the main screen for v1.

**Screens**

- **Home:** Categories (list or tree) + prominent search. If unauthenticated → redirect to Login.
- **Category products:** Products in category with inline rating (and optional short review). Products + "my reviews" merged client-side.
- **Search results:** Same product+rating list, driven by `products?q=...` + my reviews.
- **Product detail:** Full review(s), purchase history; optional quick "rate again" or "log purchase" actions.
- **Login:** Email + password; store token; redirect to Home.
- **Menu:** Single place for all entity management (categories, locations, products, purchases, reviews).

**Data flow (current API, no backend changes)**

- **Categories:** `GET /api/v1/categories` (optionally `?parent_id=...` for tree). Cache after first load for speed.
- **Products in category:** `GET /api/v1/products?category_id=<uuid>`.
- **Product search:** `GET /api/v1/products?q=<string>` (name/brand).
- **My ratings for product list:** `GET /api/v1/reviews` (default: current user). Merge with product list in the frontend by `product_id`; show latest or best rating per product (e.g. most recent review).
- **Product detail:** `GET /api/v1/products/:id`, `GET /api/v1/reviews?product_id=:id`. Purchases: `GET /api/v1/purchases?product_id=:id` if needed.

Two-call pattern for list views (products + my reviews) is sufficient; no new backend endpoint required.

---

## Data models

All primary keys are UUIDs. Timestamp columns are stored as **64-bit integers**
(UNIX time: seconds since 1970-01-01 UTC). In SQLite this is `INTEGER`.

### User

| Field      | Type              | Notes                          |
|------------|-------------------|--------------------------------|
| id         | UUID              | Primary key                    |
| name       | string            |                                |
| email      | string            | Unique, used for login         |
| password   | string            | Argon2 hash (PHC format: algorithm, params, salt, hash; one salt per password) |
| created_at | integer (UNIX)    | Set on create                  |
| updated_at | integer (UNIX)    | Set on create and update       |
| deleted_at | integer (UNIX)?   | Set when soft-deleted; null = active |

### Category

| Field      | Type              | Notes                          |
|------------|-------------------|--------------------------------|
| id         | UUID              | Primary key                    |
| parent_id  | UUID              | Optional; self-reference for hierarchy |
| name       | string            |                                |
| created_at | integer (UNIX)    | Set on create                  |
| updated_at | integer (UNIX)    | Set on create and update       |
| deleted_at | integer (UNIX)?   | Set when soft-deleted; null = active |

### Product

| Field      | Type              | Notes                          |
|------------|-------------------|--------------------------------|
| id         | UUID              | Primary key                    |
| category_id| UUID              | Foreign key → Category         |
| brand      | string            |                                |
| name       | string            |                                |
| created_at | integer (UNIX)    | Set on create                  |
| updated_at | integer (UNIX)    | Set on create and update       |
| deleted_at | integer (UNIX)?   | Set when soft-deleted; null = active |

### Location (Store)

| Field      | Type              | Notes       |
|------------|-------------------|-------------|
| id         | UUID              | Primary key |
| name       | string            |             |
| deleted_at | integer (UNIX)?   | Set when soft-deleted; null = active |

### Review

| Field      | Type              | Notes                          |
|------------|-------------------|--------------------------------|
| id         | UUID              | Primary key                    |
| product_id | UUID              | Foreign key → Product          |
| user_id    | UUID              | Foreign key → User             |
| rating     | decimal           | 1–5 stars, decimal subdivisions (e.g. 4.5) |
| text       | string?           | Optional review text            |
| created_at | integer (UNIX)    | Set on create                  |
| updated_at | integer (UNIX)    | Set on create and update       |
| deleted_at | integer (UNIX)?   | Set when soft-deleted; null = active |

### Purchase

| Field        | Type              | Notes                |
|--------------|-------------------|----------------------|
| id           | UUID              | Primary key          |
| user_id      | UUID              | Foreign key → User; who made the purchase |
| product_id   | UUID              | Foreign key → Product |
| location_id  | UUID              | Foreign key → Location |
| quantity     | integer           | Number of items; default 1   |
| price        | decimal           | Unit price (EUR; currency hardcoded) |
| purchased_at | integer (UNIX)    | When the purchase occurred |
| deleted_at   | integer (UNIX)?   | Set when soft-deleted; null = active |

---

## API


The REST API documentation is available in [docs/api.md](api.md). It includes:

- Authentication (JWT token-based)
- Best practices (HTTP status codes, protected fields, error handling)
- Complete endpoint documentation for all resources
- Request/response formats and examples

Runnable HTTP examples are available in [docs/api.http](api.http).

---

## Deployment

- **Same domain**: Backend and frontend are served from the **same domain** (e.g. `https://pocketratings.example.com`). The API is under `/api/v1/`; all other paths are handled by the frontend (Svelte).
- **Nginx**: The monorepo includes an **nginx** configuration that splits traffic: requests to `/api/v1/` are proxied to the backend (Rust API); all other requests are proxied to the frontend (Svelte). This allows a single entry point and avoids CORS for same-origin requests.

**Configuration**

- App configuration is read from **environment variables**. The backend exposes a `config` module (`config/mod.rs`) that loads them. No config file is required; env vars are the single source of truth.

**Environment variables:**

- `DB_PATH` — Database path (default: `./pocketratings.db`)
- `JWT_SECRET` — JWT signing secret (**required**)
- `BIND` — Server bind address (default: `127.0.0.1:3099`)
- `PID_FILE` — Path to PID file for daemon mode (default: temporary directory, e.g., `/tmp/pocketratings.pid` on Unix, `%TEMP%\pocketratings.pid` on Windows)

---

## CLI

The CLI is the same binary as the backend (`pocketratings`). It operates on the **same SQLite database** as the API (no HTTP). Use it for registration (v1-only way to create users), admin, scripting, and starting/stopping the API server. Database path: configurable via env (e.g. `DB_PATH`); default e.g. `./pocketratings.db` or a standard app data path.

**Server**

- `pocketratings server start [--bind <addr>] [--daemon]` — Start the API server. Bind address from `--bind` or env (e.g. `BIND`); default `127.0.0.1:3099` (port 3099 to avoid clashes with common dev ports like 8080/3000). Foreground by default; `--daemon` runs in background and writes a PID file so it can be stopped later. PID file location is configurable via `PID_FILE` environment variable; defaults to a temporary directory (e.g., `/tmp/pocketratings.pid` on Unix, `%TEMP%\pocketratings.pid` on Windows).
- `pocketratings server stop` — Stop the server if it was started with `--daemon` (read PID file, send SIGTERM). Exit with error if no PID file or process not running.

**User (account)**

- `pocketratings user register --name <name> --email <email> --password <password>` — Create a user (v1: only way to register). Password hashed with Argon2 before store.
- `pocketratings user list` — List users (e.g. for admin; optional for v1).
- `pocketratings user delete <id> [--force]` — Soft-delete a user by UUID (default). Use `--force` to remove the user row from the database. Fails if user has purchases or reviews.

**Categories**

- `pocketratings category create --name <name> [--parent-id <uuid>]`
- `pocketratings category list [--parent-id <uuid>]`
- `pocketratings category show <id>`
- `pocketratings category update <id> [--name <name>] [--parent-id <uuid>]`
- `pocketratings category delete <id> [--force]` — Soft-delete by default; use `--force` to remove the row. Fails if category has any child categories or products.

**Locations**

- `pocketratings location create --name <name>`
- `pocketratings location list`
- `pocketratings location show <id>`
- `pocketratings location update <id> --name <name>`
- `pocketratings location delete <id> [--force]` — Soft-delete by default; use `--force` to remove the row. Fails if location has purchases.

**Products**

- `pocketratings product create --name <name> --brand <brand> --category-id <uuid>`
- `pocketratings product list [--category-id <uuid>] [--q <search>]`
- `pocketratings product show <id>`
- `pocketratings product update <id> [--name <name>] [--brand <brand>] [--category-id <uuid>]`
- `pocketratings product delete <id> [--force]` — Soft-delete by default; use `--force` to remove the row. Fails if product has purchases.

**Purchases**

- `pocketratings purchase create --product-id <uuid> --location-id <uuid> --price <amount> [--user-id <uuid>] [--quantity <n>] [--at <iso-date>]` — Default quantity 1, `--at` default now. If `--user-id` omitted, require e.g. `--email` to identify the purchaser (v1: one user per family device or explicit flag).
- `pocketratings purchase list [--user-id <uuid>] [--product-id <uuid>] [--location-id <uuid>] [--from <date>] [--to <date>]`
- `pocketratings purchase show <id>`
- `pocketratings purchase delete <id> [--force]` — Soft-delete by default; use `--force` to remove the row.

**Reviews**

- `pocketratings review create --product-id <uuid> --rating <1-5> [--user-id <uuid>] [--text <text>]` — If `--user-id` omitted, require e.g. `--email`.
- `pocketratings review list [--product-id <uuid>] [--user-id <uuid>]`
- `pocketratings review show <id>`
- `pocketratings review update <id> [--rating <1-5>] [--text <text>]`
- `pocketratings review delete <id> [--force]` — Soft-delete by default; use `--force` to remove the row.

**Conventions**

- IDs are UUIDs. List commands exclude soft-deleted records unless `--include-deleted` (or similar) is set.
- Output: human-readable by default; optional `--output json` for scripting.
- Same validation and business rules as the API (e.g. category name unique per parent; category delete only when no child categories and no products).

---

## Backend crates (Rust)

Dependencies for the backend (API + CLI, SQLite). All under the same binary.

| Crate | Purpose |
|-------|--------|
| **axum** | HTTP server: routing, JSON extractors, middleware. Fits async and tower ecosystem. |
| **tower** | Middleware (e.g. auth layer that returns 403 when no valid token). |
| **tokio** | Async runtime (required by axum). Use `full` or only needed features. |
| **sqlx** | Async SQLite driver; compile-time checked queries; built-in migrations (`sqlx migrate`). |
| **argon2** | Password hashing (PHC format). Already specified in design notes. |
| **jsonwebtoken** | JWT: issue token on login, validate on protected routes. Stateless; no session store. |
| **serde**, **serde_json** | Serialization for request/response and CLI `--output json`. |
| **uuid** | UUID type with `serde` feature for IDs. |
| **rust_decimal** | Decimal for price and rating (no float rounding). Serde support. |
| **clap** | CLI argument parsing (derive API, subcommands for user/category/location/product/purchase/review). |
| **tracing**, **tracing-subscriber** | Structured logging; env-based level (e.g. `RUST_LOG` via `EnvFilter`). Preferred over env_logger for async/axum. |
| **thiserror**, **anyhow** | Error types and context (thiserror for library errors, anyhow in bin). |
| **dotenv** | Load `.env` into env vars for local dev. Call `dotenv::dotenv().ok()` early in `main`; production sets env directly. |

**Notes**

- **Local env**: Load `.env` at startup so `DB_PATH`, `JWT_SECRET`, etc. can be set in a file (gitignored) for local development.
- **Auth**: JWT with a secret (env e.g. `JWT_SECRET`). Login returns a token; frontend and CLI send `Authorization: Bearer <token>`. No session table for v1.
- **Migrations**: SQL files in `backend/migrations/`; run via `sqlx migrate run` at startup or out-of-band. Include in deployment/CLI.
- **CLI**: Uses same `config` and same DB as API (via `Config::from_env()` and sqlx pool). No HTTP from CLI.

**Rust coding workflow**

- Test-driven development: every feature has a test. In addition to unit tests, require tests for all CLI commands and all REST endpoints.
- Safe code: no `unwrap()` or unsafe patterns in production code; use `Result` and `?`.
- Proper error handling: thiserror for library errors, anyhow where appropriate; API and CLI map errors to status codes and messages.
- Full workflow and checklist are in the project skill [.cursor/skills/rust-backend-workflow/SKILL.md](.cursor/skills/rust-backend-workflow/SKILL.md); the agent applies it when working on the backend.

---

## Design notes and constraints

**What works well**

- UUIDs for all primary keys: no sequential leaks, simple and safe for a small app.
- Timestamps as 64-bit UNIX time (integer): consistent, portable, and efficient in SQLite.
- Category hierarchy via optional `parent_id`: standard and flexible.
- Decimal for rating and price: appropriate for subdivisions and currency.

**Constraints to enforce**

- **User**: `email` unique (one account per email).
- **Review**: Multiple reviews per (user, product) allowed (taste and products change over time). Rating in range 1–5; allow decimal steps (e.g. 0.5) if desired.
- **Category**: Category name is unique per parent (same name allowed under different parents). Soft-delete a category only if it has no products (enforce in API/CLI).

**Password hashing (Argon2)**

- Use the **argon2** crate. Per-password salt is best practice: each user gets a unique random salt so identical passwords produce different hashes and rainbow tables are useless.
- The crate outputs a single string in **PHC format** (e.g. `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`), which embeds algorithm, parameters, salt, and hash. Store that string in `User.password`; no separate salt column is needed. Verification is: hash the submitted password with the same algorithm/params/salt and compare.
- Ensure the stored value is long enough for the PHC string (e.g. 200+ characters is safe).

**Currency**

- All purchase amounts are in **EUR** (hardcoded; no currency field).

**Soft deletes**

- Every entity has **deleted_at** (nullable integer, UNIX time). Null = active; set to UNIX time (64-bit integer) when soft-deleted. List/read queries filter `WHERE deleted_at IS NULL` unless explicitly including deleted records.

**Other**

- **Purchase total**: Total paid = `price` × `quantity` (price is always unit price).
