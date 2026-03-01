# Pocket Ratings API Documentation

REST over HTTP/JSON. **Base path**: `/api/v1/`. All endpoints live under this path so the rest of the URI space stays
free for the frontend.

## Authentication

**Token-based (JWT)**: The API uses JWT tokens for authentication. Clients authenticate by sending a token in the
`Authorization` header.

- **Login**: `POST /api/v1/auth/login` returns a JWT token
- **Protected endpoints**: Include `Authorization: Bearer <token>` header in all requests
- **Stateless**: No session store needed; token is signed with `JWT_SECRET` environment variable
- **Unauthenticated access**: Only `POST /api/v1/auth/login` and `GET /api/v1/version` are unauthenticated. All other
  endpoints return `403 Forbidden` if authentication is missing or invalid
- **Registration**: In v1, user registration is **CLI-only** (no `POST /api/v1/auth/register` endpoint)

### Token Expiration and Refresh

- **Base expiration**: Tokens expire after 30 days (configurable via `JWT_EXPIRATION_SECONDS` environment variable)
- **Sliding expiration**: Tokens are automatically refreshed during active use:
  - On each authenticated request, if the token expires within 7 days (configurable via `JWT_REFRESH_THRESHOLD_SECONDS`),
    a new token is issued with extended expiration
  - The new token is returned in the `X-New-Token` response header
  - Clients should update their stored token when this header is present
- **Token refresh**: If a token expires, clients must re-authenticate via `POST /api/v1/auth/login`

## Best Practices

### HTTP Status Codes

- **200 OK**: Successful GET, PUT, PATCH, DELETE requests
- **201 Created**: Successful POST request that creates a resource
- **400 Bad Request**: Validation errors, malformed requests
- **403 Forbidden**: Missing or invalid authentication token, or authorization failure (e.g., editing another user's
  review)
- **404 Not Found**: Resource not found (e.g., invalid UUID in path)
- **405 Method Not Allowed**: Unsupported HTTP method for an endpoint (e.g., POST on a read-only endpoint)
- **409 Conflict**: Business rule violations (e.g., deleting a category that has products, deleting a location that has
  purchases)
- **422 Unprocessable Entity**: Semantic validation errors (optional refinement for complex validation failures)
- **500 Internal Server Error**: Server errors

### Protected Fields

The following fields cannot be set or modified by clients and are managed automatically by the server:

- **`id`**: Primary key (UUID), set automatically on creation
- **`created_at`**: Timestamp set automatically on creation
- **`updated_at`**: Timestamp set automatically on create and update operations
- **`deleted_at`**: Managed by soft-delete operations only; cannot be set directly

Attempts to include these fields in request bodies will be ignored or rejected.

### Error Responses

All error responses (4xx/5xx) return JSON with the following format:

```json
{
  "error": "error_code",
  "message": "Human-readable error message (optional)"
}
```

The `error` field mirrors the HTTP status category. The `message` field provides a human-readable description when applicable.

| HTTP status | `error` value | Meaning |
|-------------|---------------|---------|
| 400 | `bad_request` | Validation error or malformed request |
| 401 | `unauthorized` | Missing, invalid, or expired authentication token; client should clear stored token and redirect to login |
| 403 | `forbidden` | Authenticated but not allowed to access this resource |
| 404 | `not_found` | Resource not found |
| 409 | `conflict` | Business rule violation (e.g. delete category that has products) |
| 500 | `internal_server_error` | Server error |

Clients should branch on the HTTP status code. The `message` field is informational and should not be used for control flow.

### Query Parameters

- IDs in path and query parameters are UUIDs
- Dates in query parameters and request bodies use ISO 8601 format
- Monetary amounts are decimal numbers (e.g., `"2.99"` or `2.99` for EUR)
- List endpoints exclude soft-deleted records unless explicitly requested via query parameters

### Soft Deletes

- By default, `DELETE` operations perform soft-deletes (set `deleted_at` timestamp)
- Use `?force=true` query parameter for hard deletes (permanent removal)
- Soft-deleted records are excluded from list endpoints unless explicitly included

