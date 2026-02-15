# Pocket Ratings — Specification

(Goals, flows, API summary, CLI, and out-of-scope to be filled later.)

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

## Design notes and constraints

**What works well**

- UUIDs for all primary keys: no sequential leaks, simple and safe for a small app.
- UTC timestamps everywhere: consistent and portable.
- Category hierarchy via optional `parent_id`: standard and flexible.
- Decimal for rating and price: appropriate for subdivisions and currency.

**Constraints to enforce**

- **User**: `email` unique (one account per email).
- **Review**: Multiple reviews per (user, product) allowed (taste and products change over time). Rating in range 1–5; allow decimal steps (e.g. 0.5) if desired.
- **Category**: Decide whether `name` is unique per parent or globally; recommend unique per parent so you can reuse names under different parents.

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
