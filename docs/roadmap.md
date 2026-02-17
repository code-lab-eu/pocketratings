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

### Nuxt 4 Web Application

**Status:** Planned

**Scope:**
- User authentication (login with JWT token)
- Dashboard/homepage showing recent purchases and reviews
- Category management (create, list, update, delete)
- Location management (create, list, update, delete)
- Product management (create, list, update, delete, search)
- Purchase recording and management (create, list, filter by date/product/location)
- Review management (create, list, update, delete)
- Product detail pages showing purchase history and reviews

**Technical Requirements:**
- Nuxt 4 with TypeScript
- Responsive design (mobile-friendly)
- Dark mode support (optional)
- Client-side routing
- API integration with REST endpoints
- Token storage and management (localStorage or httpOnly cookies)
- Form validation
- Error handling and user feedback

**Design Considerations:**
- Simple, clean UI suitable for personal/family use
- Fast and lightweight
- Works offline (optional, future enhancement)

## Future Enhancements

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
