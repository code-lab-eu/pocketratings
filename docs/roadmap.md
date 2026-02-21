# Pocket Ratings — Roadmap

This document tracks planned features and improvements for Pocket Ratings.

## Authentication & Security

### JWT Token TTL and Refresh Strategy

**Status:** Planned

**Recommendation:**
- **Base expiration:** 30 days (suitable for personal, infrequent use)
- **Sliding expiration:** Extend token expiration by 7 days on each authenticated request if the token expires within 7 days
- **Implementation:**
  - Add `JWT_EXPIRATION_SECONDS` environment variable (default: 2592000 = 30 days)
  - Add `JWT_REFRESH_THRESHOLD_SECONDS` environment variable (default: 604800 = 7 days)
  - Middleware checks token expiration on each request
  - If token expires within refresh threshold, issue a new token with extended expiration
  - Return new token in response header (e.g., `X-New-Token`) for client to update

**Rationale:**
- 30-day base expiration accommodates infrequent personal use
- Sliding window keeps tokens valid during active use without requiring frequent re-login
- Tokens still expire after 30 days of inactivity for security

## Backend Improvements

### API error responses: JSON and frontend 404 handling

**Status:** Planned

**Goal:** Backend returns all errors as JSON (not plain text) so the frontend can parse them. Optionally include an error type in the JSON so the frontend can identify the error and respond appropriately (e.g. show a 404 page when the resource is invalid or not found).

**Current behaviour:** Some endpoints (e.g. `GET /api/v1/categories/invalid-category-id`) may return a plain string body (e.g. "Invalid URL...") instead of JSON. The frontend should treat such cases as "not found" and show a 404 (e.g. "Category not found" or a 404 page).

**Tasks:**
- **Backend:** Ensure every error response has `Content-Type: application/json` and a JSON body matching the documented shape (e.g. `{ "error": "error_code", "message": "Human-readable message" }`).
- **Backend (optional):** Add a stable `error` code (or `type`) in the JSON so the frontend can identify the error (e.g. `not_found`, `validation_error`, `conflict`) and decide how to present it.
- **Frontend:** In cases like invalid category ID or missing resource, show a 404 (e.g. "Not found" message or 404 page) instead of a generic error or broken state.
- Document error response format in [api.md](api.md). Update frontend API client to handle JSON errors and, where appropriate, map them to 404 or other user-facing behaviour.

**Rationale:** JSON errors allow the frontend to parse and display consistent messages; the frontend is responsible for presenting "not found" (404) when the resource is invalid or missing.

### Category list: optional `depth` parameter

**Status:** Planned

**Goal:** Allow callers to request only direct children when listing categories, so the frontend can show one level at a time (e.g. on the category page, child categories only, without descendants).

**Tasks:**
- Add optional query parameter `depth` to `GET /api/v1/categories`.
- When `depth=1` (and optionally `parent_id` is set), return only **direct children** (one level). Without `depth`, keep current behaviour (e.g. return all descendants or flat list per current implementation).
- Document in [api.md](api.md): parameter semantics, examples (`?parent_id=<uuid>&depth=1` for direct children only).

**Rationale:**
- Frontend category page shows child categories above products; it only needs immediate children, not the full subtree.
- Reduces payload size and clarifies API contract when only one level is needed.

### CLI Timestamp Management

**Status:** Planned

**Issue:** CLI commands manually set `updated_at` and `deleted_at` timestamps, but these should be managed automatically like in the REST API.

**Tasks:**
- Update database layer to automatically set `updated_at` on update operations
- Update database layer to automatically set `deleted_at` on soft-delete operations
- Remove manual timestamp setting from CLI commands:
  - `category update` — remove manual `updated_at` setting
  - `category delete` — remove manual `deleted_at` setting
  - `location update` — remove manual `updated_at` setting
  - `location delete` — remove manual `deleted_at` setting
  - `product update` — remove manual `updated_at` setting
  - `product delete` — remove manual `deleted_at` setting
  - `review update` — remove manual `updated_at` setting
  - `review delete` — remove manual `deleted_at` setting
  - `purchase delete` — remove manual `deleted_at` setting
- Ensure `created_at` is automatically set on create operations (verify current behavior)

**Benefits:**
- Consistency between CLI and REST API behavior
- Reduced code duplication
- Less error-prone (no risk of forgetting to set timestamps)

## Frontend

### Home page: live-updating search

**Status:** Planned (post–Phase 3)

**Goal:** Update search results as the user types, without a full page reload.

**Tasks:**
- Keep search on the home page (`/?q=...`).
- As the user types in the search input, update the URL (e.g. via `replaceState` or SvelteKit navigation) and re-run load (or refetch) so that categories and products filter in real time, without requiring the user to submit the form.
- Optionally debounce (e.g. 300 ms) to avoid excessive requests while typing.
- Do not reload the full page; use client-side navigation and data updates.

**Rationale:** Current implementation uses submit-only search for simplicity; this improves UX for in-store lookup.

### Home page: search by category name

**Status:** Planned

**Goal:** When searching on the homepage, include products that are linked to a category whose name matches the search term.

**Tasks:**
- Extend the search logic to query products by category name in addition to product name.
- Backend: Update `GET /api/v1/products` search to include products where the product's category (or any ancestor category) name matches the search term.
- Frontend: No changes needed if backend handles category matching; otherwise update frontend search to also query categories and include their products.

**Rationale:** Users may search by category name (e.g. "wine", "cheese") and expect to see all products in that category, not just products with that word in their name.

