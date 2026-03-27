# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

---

## Planned

Order: (1) blocking tasks, (2) important, (3) low-hanging fruit (1–2 SP), (4)
rest. Every item has a story point estimate in the first line of its body.

### 1. Order categories alphabetically [FE+BE] -- DONE

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

### 2. Frontend code quality review [FE] -- DONE

**2 sp.** Systematic review of all frontend components and pages for code
smells: duplication, unnecessary complexity, disproportionate code, and
carry-forward debt from earlier iterations. Fix issues found and ensure
adherence to the code-quality-review rule. **Important.**

**Tasks:**
- Review all files in `src/lib/` and `src/routes/` for duplication (blocks
  that differ by one or two values), unnecessary complexity, and
  proportionality (amount of code vs what it does).
- Review test fixtures for clarity: meaningful names, no redundant/confusing
  duplicates.
- Fix all issues found; run full frontend QC after each file.

### 3. Replace emoji with Lucide icons in header menu and BackLink [FE]

**1 sp.** Use lucide-svelte for the header **menu** (hamburger) and for
BackLink (arrow) only. The header **theme toggle** is out of scope: it uses
custom layered SVG and CSS motion (task 11). Do **not** replace or revert
that control when doing this task.

**Tasks:**
- In +layout.svelte: replace ☰ with Menu; leave the theme toggle as
  implemented for task 11.
- In BackLink.svelte: replace "←" with ArrowLeft (or ChevronLeft), size
  appropriately, aria-hidden="true" on the icon so the link label is the
  screen-reader focus.

### 4. Unify card and list styling on product detail page [FE]

**1 sp.** Use the design system (pr-card and related utilities) for review
cards and purchase history on the product detail page instead of ad-hoc
Tailwind so all card-like surfaces share one visual language.

**Tasks:**
- In products/[id]/+page.svelte: style review cards with pr-card (or
  variant) instead of inline rounded-lg border...; style purchase list
  items with pr-card or the same list pattern used elsewhere so borders,
  background, and hover align with the rest of the app.

### 5. Make reusable components route-agnostic [FE]

**1 sp.** `ProductList` hardcodes `/products/[id]` in its href;
`EmptyState` calls `resolve()` internally instead of receiving a
pre-resolved href. Reusable components should have no route
knowledge -- the caller owns route resolution. Follow the pattern
established in `CategoryLinkList` (`hrefFor` callback or
pre-resolved href string). Prefer doing this before adding new
ProductList callsites so href wiring stays in one place per page.

**Tasks:**
- `ProductList`: replace hardcoded
  `resolve('/products/${product.id}')` with a caller-provided
  `hrefFor: (id: string) => string` prop.
- `EmptyState`: accept a pre-resolved `href` string instead of
  calling `resolve()` internally; move `resolve()` to callsites.
- Remove `import { resolve } from '$app/paths'` from both
  components after the change.

### 6. Make the search field more prominent [FE]

**1 sp.** Home and category pages already expose search per [spec.md](spec.md);
improve visual hierarchy so the search input reads as a primary control
(larger tap target, clearer label or placeholder, spacing) without adding a
separate search route.

**Tasks:**
- Audit home and category page layout: compare search prominence to spec
  ("prominent search bar" on home; search on category).
- Adjust styles (and minimal markup if needed) so search is easy to find;
  keep debounced behaviour and API usage unchanged.

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

### 8. Product detail page: inline add review and add purchase [FE]

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

### 9. Draggable star slider rating input [FE] -- DONE

**2 sp.** Replace the plain number input for review rating with a
draggable star slider. A transparent native range input is layered
over 5 SVG stars; stars fill continuously as the user drags. A
floating label above the thumb shows the precise value (e.g. 3.8)
in real time. On release the value snaps to the nearest 0.25.
A subtle pulse animation on release provides tactile feedback.

**Tasks:**
- New component `StarRatingInput.svelte`: range input (min 1,
  max 5, step 0.25) layered over SVG star row; floating numeric
  label tracks thumb position; pulse keyframe on release.
- Swap number input for `StarRatingInput` in
  `manage/reviews/add/+page.svelte` and
  `manage/reviews/[id]/+page.svelte`.
- Add pulse keyframe and slider thumb styling to `layout.css`.
- Respect `prefers-reduced-motion` (skip pulse, keep instant
  fill).

### 10. Playful empty states with contextual icons [FE] — DONE

**2 sp.** Enhance `EmptyState.svelte` with contextual icons above the
message (Lucide) and friendlier copy that encourages action.
Icons use `currentColor` with per-icon accent colors. A subtle
fade-in + slide-up entrance animation plays on mount.

**Tasks:**
- Add optional `icon` prop to `EmptyState.svelte`; add entrance
  animation (fade + translateY).
- Update all `EmptyState` callsites with contextual icon choice
  and friendlier copy.
- Respect `prefers-reduced-motion`.

### 11. Animated sun/moon theme toggle [FE] — DONE

**1 sp.** Replace the static Unicode sun/moon characters in the
header theme toggle with SVG icons that morph and rotate into
each other on toggle (~400 ms, CSS-only). Sun rays retract while
a crescent slides in; reverse for moon-to-sun.

**Tasks:**
- In `+layout.svelte`, use layered SVGs (sun and moon) via
  `ThemeToggleIcon`; toggle visibility, ray scale, slide, and
  wrapper tilt via CSS transitions (~400 ms).
- Add reduced-motion overrides for the theme toggle in
  `layout.css`.

### 12. Search no-results state with personality [FE]

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

### 13 After adding a new category, redirect to it

### 14 Add a favicon

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