## Endpoints

### Version

#### `GET /api/v1/version`

Returns the current API (server) version. No authentication required.

**Response:** `200 OK`
```json
{
  "version": "0.1.0"
}
```

---

### Auth

#### `POST /api/v1/auth/login`

Authenticate and receive a JWT token.

**Request body:**
```json
{
  "email": "user@example.com",
  "password": "password"
}
```

**Response:** `200 OK`
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Errors:**
- `400 Bad Request`: Invalid request body
- `401 Unauthorized`: Invalid email or password

#### `GET /api/v1/me`

Returns the current authenticated user's id and name (e.g. for display in the frontend). Requires a valid Bearer token.

**Response:** `200 OK`
```json
{
  "user_id": "uuid",
  "name": "Alice"
}
```

**Errors:**
- `403 Forbidden`: Missing or invalid authorization token
- `404 Not Found`: User not found (e.g. deleted)

---

### Categories

#### `GET /api/v1/categories`

List categories as a **nested tree**. Each category object includes a `children` array of the same shape (nested categories). Top-level array: root categories when `parent_id` is omitted, or direct children of the given parent when `parent_id` is set.

**Query parameters:**
- `parent_id` (optional, UUID): When set, the top-level array is the direct children of this category. Omit for the full tree (roots at top level).
- `depth` (optional, integer): When `1`, return only one level (roots when no `parent_id`, or direct children of `parent_id`); each item has an empty `children` array. Omit for full tree depth.

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "parent_id": null,
    "name": "Groceries",
    "created_at": 1708012800,
    "updated_at": 1708012800,
    "deleted_at": null,
    "children": [
      {
        "id": "uuid",
        "parent_id": "uuid",
        "name": "Fruit",
        "created_at": 1708012800,
        "updated_at": 1708012800,
        "deleted_at": null,
        "children": []
      }
    ]
  }
]
```

#### `GET /api/v1/categories/:id`

Get a single category by ID.

**Response:** `200 OK` (same format as list item)

**Errors:**
- `404 Not Found`: Category not found

#### `POST /api/v1/categories`

Create a new category.

**Request body:**
```json
{
  "name": "Groceries",
  "parent_id": "uuid" | null
}
```

**Constraints:**
- `name` is required
- `parent_id` is optional
- Category name must be unique per parent

**Response:** `201 Created` (category object)

**Errors:**
- `400 Bad Request`: Validation error (e.g., duplicate name under same parent)
- `404 Not Found`: Parent category not found (if `parent_id` provided)

#### `PATCH /api/v1/categories/:id`

Update a category.

**Request body:**
```json
{
  "name": "Groceries (renamed)",
  "parent_id": "uuid" | null
}
```

All fields are optional. Only provided fields are updated.

**Response:** `200 OK` (updated category object)

**Errors:**
- `400 Bad Request`: Validation error
- `404 Not Found`: Category not found
- `404 Not Found`: Parent category not found (if `parent_id` provided)

#### `DELETE /api/v1/categories/:id`

Soft-delete a category.

**Query parameters:**
- `force` (optional, boolean): If `true`, perform hard delete

**Response:** `204 No Content` (or `200 OK` with empty body)

**Errors:**
- `404 Not Found`: Category not found
- `409 Conflict`: Category has child categories or products (cannot be deleted)

---

### Locations

#### `GET /api/v1/locations`

List all locations.

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "name": "Local supermarket",
    "deleted_at": null
  }
]
```

#### `GET /api/v1/locations/:id`

Get a single location by ID.

**Response:** `200 OK` (same format as list item)

**Errors:**
- `404 Not Found`: Location not found

#### `POST /api/v1/locations`

Create a new location.

**Request body:**
```json
{
  "name": "Local supermarket"
}
```

**Response:** `201 Created` (location object)

**Errors:**
- `400 Bad Request`: Validation error

#### `PATCH /api/v1/locations/:id`

Update a location.

**Request body:**
```json
{
  "name": "Corner store"
}
```

