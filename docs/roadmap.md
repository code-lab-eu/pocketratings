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

### Svelte Web Application

**Status:** Planned

**Primary use case:** In-store decision making — user in a shop (e.g. supermarket) looks up products by category or search and sees their own ratings at a glance to decide what to buy. See [spec: Frontend (web app)](spec.md#frontend-web-app) for information architecture, screens, and data flow.

**Scope:**
- **Primary (home):** Category list + prominent search; category → products with my rating; search results with ratings; product detail (reviews, purchase history).
- **Auth:** Login with JWT; token in localStorage; handle `X-New-Token` refresh. Registration remains CLI-only.
- **Management (menu):** Categories CRUD, Locations CRUD, Products CRUD, Purchases, Reviews — all behind a single entry point (e.g. hamburger or "More" menu).

**Technical Requirements:**
- Svelte with TypeScript
- Mobile-first, responsive (single column, thumb-friendly)
- Client-side routing
- API integration with REST endpoints; two-call pattern for list views (products + my reviews merged in frontend)
- Token storage and refresh handling
- Form validation, error handling, user feedback
- Dark mode support (optional)

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
