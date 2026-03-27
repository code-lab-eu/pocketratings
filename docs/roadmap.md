# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Planned

Order: (1) blocking tasks, (2) important, (3) low-hanging fruit (1–2 SP), (4)
rest. Every item has a story point estimate in the first line of its body.

### 1. Make reusable components route-agnostic [FE] — DONE

**1 sp.** `ProductList` hardcodes `/products/[id]` in its href;
`EmptyState` calls `resolve()` internally instead of receiving a
pre-resolved href. Reusable components should have no route
knowledge — the caller owns route resolution. Follow the pattern
established in `CategoryLinkList` (`hrefFor` callback or
pre-resolved href string). Prefer doing this before adding new
`ProductList` callsites so href wiring stays in one place per page.

**Tasks:**
- `ProductList`: replace hardcoded
  `resolve('/products/${product.id}')` with a caller-provided
  `hrefFor: (id: string) => string` prop.
- `EmptyState`: accept a pre-resolved `href` string instead of
  calling `resolve()` internally; move `resolve()` to callsites.
- Remove `import { resolve } from '$app/paths'` from both
  components after the change.

### 2. Replace emoji with Lucide icons in header menu and BackLink [FE] — DONE

**1 sp.** Use lucide-svelte for the header **menu** (hamburger) and for
BackLink (arrow) only. The header **theme toggle** is out of scope: it uses
custom layered SVG (`ThemeToggleIcon`) and CSS motion. Do **not** replace or
revert that control when doing this task.

**Tasks:**
- In +layout.svelte: replace ☰ with Menu; leave the theme toggle unchanged.
- In BackLink.svelte: replace "←" with ArrowLeft (or ChevronLeft), size
  appropriately, aria-hidden="true" on the icon so the link label is the
  screen-reader focus.

### 3. Unify card and list styling on product detail page [FE]

**1 sp.** Use the design system (pr-card and related utilities) for review
cards and purchase history on the product detail page instead of ad-hoc
Tailwind so all card-like surfaces share one visual language.

**Tasks:**
- In products/[id]/+page.svelte: style review cards with pr-card (or
  variant) instead of inline rounded-lg border...; style purchase list
  items with pr-card or the same list pattern used elsewhere so borders,
  background, and hover align with the rest of the app.

### 4. Make the search field more prominent [FE]

**1 sp.** Home and category pages already expose search per [spec.md](spec.md);
improve visual hierarchy so the search input reads as a primary control
(larger tap target, clearer label or placeholder, spacing) without adding a
separate search route.

**Tasks:**
- Audit home and category page layout: compare search prominence to spec
  ("prominent search bar" on home; search on category).
- Adjust styles (and minimal markup if needed) so search is easy to find;
  keep debounced behaviour and API usage unchanged.

### 5. Search no-results state with personality [FE]

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

### 6. After creating a category, redirect to its public page [FE]

**1 sp.** Today `manage/categories/new` returns to the category list after
success. Navigate to `/categories/:id` (using the `id` from
`createCategory`'s response) so the user lands on the new category in
context; optionally still invalidate data so manage lists stay fresh.

**Tasks:**
- In `manage/categories/new/+page.svelte`: after successful
  `createCategory`, `goto` the public category URL with
  `resolve('/categories/[id]', { id })` (or equivalent).
- Add or extend a test if behaviour is covered by component or e2e tests.
- Note in [spec.md](spec.md) if management flows doc should mention the
  redirect.

### 7. Add a favicon [FE]

**1 sp.** Ship a tab icon: `frontend/src/lib/assets/favicon.svg` exists but
is not wired in the document head. Link it via SvelteKit convention (`<link
rel="icon" ...>` in root layout or static favicon) so browsers show the app
icon.

**Tasks:**
- Add favicon link (and optional `apple-touch-icon` if desired) consistent
  with SvelteKit 2 / project static asset paths.
- Verify in dev and build output.

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

### 9. Typography and modern font stack [FE]

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

### 10. Product detail page: inline add review [FE]

**1 sp.** In the Reviews section on the product detail page, keep **Add
review** as a link (full `manage/reviews/add` page remains available). Place
the link under the last review or under the empty
state. On click, show the inline form **in that spot**, replacing the link;
cancel or successful submit restores the link. Reuse `POST /api/v1/reviews`;
product is fixed from the current page. Remove **Add review** from the
separate footer/actions block; **Add purchase** may stay there until task
11.

**Tasks:**
- Swap link and inline form (rating, optional text); validation and messages
  aligned with manage add review; on success `invalidateAll` (or append) and
  restore the link.
- Optional: share small validation/helpers with `manage/reviews/add` to avoid
  drift.
- Update [spec.md](spec.md) if product page behaviour is specified there.

### 11. Product detail page: inline add purchase [FE]

**2 sp.** In the Purchase history section, keep **Add purchase** as a link
(full manage add page remains available). Place the link at the bottom of
the section. Same swap-to-inline pattern as task 10. Reuse `POST
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
- Update [spec.md](spec.md) if not already done in task 10.

### 12. Show ratings in both review cards as stars

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