### Search engine evaluation

**Status:** Future consideration

**Goal:** Evaluate whether to integrate a more powerful search engine like Typesense for improved search capabilities.

**Considerations:**
- Current search is basic (name matching, possibly category matching).
- Typesense (or similar) would provide:
  - Full-text search with typo tolerance
  - Faceted search (filter by category, location, etc.)
  - Relevance ranking
  - Multi-field search (name, description, category, etc.)
- Trade-offs:
  - Additional infrastructure and complexity
  - Need to keep search index in sync with database
  - May be overkill for personal use case
- Decision point: Evaluate when search requirements grow (e.g. large product catalog, need for advanced filtering, or user feedback indicates search is insufficient).

### Purchases API: include location in response

**Status:** Planned

**Goal:** Allow the frontend to show location name in purchase history without a separate `GET /api/v1/locations` call or client-side resolution by id.

**Tasks:**
- Extend purchase list and purchase detail responses to include location data with each purchase item. Options: (a) nested object `"location": { "id": "uuid", "name": "Store Name" }`, or (b) top-level `"location_name": "Store Name"` (and keep `location_id`).
- Document the response shape in [api.md](api.md).
- Frontend will use this when available; until then the frontend may call `listLocations()` and resolve `location_id` client-side.

**Rationale:** Reduces round-trips and keeps product-detail load simpler when showing purchase history.

### Svelte Web Application

**Status:** Planned

**Primary use case:** In-store decision making — user in a shop (e.g. supermarket) looks up products by category or search and sees clear product ratings at a glance (based on the average rating from all reviews) to decide what to buy. See [spec: Frontend (web app)](spec.md#frontend-web-app) for information architecture, screens, and data flow.

**Scope:**
- **Primary (home):** Category list + prominent search; category → products with an average rating (computed from all reviews); search results with ratings; product detail (reviews, purchase history).
- **Auth:** Login with JWT; token in localStorage; handle `X-New-Token` refresh. Registration remains CLI-only.
- **Management (menu):** Categories CRUD, Locations CRUD, Products CRUD, Purchases, Reviews — all behind a single entry point (e.g. hamburger or "More" menu).

**Technical Requirements:**
- Svelte with TypeScript
- Mobile-first, responsive (single column, thumb-friendly)
- Client-side routing
- API integration with REST endpoints; two-call pattern for list views (products + reviews merged in frontend) so that the frontend can compute per-product average ratings from all reviews, while optionally highlighting the current user's rating separately.
- Token storage and refresh handling
- Form validation, error handling, user feedback
- Dark mode support (optional)

### Product list ratings: global average

**Status:** Planned

**Goal:** Ensure that product lists (on home and category pages) display ratings based on the **average of all reviews from all users**, not a single user’s ratings.

**Tasks:**
- Frontend:
  - When loading product lists, fetch all relevant reviews (not only one user), and compute per-product average ratings client-side.
  - Optionally (future): fetch current-user reviews separately and visually distinguish the current user’s rating (e.g. badge or secondary marker) without overriding the global average.
- Backend (optional future enhancement):
  - Consider adding aggregated fields (e.g. `average_rating`, `review_count`) to product responses or a dedicated aggregate endpoint to avoid recomputing averages on every request.
  - Document any aggregate fields in [api.md](api.md) and update the spec once implemented.

**Rationale:**
- Averages across all users provide a more objective signal for decision making than showing only one user’s ratings.
- Keeping the averaging logic in the frontend initially avoids premature backend complexity, while leaving room for a future API-level optimization.

**Design Considerations:**
- Lookup-first, not data-entry-first; management tucked away in menu
- Fast and lightweight; debounced search; consider caching category list
- Accessibility: semantic HTML, focus order, sufficient contrast

## Future Enhancements

### Frontend (beyond current data models)

**Status:** Future consideration

- **Favorite / pinned categories** — Quick access to e.g. "Wines", "Cheeses" on home. Requires either a frontend-only preference (e.g. pinned category IDs in localStorage) or a backend "favorites" or "order" field. v1 can implement frontend-only pins (localStorage) without backend changes.
- **Recently viewed or recently added products** — "Recently added" or "Last viewed" section on home. Requires either client-side history (e.g. last N product IDs in sessionStorage/localStorage) or backend support (e.g. `last_viewed_at`, `created_at` ordering and limit). v1 can implement frontend-only "recently viewed" (localStorage) without backend changes.
- **Offline / PWA** — Cache categories and recent product/rating data for use without network; sync when back online. Requires service worker and possibly API extensions.
- **Barcode scanning** — See Barcode Scanning below; would drive "look up product by barcode" from home or search.

### Data Export

**Status:** Future consideration

- Export purchases and reviews to CSV/JSON
- Backup/restore functionality

### Analytics & Insights

**Status:** Future consideration

- Spending trends over time
- Most purchased products
- Average ratings by category
- Price tracking over time

### Barcode Scanning

**Status:** Future consideration (mentioned in spec as non-goal for v1)

- Product lookup via barcode
- Requires browser API support detection
- Fallback to manual entry

## Completed

- REST API specification and documentation
- CLI implementation for all entities
- Database schema and migrations
- Authentication (JWT) — basic implementation
- Soft-delete functionality
- Protected fields enforcement in API documentation
- Version endpoint: `GET /api/v1/version` (unauthenticated; returns server version)
- No-unsafe enforcement: `#![forbid(unsafe_code)]` in backend crate roots so the build fails if `unsafe` is introduced
