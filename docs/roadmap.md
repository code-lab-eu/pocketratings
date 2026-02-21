# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Completed

- **API error responses: JSON and frontend 404** — Backend returns all 4xx/5xx
  as JSON (`ErrorBody` with `error` + `message`); stable codes (`not_found`,
  `bad_request`, etc.). Frontend parses JSON, throws `ApiClientError` with
  status/errorCode; detail pages set `notFound` when status === 404. Documented
  in [api.md](api.md).

---

## Planned

### 1. Category list: optional `depth` parameter

**Goal:** Allow `GET /api/v1/categories?parent_id=<id>&depth=1` to return only
direct children (one level), so the category page does not need the full
subtree.

**Tasks:**
- Add optional query parameter `depth` to `GET /api/v1/categories`; when
  `depth=1`, return only direct children.
- Document in [api.md](api.md).

### 2. Purchases API: include location in response

**Goal:** Purchase list/detail responses include location data (e.g. nested
`"location": { "id", "name" }` or `location_name`) so the frontend can show
location without a separate `GET /api/v1/locations` call.

**Tasks:**
- Extend purchase list and detail responses with location; document in
  [api.md](api.md). Frontend can then drop client-side resolution by id.

### 3. Home page: search by category name

**Goal:** Search on the homepage includes products whose category (or ancestor)
name matches the search term, not only product name/brand.

**Tasks:**
- Backend: Extend `GET /api/v1/products?q=...` to include products linked to
  a category whose name matches `q`. Frontend may need no change if backend
  handles it.

### 4. CLI timestamp management

**Goal:** `updated_at` and `deleted_at` set automatically in the database
layer (like the REST API), not manually in each CLI command.

**Tasks:**
- Database layer: set `updated_at` on update, `deleted_at` on soft-delete.
- Remove manual timestamp setting from CLI commands (category, location,
  product, review, purchase update/delete). Verify `created_at` on create.

### 5. Home page: live-updating search

**Goal:** Search results update as the user types (e.g. URL + debounced
refetch), without requiring form submit.

**Tasks:**
- Keep search on home; update URL (e.g. `replaceState`) and refetch as user
  types; optional debounce (e.g. 300 ms). Client-side navigation, no full
  reload.

---

## Distant future

Ideas to revisit later (v2 or when requirements grow). No commitment to order.

- **Loading indicators** — Global progress bar or spinner during navigation;
  deferred because backend is fast and load times are low.
- **Search engine evaluation** — Consider Typesense or similar for full-text
  search, typo tolerance, faceted search; evaluate when catalog or needs grow.
- **Frontend (beyond current data models)** — Favorite/pinned categories
  (e.g. localStorage); recently viewed or recently added products; offline/PWA
  with service worker.
- **Data export** — Export purchases and reviews to CSV/JSON; backup/restore
  UI.
- **Analytics & insights** — Spending trends, most purchased products,
  average ratings by category, price tracking.
- **Barcode scanning** — Product lookup via barcode (browser API; spec lists as
  non-goal for v1; fallback to manual entry).
