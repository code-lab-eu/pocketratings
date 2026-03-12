# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Planned

Order: (1) blocking tasks, (2) important, (3) low-hanging fruit (1–2 SP), (4)
rest. Every item has a story point estimate in the first line of its body.

### 1. Product page: breadcrumbs with full category path [FE] — DONE

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

### 2. DB get_by_id: support retrieving soft-deleted entities [BE] — DONE

**2 sp.** Today, `get_by_id` (and `get_by_id_with_relations` where present)
exclude soft-deleted rows in all six entity modules (category, product, user,
location, purchase, review). There is no standard way to load a soft-deleted
entity by ID (e.g. for restore, audit, or admin "show deleted"). List-style
functions already use `include_deleted: bool`; get-by-id does not.

**Tasks:**
- Refactor so callers can retrieve any entity by ID (including soft-deleted)
  without changing API or CLI behaviour. Options: (1) add
  `include_deleted: bool` (default false) to `get_by_id` and
  `get_by_id_with_relations` so existing callers keep active-only semantics
  and new code can pass `true`; or (2) make get-by-id return all rows and
  have API/CLI treat soft-deleted as not found (single consistent check at
  call sites).
- Apply the same convention to all entity types: category, product, user,
  location, purchase, review (and relation variants where they exist).
- Category cache: when serving get_by_id from cache, honour the same
  include_deleted semantics so behaviour is consistent with the DB path.
- Update tests that assert "get_by_id must exclude soft-deleted" to match
  the chosen design; add tests that verify soft-deleted entities can be
  retrieved when requested.
- Document the chosen contract in the db module docs (and [api.md](api.md)
  only if REST behaviour or docs reference it).

### 3. Category list: immediate children only with inline expand [FE+BE]

**3 sp.** On the homepage and on category pages, the category list shows only
**immediate children** (one level), not the full tree. Each category in the
list can be **expanded inline** via a link/control that loads and shows its
children; this expand control is shown **only if the category has children**.
Requires the REST API to expose a **`has_children: bool`** on each category in
list/detail responses so the frontend can show/hide the expand link without
extra requests. Uses existing `depth=1` and `parent_id` from categories API.

**Tasks:**
- Backend: Add `has_children: bool` to category payloads returned by the
  REST API (list and by-id). Compute from existence of any non-deleted child
  category. Document in [api.md](api.md).
- Frontend (home + category page): Request only direct children (e.g.
  `depth=1` (available); or use `parent_id` and ensure only one level is
  shown). Render each category with an expand control only when
  `has_children === true`. On expand, fetch children for that category and
  render them inline (nested or indented). Update [spec.md](spec.md) so
  category list behaviour is "immediate children only; expand to show
  children inline when present."

### 4. Category page: products from current category and all child categories [FE+BE]

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

### 5. ProductList: review score and price from products API [FE+BE]

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

### 6. Product variations [FE+BE]

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
- On product create, create one initial variation. - DONE
- Associate purchases with a product variation (API, DB, frontend).
- Document in [spec.md](spec.md) and [api.md](api.md).

### 7. Accessibility audit [FE]

**2 sp.** Evaluate and fix accessibility issues across the frontend. For
example: the dark/light mode switch and log-out messages are not clearly
identifiable as clickable (e.g. no pointer cursor on hover, no focus/active
affordance).

**Tasks:**
- Audit interactive elements: ensure clickable controls (theme toggle, log
  out, buttons, links) use `cursor: pointer` (or equivalent) and have clear
  focus/active states where appropriate.
- Fix theme switch and log-out UI so they are recognisable as interactive
  (cursor, aria, and/or visible affordance).
- Optionally run axe or similar and address critical/serious findings; doc
  any deferred items.

### 8. Page titles for all pages [FE]

**1 sp.** Ensure every page has a meaningful `<title>` for the browser tab,
bookmarks, and accessibility. Home and product detail already have titles; add
or standardise titles for category page, login, manage hub, and all manage
list/form pages so every route sets a descriptive title.

### 9. Replace emoji with Lucide icons in header and BackLink [FE]

**1 sp.** Use the existing lucide-svelte library for header (menu, sun, moon)
and BackLink (arrow) so the app has a consistent, accessible icon language
instead of emoji that render differently across platforms.

**Tasks:**
- In +layout.svelte: replace ☰ with Menu, ☀ with Sun, ☾ with Moon; keep
  aria-label/title for theme toggle.
- In BackLink.svelte: replace "←" with ArrowLeft (or ChevronLeft), size
  appropriately, aria-hidden="true" on the icon so the link label is the
  screen-reader focus.

### 10. Unify card and list styling on product detail page [FE]

**1 sp.** Use the design system (pr-card and related utilities) for review
cards and purchase history on the product detail page instead of ad-hoc
Tailwind so all card-like surfaces share one visual language.

**Tasks:**
- In products/[id]/+page.svelte: style review cards with pr-card (or
  variant) instead of inline rounded-lg border...; style purchase list
  items with pr-card or the same list pattern used elsewhere so borders,
  background, and hover align with the rest of the app.

### 11. Design tokens and focus states in layout.css [FE]

**2 sp.** Add a minimal design token layer (CSS custom properties for focus
ring, optional spacing/transition) and visible focus-visible styles for
buttons and card links so theming is easier and keyboard users get clear
focus indicators (WCAG-friendly).

**Tasks:**
- In layout.css: add :root (and html.dark) variables for focus ring
  (e.g. --pr-focus-ring-color, --pr-focus-ring-offset) and optionally
  one or two spacing/transition tokens; use them in pr-btn-* and pr-card.
- Add focus-visible:outline (and optionally ring) to .pr-btn-primary and
  .pr-btn-secondary and ensure interactive pr-card links have a visible
  focus style.

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
