# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Completed

- **API error responses: JSON and frontend 404** — Backend returns all 4xx/5xx
  as JSON (`ErrorBody` with `error` + `message`); stable codes (`not_found`,
  `bad_request`, etc.). Frontend parses JSON, throws `ApiClientError` with
  status/errorCode; detail pages set `notFound` when status === 404. Documented
  in [api.md](api.md).

- **Category list: optional `depth` parameter and nested response** — `GET
  /api/v1/categories` accepts optional `depth` (e.g. `depth=1` for one level)
  and optional `parent_id`; list response is a nested tree with `children`.
  Backend: `get_children(pool, Option<parent_id>)`, `Categories::from_list`
  with depth and include_deleted, in-memory cache in db layer. Frontend:
  `Category.children`, `flattenCategories`; category listing components
  refactored. Documented in [api.md](api.md).

- **Categories: single list function with `include_deleted` option** —
  Replaced `get_all()` and `get_all_with_deleted()` with
  `get_all(pool, include_deleted: bool)`; single cache (full list), filter
  when `include_deleted == false`. All API and test call sites updated.

- **Reviews API: include product and user in response** — Review list/detail
  responses include nested `product: { id, brand, name }` and
  `user: { id, name }`. Backend: `list_with_relations` / `get_by_id_with_relations`
  with JOINs; API reuses `ProductRef` and `UserRef`. In-memory cache for
  review list (invalidated on insert/update/delete). Frontend: manage/reviews
  uses `review.product` and `review.user`; product detail shows “By
  {user.name}”. Documented in [api.md](api.md).

---

## Planned

Prioritized: blocking tasks and quick wins first, then items that depend on
them, then larger improvements.

### 1. Identify reusable frontend components

**Goal:** Systematically find duplicated UI patterns in the frontend that can be
extracted into reusable components (e.g. in `$lib`), to reduce duplication and
keep behaviour and styling consistent.

**Tasks:**
- Audit routes and pages for repeated patterns: forms (inputs, selects,
  buttons), list rows (links, actions), empty states, navigation chunks.
- Produce a short list of candidate components with suggested props/slots and
  consumer pages.
- Prioritise and implement extractions (or add to this roadmap as separate
  items). Prefer small, focused components over large ones.

### 2. Purchases API: include location in response — Done

**Goal:** Purchase list/detail responses include location data (e.g. nested
`"location": { "id", "name" }` or `location_name`) so the frontend can show
location without a separate `GET /api/v1/locations` call.

**Tasks:**
- Extend purchase list and detail responses with location; document in
  [api.md](api.md). Frontend can then drop client-side resolution by id.

**Done:** Backend returns nested `user`, `product`, and `location` on list, get, create, and update; documented in api.md and api.http. Frontend types and pages (manage/purchases, product detail) to be updated per plan.

### 3. Reusable search on home and category pages

**Goal:** The search currently on the homepage is also shown on category pages,
implemented as a single reusable component.

**Tasks:**
- Extract the search form/UI from the home page into a reusable component
  (e.g. in `$lib`), with props for current `q` and form action/base URL (or use
  client-side navigation + URL so it works on both `/` and `/categories/:id`).
- Use this component on the home page and on the category page
  (`/categories/[id]`), so search on category page filters categories/products
  in the same way (or scoped to that category if we keep current API
  behaviour; spec can clarify). Document in [spec.md](spec.md) that search
  appears on both home and category pages.

### 4. Management list UX: edit/delete icons; entity name → view page

**Goal:** On management list pages (categories, products, locations, reviews,
purchases), use separate **Edit** and **Delete** actions as **icons** (not
text links/buttons). The **entity name** links to the **public entity page**
when it exists (e.g. category → `/categories/:id`, product → `/products/:id`);
otherwise it is not a link (or remains secondary link to edit, per
product/category).

**Tasks:**
- Introduce shared list-row pattern or small component: entity name (optional
  link to view URL), Edit icon (link to `/manage/.../[:id]`), Delete icon
  (existing delete handler). Use consistent icon set (e.g. pencil, trash) and
  `aria-label` for accessibility.
- **Categories:** Name → `/categories/:id` (view); Edit icon →
  `/manage/categories/:id`; Delete icon → delete.
- **Products:** Name → `/products/:id` (view); Edit icon →
  `/manage/products/:id`; Delete icon → delete.
- **Locations:** No public location page → name is plain text (or keep link
  to edit); Edit icon → `/manage/locations/:id`; Delete icon → delete.
- **Reviews:** Keep product name → `/products/:id`; add Edit icon → edit
  review (e.g. `/manage/reviews/[:id]` if exists, or add); Delete icon →
  delete.
- **Purchases:** No public purchase page → primary text not a link (or link to
  product); add Edit icon → edit purchase; Delete icon → delete.
- Update [spec.md](spec.md) (Management / list behaviour) to describe: “List
  rows: entity name links to view page when it exists; separate Edit and
  Delete icon actions.”

### 5. Category list: immediate children only with inline expand

**Goal:** On the homepage and on category pages, the category list shows only
**immediate children** (one level), not the full tree. Each category in the
list can be **expanded inline** via a link/control that loads and shows its
children; this expand control is shown **only if the category has children**.
Requires the REST API to expose a **`has_children: bool`** on each category in
list/detail responses so the frontend can show/hide the expand link without
extra requests.

**Ties to:** Category list optional `depth` parameter (completed). — use `depth=1` to
fetch only direct children for the initial list; when user expands a category,
fetch its children (e.g. `GET /api/v1/categories?parent_id=<id>&depth=1`) and
render them inline.

