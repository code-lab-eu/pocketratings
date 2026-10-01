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

### 2. Product detail page: inline add purchase [FE] — DONE

**2 sp.** In the Purchase history section, **Add purchase** is a button that
opens the purchase form in place; the product page no longer links to the manage
add page, which stays available from Manage. Place the button at the bottom of
the section. Same swap-to-inline pattern as inline add review. Reuse
`POST /api/v1/purchases`; prefill product and default variation from the current
page (e.g. extend `PurchaseForm.svelte` or equivalent). Load locations in
`products/[id]` data when needed. Remove **Add purchase** from the footer;
remove the footer/actions block entirely if it becomes empty.

**Tasks:**
- `listLocations()` in `products/[id]/+page.ts` alongside existing loads;
  handle errors consistently with the rest of the page.
- Inline form: variation, location, quantity, price, date; validation aligned
  with manage add purchase; on success refresh (or append) and restore the
  button.
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

### 6. CLI command to generate a password reset link [FE+BE] — DONE

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

### 7. Update to Node 26 [FE] — DONE

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

### 8. Default purchase date uses UTC instead of local time [FE]

**1 sp.** `PurchaseForm.svelte` prefills the **Date** field with
`new Date().toISOString().slice(0, 16)`, which is the current time in UTC. A
`datetime-local` input expects local time, so outside UTC the default is off by
the UTC offset (e.g. 3 hours in the past at UTC+3). This affects the manage add
purchase page and the inline add purchase form on the product page. The edit
page has the same issue when it converts `purchased_at` to the field value.

**Tasks:**
- Add a formatter in `lib/utils/formatters.ts` that turns a `Date` into a local
  `YYYY-MM-DDTHH:mm` string, with a unit test that pins the timezone (e.g. `TZ`
  in the test) and checks a non-UTC offset.
- Use it for the default date in `PurchaseForm.svelte` and for the initial value
  in `manage/purchases/[id]/+page.svelte`.
- Submitting still converts the field value with `new Date(value)`, which parses
  it as local time; confirm the round trip in a test.

### 9. Align icon-style buttons on one component [FE]

**2 sp.** Five buttons are written by hand with their own classes and ARIA
attributes: the theme toggle and **Log out** in `routes/+layout.svelte`, the
show/hide toggle in `PasswordField.svelte`, the delete icon in
`ManageListRow.svelte`, and the expand/collapse chevron in
`CategoryLinkList.svelte`. The first four use `pr-btn-icon` with differing extra
classes; the chevron uses neither `pr-btn-icon` nor the 44px tap target the
others have.

**Tasks:**
- Propose the approach before implementing: an `icon` variant on `Button` or a
  separate `IconButton` component. Either needs an accessible name
  (`aria-label`), optional `aria-pressed` and `title`, and the 44px minimum tap
  target.
- Move all five buttons to the agreed component in the same change; decide
  whether **Log out** (text, not an icon) belongs in it or uses
  `variant="link"`.
- Keep existing tests passing (`ManageListRow`, `password-field`,
  `category-link-list`, layout) and add coverage where a button has none.
  Document the result in `frontend/STYLES.md`.

### 10. Edit and delete icons on purchases and reviews on the product page [FE]

**4 sp.** On the product detail page, the Purchase history and Reviews sections
list every family member's purchases and reviews of the product, but offer no
way to change or remove one. Add the same edit (pencil) and delete (trash) icons
that each row has on the management pages (`/manage/purchases` and
`/manage/reviews`, both using `ManageListRow.svelte`): edit links to
`/manage/purchases/[id]` or `/manage/reviews/[id]`; delete asks "Delete this
purchase?" or "Delete this review?", calls `DELETE /api/v1/purchases/:id` or
`DELETE /api/v1/reviews/:id`, and refreshes the page data.

**Tasks:**
- Show the icons only on the current user's purchases and reviews. The API
  returns `403` when editing or deleting another user's purchase or review.
- Return to the product page after editing, with one mechanism for both edit
  pages; agree how they learn where to return to (e.g. a query parameter) before
  implementing. The purchase edit page currently goes to `/manage/purchases`
  after Save, Cancel, and from its back link; the review edit page already goes
  to the product page after Save, but Cancel and its back link go to
  `/manage/reviews`.
- Reuse the icons from `ManageListRow` rather than copying its markup: propose
  how to share them (e.g. extract the edit and delete icons into a component
  used by `ManageListRow` and both product page lists) before implementing.
  Coordinate with task 9, which moves the `ManageListRow` delete icon to a
  shared icon button.
- Tests first, for both lists: icons present only on own items, edit link
  targets, delete with confirm (confirmed and cancelled), and the return
  navigation from both edit pages. Update [spec.md](spec.md).

### 11. Overhaul docs/api.http: readable, runnable, no duplicated reference [BE]

**4 sp.** `docs/api.http` is meant to hold runnable examples, but it is hard to
read, it cannot be run without hand-editing, several examples cannot be run at
all, and it repeats reference material from `docs/api.md` that has drifted and
is now wrong.

**Tasks:**
- Replace the comment format. Today each request has one long line that puts the
  HTTP method, the URI, the body fields, and a short description inline. Adopt a
  structured, easy to read format instead (e.g. a title per request and short
  separate lines for its purpose and notable responses), and do not repeat what
  the request line and JSON body already show. Agree the format before
  implementing.
