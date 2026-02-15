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

**Categories**

- **Create**: User creates a category (name; optional parent for hierarchy).
- **List**: User sees categories (tree or flat).
- **Update / soft-delete**: User can rename a category. Soft-delete is only allowed if the category has no products (move or delete products first).

**Locations (stores)**

- **Create**: User adds a location (name).
- **List**: User sees all locations.
- **Update / soft-delete**: User can rename or soft-delete a location.

**Products**

- **Create**: User adds a product (name, brand, category).
- **List**: User sees products (filter by category, search by name/brand).
- **Update / soft-delete**: User can edit product or soft-delete it.

**Purchases**

- **Record**: User records a purchase: product, location, quantity, unit price (EUR), date. In v1 the current user is always recorded as the purchaser.
- **List**: User sees purchases (e.g. by product, by location, by date range, or “my purchases”).

**Reviews**

- **Write**: User adds a review for a product (rating 1–5, optional text). Multiple reviews per (user, product) allowed over time.
- **List**: User sees reviews (by product, or “my reviews”).
- **Update / soft-delete**: User can edit or soft-delete a review.

**Reading / dashboard (optional for v1)**

- View “recent purchases”, “recent reviews”, or a product’s purchase and review history. Detail can be defined when designing the API and UI.

---

## Data models

All primary keys are UUIDs. Timestamps are stored in UTC.

### User

| Field    | Type     | Notes                          |
|----------|----------|--------------------------------|
| id       | UUID     | Primary key                    |
| name     | string   |                                |
| email    | string   | Unique, used for login         |
| password | string   | Argon2 hash (PHC format: algorithm, params, salt, hash; one salt per password) |
| created_at | timestamp | Set on create                |
| updated_at | timestamp | Set on create and update     |
| deleted_at | timestamp? | Set when soft-deleted; null = active |

### Category

| Field     | Type     | Notes                          |
|-----------|----------|--------------------------------|
| id        | UUID     | Primary key                    |
| parent_id | UUID     | Optional; self-reference for hierarchy |
| name      | string   |                                |
| deleted_at | timestamp? | Set when soft-deleted; null = active |

### Product

| Field       | Type     | Notes                          |
|-------------|----------|--------------------------------|
| id          | UUID     | Primary key                    |
| category_id | UUID     | Foreign key → Category         |
| brand       | string   |                                |
| name        | string   |                                |
| created_at  | timestamp | Set on create                |
| updated_at  | timestamp | Set on create and update     |
| deleted_at  | timestamp? | Set when soft-deleted; null = active |

### Location (Store)

| Field | Type   | Notes       |
|-------|--------|-------------|
| id        | UUID     | Primary key |
| name      | string   |             |
| deleted_at | timestamp? | Set when soft-deleted; null = active |

### Review

| Field       | Type     | Notes                          |
|-------------|----------|--------------------------------|
| id          | UUID     | Primary key                    |
| product_id  | UUID     | Foreign key → Product          |
| user_id     | UUID     | Foreign key → User             |
| rating      | decimal  | 1–5 stars, decimal subdivisions (e.g. 4.5) |
| text        | string?  | Optional review text            |
| created_at  | timestamp | Set on create                |
| updated_at  | timestamp | Set on create and update     |
| deleted_at  | timestamp? | Set when soft-deleted; null = active |

### Purchase

| Field        | Type     | Notes                |
|--------------|----------|----------------------|
| id           | UUID     | Primary key          |
| user_id      | UUID     | Foreign key → User; who made the purchase |
| product_id   | UUID     | Foreign key → Product |
| location_id  | UUID     | Foreign key → Location |
| quantity     | integer  | Number of items; default 1   |
| price        | decimal  | Unit price (EUR; currency hardcoded) |
| purchased_at | timestamp | When the purchase occurred |
| deleted_at   | timestamp? | Set when soft-deleted; null = active |

---

## API summary

REST over HTTP/JSON. **Base path**: `/api/v1/`. All endpoints live under this path so the rest of the URI space stays free for the frontend.

**Authentication**

- In v1, **only** `POST /api/v1/auth/login` is unauthenticated. All other endpoints must receive a valid token/session; otherwise return **403** (no public data; family/private use only). Registration in v1 is **CLI-only** (no `POST /auth/register` in the API).
- List endpoints exclude soft-deleted records unless a query param requests them.

**Auth**

- `POST /api/v1/auth/login` — Body: `{ email, password }`. Returns token or session. (Only unauthenticated endpoint.)

**Categories**

- `GET /api/v1/categories` — List categories (query: `?parent_id=uuid` for children; omit for root or flat list).
- `GET /api/v1/categories/:id` — Single category.
- `POST /api/v1/categories` — Body: `{ name, parent_id? }`. Name unique per parent.
- `PATCH /api/v1/categories/:id` — Body: `{ name?, parent_id? }`.
- `DELETE /api/v1/categories/:id` — Soft-delete. 400 if category has any products.

**Locations**

- `GET /api/v1/locations` — List locations.
- `GET /api/v1/locations/:id` — Single location.
- `POST /api/v1/locations` — Body: `{ name }`.
- `PATCH /api/v1/locations/:id` — Body: `{ name }`.
- `DELETE /api/v1/locations/:id` — Soft-delete.

**Products**

- `GET /api/v1/products` — List products. Query: `?category_id=uuid`, `?q=search` (name/brand).
- `GET /api/v1/products/:id` — Single product (optionally include purchase/review counts or recent).
- `POST /api/v1/products` — Body: `{ name, brand, category_id }`.
- `PATCH /api/v1/products/:id` — Body: `{ name?, brand?, category_id? }`.
- `DELETE /api/v1/products/:id` — Soft-delete.

