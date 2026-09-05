---
name: api-documentation
description: How to document a REST endpoint in docs/api.md and docs/api.http. Use when adding or changing a route under backend/src/api/, or when editing docs/api.md or docs/api.http.
---

# API endpoint documentation

When you add or change a REST endpoint that is not yet described in the
project docs, update both files.

## docs/api.md

Add or update the endpoint in the correct section (Auth, Categories, etc.):

- Path and method (e.g. `GET /api/v1/me`)
- Short description, auth requirement
- Request body (if any), query params (if any)
- Response: status and JSON example
- Errors: list of status codes and when they occur

## docs/api.http

Add or update a runnable example:

- Comment with path and purpose
- Request with method, URL, and headers (e.g.
  `Authorization: Bearer {{token}}` for protected routes)
- Body for POST/PATCH if applicable

Use the same style as the existing endpoints in those files.