- Use REST Client request variables so the file runs top to bottom: name the
  login request and read the token from its response instead of copying it into
  `@token`, and take ids from the create responses instead of the all-zero
  placeholder ids. Define `variationId`, which is used but never defined.
- Uncomment the 11 commented-out requests, or state why one stays commented out
  (e.g. hard deletes). `PATCH` and `DELETE /api/v1/variations/:id` currently
  have no runnable example.
- Remove reference material that belongs in `docs/api.md`: the status code list
  (it says 200 for DELETE, which is 204, and mentions PUT, which does not exist)
  and the protected fields notes repeated in nine comments.
- Fix the examples that contradict `docs/api.md`: `GET /api/v1/reviews` without
  parameters returns all users' reviews, not "my reviews"; `?q=` also searches
  category names; purchase responses also nest `variation`; the create purchase
  example omits `variation_id`.
- Use one separator style (`###`, blank line, comment) and remove the empty
  request block created by `###` followed by `### Auth`.
- Record the conventions in the api-documentation skill.

### 12. Show edit and delete actions as icons everywhere [FE]

**3 sp.** The manage lists (e.g. `/manage/products`) show edit and delete as
pencil and trash icons (`ManageListRow.svelte`, with `EditLink.svelte` for
edit). Elsewhere they are text buttons: the Variations list on the manage
product page (`manage/products/[id]/+page.svelte`) has an **Edit** button (a
secondary `Button` that opens the inline edit form) and a **Delete** button per
variation, and the location, category, and product edit pages
(`manage/locations/[id]`, `manage/categories/[id]`, `manage/products/[id]`) have
a **Delete** button next to **Save** and **Cancel**. The four delete buttons are
raw `<button>` elements with `pr-btn-danger`; the product and variation delete
buttons are disabled with a tooltip (`title`) when deletion is not allowed. Show
all of these as icons.

**Tasks:**
- Do this after task 9, which adds the shared icon button. The Variations
  **Edit** opens a form on the same page, so it is an icon button rather than
  `EditLink`; the delete icons are icon buttons too, styled like the delete icon
  in `ManageListRow`.
- Replace the Variations **Edit** and **Delete** buttons and the three edit-page
  **Delete** buttons in the same change. Keep the disabled state and tooltip.
- Remove the `.pr-btn-danger` class from `frontend/src/routes/layout.css`; these
  four buttons are its only users.
- Tests first: accessible names (e.g. "Edit {variation}", "Delete {variation}"),
  the edit icon opens the inline form, delete asks for confirmation, and a
  disabled delete keeps its tooltip; the manage product page has no tests for
  these yet. Update [spec.md](spec.md) and `frontend/STYLES.md`.

### 13. Use PageHeading for every page title [FE]

**1 sp.** `frontend/STYLES.md` says page titles use the shared `PageHeading`
component (`lib/PageHeading.svelte`) instead of a hand-written `<h1>`, and 17
pages do. The login page (`routes/login/+page.svelte`), the category page
(`routes/categories/[id]/+page.svelte`), and the product detail page
(`routes/products/[id]/+page.svelte`) still write `<h1 class="pr-heading-page">`
by hand.

**Tasks:**
- Switch all three to `PageHeading` in one change, passing their extra classes
  through its `class` prop (`mb-6` on the login page; `min-w-0 break-words` on
  the product page, where the heading sits next to the edit link). No visible
  change.
- Keep the existing login, category, and product page tests passing.

### 14. List the markdown wrap check in AGENTS.md [FE+BE]

**1 sp.** Pre-push and CI run `./scripts/markdown-wrap.py check` next to
`./scripts/ascii-punctuation.sh check`, but AGENTS.md mentions only the
punctuation check, in its "Completion gate" and "Markdown and punctuation"
sections. An agent therefore does not run the wrap check before calling work
done and only finds wrapping mistakes at push time.

**Tasks:**
- Add the wrap check to both sections, next to the punctuation check, with the
  command that checks the paragraphs changed on the branch:
  `./scripts/markdown-wrap.py check --since "$(git merge-base origin/master HEAD)"`.

### 15. Document 401 for a missing or invalid token [BE]

**1 sp.** For a missing, invalid, or expired token the backend returns
`401 Unauthorized` (`backend/src/api/auth/middleware.rs`), and the error code
table in `docs/api.md` says so, but other places say `403 Forbidden`. `403` is
only correct when an authenticated user is not allowed to do something, such as
editing another user's review.

**Tasks:**
- `docs/api.md`: the "Unauthenticated access" rule and the errors of
  `GET /api/v1/me` say 403; the HTTP status code list puts a missing or invalid
  token and an authorization failure under one 403 entry. Change them to 401,
  and give 401 its own entry in the status code list.
- `docs/spec.md`: the Login flow says "all others return 403 if not
  authenticated" and calls login the only unauthenticated endpoint, but the
  password reset endpoints and `GET /api/v1/version` are unauthenticated too;
  the backend crates table says the tower middleware "returns 403". Correct
  both.
- `docs/api.http`: the HTTP status code list in its header puts a missing or
  invalid token and an authorization failure under one 403 entry. Fix it the
  same way as the `docs/api.md` status code list.

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
