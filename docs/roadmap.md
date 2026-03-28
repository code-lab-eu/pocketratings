# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Planned

Order: (1) blocking tasks, (2) important, (3) low-hanging fruit (1–2 SP), (4)
rest. Every item has a story point estimate in the first line of its body.

### 1. Add a favicon [FE] — DONE

**1 sp.** Ship a tab icon: `frontend/src/lib/assets/favicon.svg` exists but
is not wired in the document head. Link it via SvelteKit convention (`<link
rel="icon" ...>` in root layout or static favicon) so browsers show the app
icon.

**Tasks:**
- Add favicon link (and optional `apple-touch-icon` if desired) consistent
  with SvelteKit 2 / project static asset paths.
- Verify in dev and build output.

### 2. Make the search field more prominent [FE] — DONE

**1 sp.** Home and category pages already expose search per [spec.md](spec.md);
improve visual hierarchy so the search input reads as a primary control
(larger tap target, clearer label or placeholder, spacing) without adding a
separate search route.

**Tasks:**
- Audit home and category page layout: compare search prominence to spec
  ("prominent search bar" on home; search on category).
- Adjust styles (and minimal markup if needed) so search is easy to find;
  keep debounced behaviour and API usage unchanged.

### 3. Search no-results state with personality [FE] - DONE

**1 sp.** When search returns zero categories and zero products,
show a consolidated no-results state: a shrug character or small
SVG illustration, the query echoed back ("Nothing found for
'oat milk'"), a suggestion ("Try a shorter search or browse
categories"), and a "Clear search" action that resets the field.

**Tasks:**
- In `+page.svelte` (home) and `categories/[id]/+page.svelte`,
  add a consolidated no-results block when `isSearching` and
  both lists are empty.
- Optionally expose a `clear()` callback or method from
  `SearchForm.svelte`.
- Respect `prefers-reduced-motion` for any entrance animation.

### 4. Unify card and list styling on product detail page [FE] — DONE

**1 sp.** Use the design system (pr-card and related utilities) for review
cards and purchase history on the product detail page instead of ad-hoc
Tailwind so all card-like surfaces share one visual language.

**Tasks:**
- In products/[id]/+page.svelte: style review cards with pr-card (or
  variant) instead of inline rounded-lg border...; style purchase list
  items with pr-card or the same list pattern used elsewhere so borders,
  background, and hover align with the rest of the app.

### 5. Show ratings as stars in review list cards [FE]

**1 sp.** Product detail (`products/[id]/+page.svelte`) and the manage
reviews list (`manage/reviews/+page.svelte`) show numeric ratings (e.g.
`formatRating(...)/5`). Reuse `StarRating.svelte` for a consistent
at-a-glance display aligned with [spec.md](spec.md) and existing
`pr-rating` styling.

**Tasks:**
- Pass each `review.rating` into `StarRating`; keep an accessible label
  (sr-only or `aria-*`) so the value is not icon-only.
- Tune size and spacing so both surfaces match the rest of the app.

### 6. Product detail page: inline add review [FE]

**1 sp.** In the Reviews section on the product detail page, keep **Add
review** as a link (full `manage/reviews/add` page remains available). Place
the link under the last review or under the empty
state. On click, show the inline form **in that spot**, replacing the link;
cancel or successful submit restores the link. Reuse `POST /api/v1/reviews`;
product is fixed from the current page. Remove **Add review** from the
separate footer/actions block; **Add purchase** may stay there until inline
add purchase is implemented.

**Tasks:**
- Swap link and inline form (rating, optional text); validation and messages
  aligned with manage add review; on success `invalidateAll` (or append) and
  restore the link.
- Optional: share small validation/helpers with `manage/reviews/add` to avoid
  drift.
- Update [spec.md](spec.md) if product page behaviour is specified there.

### 7. API error and request logging [BE]

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

### 8. Product detail page: inline add purchase [FE]

**2 sp.** In the Purchase history section, keep **Add purchase** as a link
(full manage add page remains available). Place the link at the bottom of
the section. Same swap-to-inline pattern as inline add review. Reuse `POST
/api/v1/purchases`; prefill product and default variation from the current
page (e.g. extend `PurchaseForm` or equivalent). Load locations in
`products/[id]` data when needed. Remove **Add purchase** from the footer;
remove the footer/actions block entirely if it becomes empty.

**Tasks:**
- `listLocations()` (or equivalent) in `products/[id]/+page.ts` alongside
  existing loads; handle errors consistently with the rest of the page.
- Inline form: variation, location, quantity, price, date; validation aligned
  with manage add purchase; on success refresh (or append) and restore the
  link.
- Reuse or extend `PurchaseForm` with props for fixed product and variations
  from `GET /api/v1/products/:id` where practical.
- Update [spec.md](spec.md) if not already covered when documenting inline
  review.

### 9. Deprioritize variation label when unit is set on new product [FE]

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

### 10. Typography and modern font stack [FE] — DONE

**3 sp.** Shortlist a few contemporary, readable typefaces (UI body and
headings can differ) — e.g. explore options from Google Fonts, Fontsource,
or other self-hosted/OFL-friendly families — then pick one pairing that fits
the app (legibility, personality, dark/light). Wire fonts into the Tailwind
theme / `layout.css` with sensible fallbacks and loading that avoids jarring
FOUC where practical. After the stack is set, align type scale, line-height,
and heading hierarchy so pages feel consistent (management vs. browse,
form labels vs. body) without a full rebrand.

**Tasks:**
- Review 3–5 modern candidates (mix of geometric, humanist, or neo-grotesk
  as appropriate); compare in context on home, product detail, and a manage
  screen; note license and file size.
- Implement chosen fonts (prefer self-hosted or project-approved CDN);
  define CSS variables or Tailwind `fontFamily` for `sans` (and display if
  used).
- Audit key routes (`+layout.svelte`, home, category, product detail,
  manage shells) for mixed font sizes or ad-hoc classes; unify with the new
  stack.
- Document any new token or pattern in a short comment or spec note if
  behaviour changes visibly.

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
