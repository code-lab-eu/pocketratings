# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Planned

Order: (1) blocking tasks, (2) important, (3) low-hanging fruit (1–2 SP), (4)
rest. Every item has a story point estimate in the first line of its body.

### 1. Design tokens and focus states in layout.css [FE] — DONE

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

### 2. Accessibility audit [FE] — DONE

**2 sp.** Evaluate and fix accessibility issues across the frontend. For
example: the dark/light mode switch and log-out messages are not clearly
identifiable as clickable (e.g. no pointer cursor on hover, no focus/active
affordance). **Important.** Do task 1 first so focus-visible styles are in
place before the audit.

**Tasks:**
- Audit interactive elements: ensure clickable controls (theme toggle, log
  out, buttons, links) use `cursor: pointer` (or equivalent) and have clear
  focus/active states where appropriate.
- Fix theme switch and log-out UI so they are recognisable as interactive
  (cursor, aria, and/or visible affordance).
- Optionally run axe or similar and address critical/serious findings; doc
  any deferred items.

### 3. Page titles for all pages [FE] — DONE

**1 sp.** Ensure every page has a meaningful `<title>` for the browser tab,
bookmarks, and accessibility. Home and product detail already have titles; add
or standardise titles for category page, login, manage hub, and all manage
list/form pages so every route sets a descriptive title.

### 4. Replace emoji with Lucide icons in header and BackLink [FE]

**1 sp.** Use the existing lucide-svelte library for header (menu, sun, moon)
and BackLink (arrow) so the app has a consistent, accessible icon language
instead of emoji that render differently across platforms.

**Tasks:**
- In +layout.svelte: replace ☰ with Menu, ☀ with Sun, ☾ with Moon; keep
  aria-label/title for theme toggle.
- In BackLink.svelte: replace "←" with ArrowLeft (or ChevronLeft), size
  appropriately, aria-hidden="true" on the icon so the link label is the
  screen-reader focus.

### 5. Unify card and list styling on product detail page [FE]

**1 sp.** Use the design system (pr-card and related utilities) for review
cards and purchase history on the product detail page instead of ad-hoc
Tailwind so all card-like surfaces share one visual language.

**Tasks:**
- In products/[id]/+page.svelte: style review cards with pr-card (or
  variant) instead of inline rounded-lg border...; style purchase list
  items with pr-card or the same list pattern used elsewhere so borders,
  background, and hover align with the rest of the app.

### 6. Order categories alphabetically [FE+BE]

**1 sp.** Show categories in alphabetical order by name wherever they are
listed (home, category page, API tree). Today the API and CLI return
categories in undefined or insertion order.

**Tasks:**
- Backend: When building the category tree or returning list responses, sort
  sibling categories by name (e.g. in `categories_to_response_list` and
  tree construction). Verify CLI category list order; if it does not sort by
  name, include CLI in scope. Ensure REST list/tree order is consistent.
  Document in [api.md](api.md) that category list/tree order is by name
  ascending.
- Frontend: Rely on API order; no change if backend returns sorted. If
  frontend sorts locally elsewhere, align with same rule (name ascending).

### 7. Category page: products from current category and all child categories [FE+BE] — DONE

**2 sp.** On the category page, show all products that belong to the current
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

### 8. API error and request logging [BE]

**2 sp.** Log API errors and optionally request/response status so that
failures (e.g. 4xx/5xx) are visible in the backend process output. Currently
handler errors are not logged.

**Tasks:**
- Add logging when a handler returns an error: e.g. a Tower middleware that
  logs response status (and request method/path) for 4xx/5xx, or log at the
  point where `ApiError` is returned. Use `tracing` (already in use at
  startup); avoid logging sensitive data (e.g. no tokens or full bodies).
- Prefer one consistent approach (middleware vs. per-handler); document in
  README or dev docs how to enable debug logs if needed.

### 9. ProductList styling: star rating, price, layout [FE] — DONE

**2 sp.** Improve the design of ProductList so rating and price are shown
attractively with reusable components and a compact, well-spaced layout.
Apply UI designer principles: reusable components, clear visual hierarchy,
accessibility (e.g. aria-label for star rating).

**Tasks:**
- Add a reusable star-rating component: displays a 1–5 score as 5 stars
  with half and quarter star granularity (e.g. 4.25 shows 4 full, 1 quarter).
  Star rating display follows API range (1–5, step 0.1); quarter/half star is
  display granularity only, not a new rating step. Use in ProductList for
  review_score; ensure accessible (e.g. aria-label).
- Add a reusable price component: accepts amount string and appends the
  euro symbol (e.g. "2.99" -> "2.99 EUR" or "2,99 €" per locale). Use in
  ProductList for price.
- In ProductList: remove the "Rating: " and "Price: " label prefixes; show
  only the star rating and price components.
- Layout: arrange name, brand, star rating, and price inside each list item
  in a compact but attractive way with sufficient whitespace; keep pr-card
  and design system consistency.

### 10. Category list: immediate children only with inline expand [FE+BE]

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
- Manage pages (category parent select, product category select) continue to
  use full tree for dropdowns; do not switch them to depth=1. API must still
  support full tree when depth is omitted.

### 11. Product detail page: inline add review and add purchase [FE]

**3 sp.** On the product detail page, allow the user to add a review or a
purchase using an inline form (e.g. collapsible section or form below the
existing reviews/purchases) instead of navigating to manage/reviews/add or
manage/purchases/add. Reuse existing API (POST review, POST purchase);
prefill product (and default variation for purchase) from the current page.

**Tasks:**
- Add an "Add review" section on the product page: form with rating and
  optional text; product is fixed; on success, refresh data or append
  optimistically and clear form.
- Add an "Add purchase" section: form with variation, location, quantity,
  price, date; product and default variation prefilled; on success, refresh
  or append and clear form.
- Ensure validation and error messages match existing manage pages; link to
  full manage flows for edit/delete. Update [spec.md](spec.md) if product
  page behaviour is specified there.

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
