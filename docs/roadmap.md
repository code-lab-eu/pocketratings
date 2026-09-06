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
  the [error-messages](../.agents/skills/error-messages/SKILL.md) skill.
- Encode "show product `Select` iff no product id" as a clear prop (or
  equivalent); keep edit and inline add flows on fixed product id without a
  dropdown.
- Preserve existing transitions and `pr-inline-form` usage on the product
  page; no behaviour regression for manage add/edit.
- Extend or add component tests where behaviour is non-trivial; run frontend
  QC.

### 5. CLI command to change a user's password [BE] — DONE

**2 sp.** Add a CLI command to change a user's password so an administrator
can reset credentials without direct database access. Reuse the existing
password hashing used by the auth flow; identify the target user (e.g. by
username/email) and update the stored hash.

**Tasks:**
- Add a subcommand to the CLI that takes the target user and a new password
  (prompt for the password rather than passing it as a plain argument where
  practical); validate the user exists.
- Hash with the same mechanism as registration/login and persist via the `db`
  user module; report success or a clear error if the user is not found.
- Add a test covering the update path; document the command in README or dev
  docs.

### 6. CLI command to generate a password reset link [FE+BE]

**4 sp.** Add a CLI command that generates a password reset link for a user.
The operator sends the link to the user over a secure channel (e.g. email);
the user opens it and sets a new password without the operator ever learning
it. The link carries a one-time token that is valid for 12 hours and is
invalidated the moment it is used.

**Tasks:**
- **DB:** migration for a password reset token table (user id, token hash,
  expiry, used/consumed marker). Store only a hash of the token, never the
  raw value. Add a `db` module with create, look-up-by-token, and consume
  operations; expired or already used tokens must not resolve.
- **CLI:** subcommand under the existing user commands that takes the target
  user (email, as `set_password` does), creates a token, and prints the full
  reset URL to stdout. Base URL comes from configuration (see the
  **env-configuration** skill) so the printed link matches the deployment.
  Fail with a clear error if the user does not exist.
- **API:** unauthenticated endpoints to validate a token and to set a new
  password with it. Reuse `auth::password::hash_password` and
  `db::user::update_password`; consume the token in the same transaction as
  the password update so it cannot be replayed. Return a generic error for
  invalid, expired, or used tokens (do not distinguish them). Document in
  [api.md](api.md) and [api.http](api.http) per the **api-documentation**
  skill.
- **FE:** a reset route that takes the token from the URL, validates it on
  load, and shows a form with **New password** and **Confirm password**. Both
  fields get a show/hide toggle button (accessible name reflecting state,
  `aria-pressed` or equivalent). Client-side validation: both filled and
  matching. On success, redirect to `login` with a confirmation; on an
  invalid or expired token, show a message telling the user to request a new
  link. No navigation entry - the route is only reachable via the link.
- Tests first at every layer: db token lifecycle (create, expire, consume),
  CLI command, API endpoints (valid, expired, reused token), and the reset
  page (validation, mismatch, toggle). Update [spec.md](spec.md) with the new
  screen and flow.

### 7. Update to Node 26 [FE]

**1 sp.** The frontend currently pins Node.js 24 LTS. Move to Node 26 so
development, CI, and the documented prerequisites all target the same,
current LTS release.

**Tasks:**
- Bump `requiredMajor` in `frontend/scripts/check-node-version.cjs` from 24
  to 26 and update the message it prints.
- Set `node-version: "26"` in the frontend job of
  [ci.yml](../.github/workflows/ci.yml).
- Update `@types/node` in `frontend/package.json` to the matching major and
  refresh `bun.lock`.
- Update the **Node.js** prerequisite in [README.md](../README.md).
- Run frontend QC on Node 26 to confirm lint and tests pass.

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