**Response:** `200 OK` (updated location object)

**Errors:**
- `400 Bad Request`: Validation error
- `404 Not Found`: Location not found

#### `DELETE /api/v1/locations/:id`

Soft-delete a location.

**Query parameters:**
- `force` (optional, boolean): If `true`, perform hard delete

**Response:** `204 No Content` (or `200 OK` with empty body)

**Errors:**
- `404 Not Found`: Location not found
- `409 Conflict`: Location has purchases (cannot be deleted)

---

### Products

#### `GET /api/v1/products`

List products.

**Query parameters:**
- `category_id` (optional, UUID): Filter by category
- `q` (optional, string): Search by name or brand

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "category_id": "uuid",
    "brand": "Dairy Co",
    "name": "Organic milk",
    "created_at": 1708012800,
    "updated_at": 1708012800,
    "deleted_at": null
  }
]
```

#### `GET /api/v1/products/:id`

Get a single product by ID.

**Response:** `200 OK` (product object, optionally including purchase/review counts or recent items)

**Errors:**
- `404 Not Found`: Product not found

#### `POST /api/v1/products`

Create a new product.

**Request body:**
```json
{
  "name": "Organic milk",
  "brand": "Dairy Co",
  "category_id": "uuid"
}
```

**Response:** `201 Created` (product object)

**Errors:**
- `400 Bad Request`: Validation error
- `404 Not Found`: Category not found

#### `PATCH /api/v1/products/:id`

Update a product.

**Request body:**
```json
{
  "name": "Organic milk 1L",
  "brand": "Dairy Co",
  "category_id": "uuid"
}
```

All fields are optional. Only provided fields are updated.

**Response:** `200 OK` (updated product object)

**Errors:**
- `400 Bad Request`: Validation error
- `404 Not Found`: Product not found
- `404 Not Found`: Category not found (if `category_id` provided)

#### `DELETE /api/v1/products/:id`

Soft-delete a product.

**Query parameters:**
- `force` (optional, boolean): If `true`, perform hard delete

**Response:** `204 No Content` (or `200 OK` with empty body)

**Errors:**
- `404 Not Found`: Product not found
- `409 Conflict`: Product has purchases (cannot be deleted)

---

### Purchases

#### `GET /api/v1/purchases`

List purchases.

**Query parameters:**
- `user_id` (optional, UUID): Filter by user (default: current user)
- `product_id` (optional, UUID): Filter by product
- `location_id` (optional, UUID): Filter by location
- `from` (optional, ISO 8601 date): Start date
- `to` (optional, ISO 8601 date): End date

**Response:** `200 OK` — Array of purchases. Each item has the same shape as the example below (nested `user`, `product`, `location`). When there are no matching purchases (e.g. the product exists but has no purchases), the response is `200 OK` with body `[]`.

```json
[
  {
    "id": "uuid",
    "user": { "id": "uuid", "name": "Alice" },
    "product": { "id": "uuid", "brand": "Brugge", "name": "Belegen" },
    "location": { "id": "uuid", "name": "Carrefour" },
    "quantity": 1,
    "price": "2.99",
    "purchased_at": 1708012800,
    "deleted_at": null
  }
]
```

#### `GET /api/v1/purchases/:id`

Get a single purchase by ID.

**Response:** `200 OK` — Purchase object with the same shape as list (nested `user`, `product`, `location`).

**Errors:**
- `404 Not Found`: Purchase not found

#### `POST /api/v1/purchases`

Create a new purchase.

**Request body:**
```json
{
  "product_id": "uuid",
  "location_id": "uuid",
  "quantity": 1,
  "price": "2.99",
  "purchased_at": "2025-02-15T12:00:00Z"
}
```

**Constraints:**
- `product_id` and `location_id` are required
- `quantity` defaults to 1 if not provided
- `purchased_at` defaults to current time if not provided
- `user_id` is automatically set to the current authenticated user

**Response:** `201 Created` — Purchase object with the same shape as list (nested `user`, `product`, `location`).

**Errors:**
- `400 Bad Request`: Validation error
- `404 Not Found`: Product or location not found

#### `PATCH /api/v1/purchases/:id`

Update a purchase. Only the owner (purchase's `user_id` equals current user) may update; otherwise 403.

**Request body:**
```json
{
  "product_id": "uuid",
  "location_id": "uuid",
  "quantity": 2,
  "price": "3.49",
  "purchased_at": "2025-02-15T12:00:00Z"
}
```

All fields are optional. Only provided fields are updated.

**Response:** `200 OK` — Updated purchase object with the same shape as list (nested `user`, `product`, `location`).

**Errors:**
- `400 Bad Request`: Validation error (e.g. quantity < 1, negative price)
- `403 Forbidden`: Purchase belongs to another user
- `404 Not Found`: Purchase not found, or product/location not found (if provided)

#### `DELETE /api/v1/purchases/:id`

Soft-delete a purchase.

**Query parameters:**
- `force` (optional, boolean): If `true`, perform hard delete

**Response:** `204 No Content` (or `200 OK` with empty body)

**Errors:**
- `404 Not Found`: Purchase not found

---

### Reviews

List and detail responses include nested `product: { id, brand, name }` and
`user: { id, name }`. The review list is cached in memory and invalidated on
any review insert, update, or delete.

#### `GET /api/v1/reviews`

List reviews.

**Query parameters:**
- `product_id` (optional, UUID): Filter by product
- `user_id` (optional, UUID): Filter by user

**Response:** `200 OK` — Array of reviews. When there are no matching reviews
(e.g. the product exists but has no reviews), the response is `200 OK` with
body `[]`.

```json
[
  {
    "id": "uuid",
    "product": { "id": "uuid", "brand": "Brand", "name": "Product" },
    "user": { "id": "uuid", "name": "User Name" },
    "rating": 4.5,
    "text": "Good value.",
    "created_at": 1708012800,
    "updated_at": 1708012800,
    "deleted_at": null
  }
]
```

#### `GET /api/v1/reviews/:id`

Get a single review by ID. Response includes nested `product` and `user`.

**Response:** `200 OK` (review object, same shape as list items)

**Errors:**
- `404 Not Found`: Review not found

#### `POST /api/v1/reviews`

Create a new review.

**Request body:**
```json
{
  "product_id": "uuid",
  "rating": 4.5,
  "text": "Good value."
}
```

**Constraints:**
- `product_id` and `rating` are required
- `rating` must be between 1 and 5 (decimal subdivisions allowed, e.g., 4.5)
- `text` is optional
- `user_id` is automatically set to the current authenticated user
- Multiple reviews per (user, product) are allowed

**Response:** `201 Created` (review object with nested `product` and `user`)

**Errors:**
- `400 Bad Request`: Validation error (e.g., rating out of range)
- `404 Not Found`: Product not found

#### `PATCH /api/v1/reviews/:id`

Update a review.

**Request body:**
```json
{
  "rating": 5,
  "text": "Excellent."
}
```

All fields are optional. Only provided fields are updated.

**Constraints:**
- Only the review owner can update their review

**Response:** `200 OK` (updated review object with nested `product` and `user`)

**Errors:**
- `400 Bad Request`: Validation error
- `403 Forbidden`: Not the review owner
- `404 Not Found`: Review not found

#### `DELETE /api/v1/reviews/:id`

Soft-delete a review.

**Query parameters:**
- `force` (optional, boolean): If `true`, perform hard delete

**Constraints:**
- Only the review owner can delete their review

**Response:** `204 No Content` (or `200 OK` with empty body)

**Errors:**
- `403 Forbidden`: Not the review owner
- `404 Not Found`: Review not found

---

## Runnable Examples

Runnable HTTP examples are available in [api.http](api.http). They assume:
- Server is running (default `http://127.0.0.1:3099`)
- User is registered via CLI (`pocketratings user register`)
- For protected routes: run the Login request first, then set `@token` variable with the returned token

Use the REST Client extension (`humao.rest-client`) in Cursor/VS Code to run requests directly.