**Tasks:**
- Backend: Add `has_children: bool` to category payloads returned by the REST
  API (list and by-id). Compute from existence of any non-deleted child
  category. Document in [api.md](api.md).
- Frontend (home + category page): Request only direct children (e.g.
  `depth=1` (available); or use `parent_id` and ensure only one
  level is shown). Render each category with an expand control only when
  `has_children === true`. On expand, fetch children for that category and
  render them inline (nested or indented). Update [spec.md](spec.md) so
  category list behaviour is “immediate children only; expand to show children
  inline when present.”

### 6. Category page: products from current category and all child categories

**Goal:** On the category page, show all products that belong to the current
category **and** to any descendant category (full subtree). Use a depth limit
(e.g. depth 5) for “child categories” to avoid unbounded trees.

**Blocked by:** Category list optional `depth` parameter (completed).

**Tasks:**
- Backend (if not already covered): support listing products for a category
  subtree (e.g. new query param or multiple category IDs); document in
  [api.md](api.md).
- Frontend: category page data load uses current category + descendant
  category IDs (e.g. from categories API with depth, or new endpoint); fetch
  products for that set and merge/deduplicate; show in existing product list.
  Use depth of 5 for subtree. Update [spec.md](spec.md) so category products
  include “current + all descendant categories” and reference depth.

### 7. Home page: search by category name

**Goal:** Search on the homepage includes products whose category (or ancestor)
name matches the search term, not only product name/brand.

**Tasks:**
- Backend: Extend `GET /api/v1/products?q=...` to include products linked to
  a category whose name matches `q`. Frontend may need no change if backend
  handles it.

### 8. Home page: live-updating search

**Goal:** Search results update as the user types (e.g. URL + debounced
refetch), without requiring form submit.

**Tasks:**
- Keep search on home; update URL (e.g. `replaceState`) and refetch as user
  types; optional debounce (e.g. 300 ms). Client-side navigation, no full
  reload.

### 9. CLI timestamp management

**Goal:** `updated_at` and `deleted_at` set automatically in the database
layer (like the REST API), not manually in each CLI command.

**Tasks:**
- Database layer: set `updated_at` on update, `deleted_at` on soft-delete.
- Remove manual timestamp setting from CLI commands (category, location,
  product, review, purchase update/delete). Verify `created_at` on create.

### 10. Product variations

**Goal:** Products can be sold in different variations (e.g. mayonnaise in
different jar sizes). Purchases track prices, so we need to differentiate
between buying a big jar or a small jar by associating purchases with product
variations.

**Summary:**
- New model and database table for product variations (linked to product).
- Purchases are associated with a product variation so price history is
  per-variation (e.g. small vs large jar).
- User-selectable unit per variation: grams, milliliters, other, or no unit;
  keep UX simple.
- When creating a new product, create an initial product variation
  automatically.

**Tasks:**
- Add product variation model and migration; link variations to products.
- Add unit field (grams, milliliters, other, or none) and ensure easy
  selection in UI.
- On product create, create one initial variation.
- Associate purchases with a product variation (API, DB, frontend).
- Document in [spec.md](spec.md) and [api.md](api.md).

### 12. Locations: single list function with `include_deleted` option

**Goal:** Replace `get_all()` and `get_all_with_deleted()` in the location DB
layer with one function, e.g. `get_all(pool, include_deleted: bool)`, to reduce
duplication and keep caching logic in one place (same pattern as categories).

**Tasks:**
- Add `include_deleted: bool` parameter to the location list function; merge
  the two implementations.
- Update all call sites (API and tests).
- Single cache (full list including deleted); filter when
  `include_deleted == false`.

### 13. Products: single list function with `include_deleted` option

**Goal:** Same refactor for the product DB layer: one list function (e.g.
`get_all` or `get_all_filtered`) with `include_deleted: bool`; update call
sites; cache strategy as for categories/locations.

**Tasks:**
- Add `include_deleted: bool` to the product list function(s); merge
  implementations.
- Update all call sites (API and tests).
- Single cache (full list including deleted); filter when
  `include_deleted == false`.

### 14. Add cargo-llvm-cov to QA

**Goal:** Run code coverage (cargo-llvm-cov) as part of backend quality
assurance so new or changed code is measured and, optionally, coverage
thresholds can be enforced.

**Tasks:**
- Install and run cargo-llvm-cov for the backend (e.g. in CI and/or in the
  backend QC skill). Generate reports (e.g. `--lcov`, `--html`).
- Optionally: add `--fail-under-lines` (or similar) to fail the build when
  overall line coverage drops below a threshold.
- Document in the backend QC skill and any CI workflow how to run coverage
  and interpret results.

### 16. Products API: include category in response — **Done**

**Done:** Product list and detail responses include nested `category: { id, name }`. Backend uses `ProductWithRelations` with a categories JOIN, in-memory product list cache (invalidated on mutations), and `CategoryRef` in the API. Frontend uses `product.category` for the category link and label; product detail page no longer fetches category separately. Documented in [api.md](api.md).

### 17. Category API: include parent in response

**Goal:** Category list/detail include nested `parent: { id, name } | null`
so breadcrumbs or parent display do not require a separate category fetch.

**Tasks:**
- Backend: JOIN parent category in list/get; add optional parent ref to
  category response.
- Frontend: use `category.parent` where parent name or link is needed.
- Document in [api.md](api.md).

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