**Purchases**

- `GET /api/v1/purchases` — List purchases. Query: `?user_id=uuid` (default: current user), `?product_id=uuid`, `?location_id=uuid`, `?from=date`, `?to=date`.
- `GET /api/v1/purchases/:id` — Single purchase.
- `POST /api/v1/purchases` — Body: `{ product_id, location_id, quantity?, price, purchased_at? }`. `user_id` set to current user; quantity default 1; purchased_at default now.
- `DELETE /api/v1/purchases/:id` — Soft-delete.

**Reviews**

- `GET /api/v1/reviews` — List reviews. Query: `?product_id=uuid`, `?user_id=uuid` (default: current user for “my reviews”).
- `GET /api/v1/reviews/:id` — Single review.
- `POST /api/v1/reviews` — Body: `{ product_id, rating, text? }`. `user_id` set to current user.
- `PATCH /api/v1/reviews/:id` — Body: `{ rating?, text? }`. Only own review.
- `DELETE /api/v1/reviews/:id` — Soft-delete. Only own review.

**Conventions**

- IDs in path and query are UUIDs. Dates in query/body as ISO 8601. Monetary amounts as decimal (e.g. string or number for EUR).
- 4xx/5xx with JSON body `{ error, message? }`. 404 for missing resource; 400 for validation or business rule (e.g. category has products); **403 for unauthenticated** (all endpoints except login); 403 for forbidden (e.g. editing another user’s review).

---

## Deployment

- **Same domain**: Backend and frontend are served from the **same domain** (e.g. `https://pocketratings.example.com`). The API is under `/api/v1/`; all other paths are handled by the frontend (Nuxt).
- **Nginx**: The monorepo includes an **nginx** configuration that splits traffic: requests to `/api/v1/` are proxied to the backend (Rust API); all other requests are proxied to the frontend (Nuxt/Nitro). This allows a single entry point and avoids CORS for same-origin requests.

**Configuration**

- App configuration is read from **environment variables**. The backend exposes a `config` module (`config/mod.rs`) that loads them (e.g. `DB_PATH` for the database path). No config file is required; env vars are the single source of truth.

---

## CLI

The CLI is the same binary as the backend (`pocketratings`). It operates on the **same SQLite database** as the API (no HTTP). Use it for registration (v1-only way to create users), admin, and scripting. Database path: configurable via env (e.g. `DB_PATH`); default e.g. `./pocketratings.db` or a standard app data path.

**User (account)**

- `pocketratings user register --name <name> --email <email> --password <password>` — Create a user (v1: only way to register). Password hashed with Argon2 before store.
- `pocketratings user list` — List users (e.g. for admin; optional for v1).

**Categories**

- `pocketratings category create --name <name> [--parent-id <uuid>]`
- `pocketratings category list [--parent-id <uuid>]`
- `pocketratings category show <id>`
- `pocketratings category update <id> [--name <name>] [--parent-id <uuid>]`
- `pocketratings category delete <id>` — Soft-delete. Fails with error if category has any products.

**Locations**

- `pocketratings location create --name <name>`
- `pocketratings location list`
- `pocketratings location show <id>`
- `pocketratings location update <id> --name <name>`
- `pocketratings location delete <id>` — Soft-delete.

**Products**

- `pocketratings product create --name <name> --brand <brand> --category-id <uuid>`
- `pocketratings product list [--category-id <uuid>] [--q <search>]`
- `pocketratings product show <id>`
- `pocketratings product update <id> [--name <name>] [--brand <brand>] [--category-id <uuid>]`
- `pocketratings product delete <id>` — Soft-delete.

**Purchases**

- `pocketratings purchase create --product-id <uuid> --location-id <uuid> --price <amount> [--user-id <uuid>] [--quantity <n>] [--at <iso-date>]` — Default quantity 1, `--at` default now. If `--user-id` omitted, require e.g. `--email` to identify the purchaser (v1: one user per family device or explicit flag).
- `pocketratings purchase list [--user-id <uuid>] [--product-id <uuid>] [--location-id <uuid>] [--from <date>] [--to <date>]`
- `pocketratings purchase show <id>`
- `pocketratings purchase delete <id>` — Soft-delete.

**Reviews**

- `pocketratings review create --product-id <uuid> --rating <1-5> [--user-id <uuid>] [--text <text>]` — If `--user-id` omitted, require e.g. `--email`.
- `pocketratings review list [--product-id <uuid>] [--user-id <uuid>]`
- `pocketratings review show <id>`
- `pocketratings review update <id> [--rating <1-5>] [--text <text>]`
- `pocketratings review delete <id>` — Soft-delete.

**Conventions**

- IDs are UUIDs. List commands exclude soft-deleted records unless `--include-deleted` (or similar) is set.
- Output: human-readable by default; optional `--output json` for scripting.
- Same validation and business rules as the API (e.g. category name unique per parent; category delete only when no products).

---

## Design notes and constraints

**What works well**

- UUIDs for all primary keys: no sequential leaks, simple and safe for a small app.
- UTC timestamps everywhere: consistent and portable.
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

- Every entity has **deleted_at** (nullable timestamp). Null = active; set to UTC timestamp when soft-deleted. List/read queries filter `WHERE deleted_at IS NULL` unless explicitly including deleted records.

**Other**

- **Purchase total**: Total paid = `price` × `quantity` (price is always unit price).
