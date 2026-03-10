# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Planned

Order: (1) blocking tasks, (2) important, (3) low-hanging fruit (1–2 SP), (4)
rest. Every item has a story point estimate in the first line of its body.

### 1. Management list UX: edit/delete icons; entity name → view page [FE] - DONE

**2 sp.** On management list pages (categories, products, locations, reviews,
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
- Update [spec.md](spec.md) (Management / list behaviour) to describe: "List
  rows: entity name links to view page when it exists; separate Edit and
  Delete icon actions."

### 2. Home page: search by category name [BE] - DONE

**2 sp.** Search on the homepage includes products whose category (or ancestor)
name matches the search term, not only product name/brand.

**Tasks:**
- Backend: Extend `GET /api/v1/products?q=...` to include products linked to
  a category whose name matches `q`. Frontend may need no change if backend
  handles it.

### 3. Home page: live-updating search [FE] - DONE

**2 sp.** Search results update as the user types (e.g. URL + debounced
refetch), without requiring form submit.

**Tasks:**
- Keep search on home; update URL (e.g. `replaceState`) and refetch as user
  types; optional debounce (e.g. 300 ms). Client-side navigation, no full
  reload.

### 4. Category cache: O(1) id-based lookup [BE] - DONE

**2 sp.** Add id→category lookup to the category list cache so GET
`/api/v1/categories/:id` can be served from cache when warm (no DB
round-trip for the category row).

**Tasks:**
- Backend: when building the category cache, add
  `HashMap<Uuid, Category>` (or equivalent) alongside the existing list
  and ancestor map; expose a cache-backed lookup (e.g. get_by_id from
  cache when warm, else DB and optionally warm cache).
- GET category by id uses cache when populated for the category row;
  ancestors already come from cache.
- Document in [api.md](api.md) if response behaviour changes; update
  backend cache docs/tests.

### 6. Frontend: require 2 spaces indentation [FE] - DONE

**2 sp.** Standardise frontend code on 2 spaces for indentation instead of
tabs, so diffs and terminal viewing are consistent.

**Tasks:**
- Add EditorConfig and/or Prettier (or ESLint indent rule) to enforce 2
  spaces in frontend (e.g. `frontend/`).
- Reformat existing frontend files (Svelte, TS, CSS) from tabs to 2 spaces.

### 7. Identify reusable frontend components [FE] - DONE

**3 sp.** Systematically find duplicated UI patterns in the frontend that can
be extracted into reusable components (e.g. in `$lib`), to reduce duplication
and keep behaviour and styling consistent.

**Tasks:**
- Audit routes and pages for repeated patterns: forms (inputs, selects,
  buttons), list rows (links, actions), empty states, navigation chunks.
- Produce a short list of candidate components with suggested props/slots and
  consumer pages.
- Prioritise and implement extractions (or add to this roadmap as separate
  items). Prefer small, focused components over large ones.

### 8. Category list: immediate children only with inline expand [FE+BE]

**3 sp.** On the homepage and on category pages, the category list shows only
**immediate children** (one level), not the full tree. Each category in the
list can be **expanded inline** via a link/control that loads and shows its
children; this expand control is shown **only if the category has children**.
Requires the REST API to expose a **`has_children: bool`** on each category in
list/detail responses so the frontend can show/hide the expand link without
extra requests. Uses existing `depth=1` and `parent_id` from categories API.

**Tasks:**
- Backend: Add `has_children: bool` to category payloads returned by the REST
  API (list and by-id). Compute from existence of any non-deleted child
  category. Document in [api.md](api.md).
- Frontend (home + category page): Request only direct children (e.g.
  `depth=1` (available); or use `parent_id` and ensure only one
  level is shown). Render each category with an expand control only when
  `has_children === true`. On expand, fetch children for that category and
  render them inline (nested or indented). Update [spec.md](spec.md) so
  category list behaviour is "immediate children only; expand to show children
  inline when present."

### 9. Category page: products from current category and all child categories [FE+BE]

**3 sp.** On the category page, show all products that belong to the current
category **and** to any descendant category (full subtree). Use a depth limit
(e.g. depth 5) for "child categories" to avoid unbounded trees.

**Tasks:**
- Backend: **Done.** The API supports subtree via
  `GET /api/v1/products?category_id=<uuid>` (no new query params): products
  whose category is that category or any descendant, up to a named constant
  depth; 404 when category not found or deleted. Documented in [api.md](api.md).
- Frontend: category page keeps a single request
  `GET /api/v1/products?category_id=<id>`; no need to fetch descendant IDs or
  merge. Update [spec.md](spec.md) so category products are described as
  "current + all descendant categories" (with depth limit); spec already
  aligned.

### 10. Product variations [FE+BE]

**5 sp.** Products can be sold in different variations (e.g. mayonnaise in
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

### 11. ProductList: review score and price from products API [FE+BE]

**3 sp.** When showing products in `ProductList`, include a review score and a
price. Both values are returned in the product data for `GET
/api/v1/products`, so the frontend does not need a separate call to
`/api/v1/reviews`. Review score is the **median** review score; price is the
**lowest** price. Both are computed on the backend when populating the
products cache.

**Tasks:**
- Backend: When building the products cache, compute per product (1) median
  review score from reviews, (2) lowest price from purchases (or relevant
  price source). Add `review_score` (or equivalent) and `price` (or
  `lowest_price`) to the product payload for `GET /api/v1/products`. Document
  in [api.md](api.md).
- Frontend: Update `ProductList` to display review score and price from the
  product data; remove any separate fetch to `/api/v1/reviews` for list
  display. Update [spec.md](spec.md) if list behaviour is specified there.

### 12. Product page: breadcrumbs with full category path [FE]

**2 sp.** On the product page, show breadcrumbs for the full category path
(Home → ancestor categories → current category → product name), matching the
pattern used on category pages. Reuse the same markup or extract a shared
breadcrumb component so behaviour and styling stay consistent.

**Tasks:**
- Product API already returns `product.category.ancestors`; use it to render
  Home → ancestors (reversed) → category name → product (current page).
- Reuse category-page breadcrumb code: either extract a small `Breadcrumb`
  component (e.g. accepts segments or category + current label) used by both
  category and product pages, or duplicate the nav structure with the same
  classes and aria. Prefer extraction if it keeps a single source of truth.
- Replace the product page "← Home" link with the full breadcrumb nav.
- Update [spec.md](spec.md) if product page navigation is described there.

### 13. DB get_by_id: support retrieving soft-deleted entities [BE]

**2 sp.** Today, `get_by_id` (and `get_by_id_with_relations` where present) exclude
soft-deleted rows in all six entity modules (category, product, user, location,
purchase, review). There is no standard way to load a soft-deleted entity by ID
(e.g. for restore, audit, or admin "show deleted"). List-style functions already
use `include_deleted: bool`; get-by-id does not.

**Tasks:**
- Refactor so callers can retrieve any entity by ID (including soft-deleted)
  without changing API or CLI behaviour. Options: (1) add
  `include_deleted: bool` (default false) to `get_by_id` and
  `get_by_id_with_relations` so existing callers keep active-only semantics and
  new code can pass `true`; or (2) make get-by-id return all rows and have API/CLI
  treat soft-deleted as not found (single consistent check at call sites).
- Apply the same convention to all entity types: category, product, user,
  location, purchase, review (and relation variants where they exist).
- Category cache: when serving get_by_id from cache, honour the same
  include_deleted semantics so behaviour is consistent with the DB path.
- Update tests that assert "get_by_id must exclude soft-deleted" to match the
  chosen design; add tests that verify soft-deleted entities can be retrieved
  when requested.
- Document the chosen contract in the db module docs (and [api.md](api.md) only
  if REST behaviour or docs reference it).

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
- **DB layer: global/singleton pool** — Encapsulate the database inside the
  `db` module by having it own the pool in a process-wide global (e.g.
  `db::init(path)` at startup; API/CLI call `db::category::get_by_id(id)` with
  no db parameter). Revisit when multi-DB support becomes a priority.
