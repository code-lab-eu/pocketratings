# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Planned

Order: (1) blocking tasks, (2) important, (3) low-hanging fruit (1–2 SP), (4)
rest. Every item has a story point estimate in the first line of its body.

### 1. API error and request logging [BE]

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

### 2. Product detail page: inline add purchase [FE]

**2 sp.** In the Purchase history section, keep **Add purchase** as a link
(full manage add page remains available). Place the link at the bottom of
the section. Same swap-to-inline pattern as inline add review. Reuse `POST
/api/v1/purchases`; prefill product and default variation from the current
page (e.g. extend `PurchaseForm.svelte` or equivalent). Load locations in
`products/[id]` data when needed. Remove **Add purchase** from the footer;
remove the footer/actions block entirely if it becomes empty. [spec.md](spec.md)
still describes **Add purchase** in the actions area in places; update those
rows when this ships.

**Tasks:**
- `listLocations()` in `products/[id]/+page.ts` alongside existing loads;
  handle errors consistently with the rest of the page.
- Inline form: variation, location, quantity, price, date; validation aligned
  with manage add purchase; on success refresh (or append) and restore the
  link.
- Reuse or extend `PurchaseForm.svelte` with props for fixed product and
  variations from `GET /api/v1/products/:id` where practical.

### 3. Deprioritize variation label when unit is set on new product [FE]

**2 sp.** On `manage/products/new`, the optional first-variation **Label**
field is shown with equal weight to unit and quantity, but label is often
redundant when unit and quantity imply the size. Reduce visual emphasis
(order, typography, or progressive disclosure) so **Label** is prominent
mainly when unit is **No unit** (`variationUnit === 'none'` in code), where
a free-text descriptor matters.

**Tasks:**
- Adjust layout so unit and quantity lead; surface label clearly only for the
  `none` unit case.
- Keep `first_variation` submit rules unchanged; extend tests if behaviour is
  covered.

### 4. Refactor add/edit review forms to a reusable component [FE]

**2 sp.** Today review fields (rating, optional text) and validation are
duplicated across `manage/reviews/add`, `manage/reviews/[id]`, and the inline
form on `products/[id]`. Extract a single component (or small composition)
that handles the shared markup, accessibility, and client-side validation.
The component is **agnostic to layout**: it does not know inline vs full
page; the caller wraps it (e.g. `pr-inline-form`, page chrome, `slide`). The
**product `Select`** is shown **only when no product id is passed in** (e.g.
manage add from hub with no `product_id` query); when a product id is fixed,
omit the dropdown and bind rating/text to submit. Other call-site behaviour
(`goto` vs `invalidateAll`, edit vs create) stays in parents via callbacks.

**Tasks:**
- Introduce a reusable review form component (or pair of presentational +
  submit helpers) used by all three flows; align error strings and rating
  handling with [spec.md](spec.md) and
  [error-message-formatting](.cursor/rules/error-message-formatting.mdc).
- Encode "show product `Select` iff no product id" as a clear prop (or
  equivalent); keep edit and inline add flows on fixed product id without a
  dropdown.
- Preserve existing transitions and `pr-inline-form` usage on the product
  page; no behaviour regression for manage add/edit.
- Extend or add component tests where behaviour is non-trivial; run frontend
  QC.

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
