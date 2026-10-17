---
name: env-configuration
description: Keeping .env.example in sync with the environment variables the app reads. Use when adding or changing an env var in backend/src/config/ (or other config code), or when editing backend/.env.example or the root .env.example.
---

# Environment variables and .env.example

When you add or change an environment variable that the application reads
(e.g. in `backend/src/config/`), update the corresponding `.env.example`
in the same change:

- `backend/.env.example` for backend config; the root `.env.example` for Docker
  Compose and deployment variables.
- The Podman Quadlet units do not read `.env`. For a required backend variable,
  also add an `Environment=` line to the backend drop-in example in step 1 of
  "Podman Quadlet" in `README.md`.
- Add a line for the new variable with a comment describing it and its
  default, if any.
- For optional variables with defaults, comment the line out and document
  the default in the comment, e.g.
  `# JWT_EXPIRATION_SECONDS=2592000  # default: 30 days`.
- Keep the style of the existing entries (comment above or inline,
  consistent formatting).
