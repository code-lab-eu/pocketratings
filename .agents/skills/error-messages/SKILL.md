---
name: error-messages
description: Wording rules for user-facing error messages - sentence case, trailing period, consistent phrasing. Use when writing an ApiError string in backend/src/api/, or validation and error-display copy in a .svelte or .ts file under frontend/src/.
---

# Error message formatting

User-facing error messages (the API `message` field and frontend
validation or display copy) must be consistent and professional.

## Rules

- **Capitalize** the first letter (sentence case).
- **End with a period.**
- Use consistent phrasing:
  - Empty required field: **"X is required."** (e.g. "Name is required.",
    "Email is required.")
  - Invalid value or constraint: **"X must be ..."** or **"Invalid X."**
    (e.g. "Rating must be between 1 and 5.", "Invalid price.", "Price
    must not be negative.")
  - Not found: **"X not found."** (e.g. "Category not found.", "Product
    not found.")

## Where it applies

- **Backend:** strings passed to `ApiError::BadRequest`,
  `ApiError::NotFound`, `ApiError::Unauthorized`, and the like, which are
  returned in the JSON `message` field.
- **Frontend:** client-side validation strings (e.g.
  `error = 'Name is required.'`) and fallback text when showing API errors
  (e.g. "Login failed."). The API message is shown as-is, so backend and
  frontend must follow the same style.

## Examples

- Good: "Name is required.", "Brand is required.", "Quantity must be at
  least 1.", "Invalid email or password."
- Bad: "name must not be empty", "invalid rating", "Product not found"
  (no period).
