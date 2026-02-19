---
name: frontend-development
description: Enforces tests and quality control when writing or modifying Pocket Ratings frontend (Svelte/SvelteKit) code. Use when working on the frontend, adding features, new routes, API client helpers, or components.
---

# Frontend Development Workflow

Apply this workflow whenever writing or changing code in `frontend/` (Pocket Ratings).

## Tests are required

- **Do not add or change production code without corresponding tests.** This includes: new or changed API client functions in `lib/api.ts`, shared logic (e.g. merging products and reviews), new or changed types used by the API, and meaningful UI (pages and components).
- **Prefer writing tests as you go.** When adding a feature, add the test(s) that define the desired behaviour in the same change as the implementation. New behaviour without a test is incomplete work.
- **What to test:**
  - **API client and pure logic:** Unit tests (Vitest) for new helpers in `lib/api.ts` and any shared logic. Use mocked `fetch` or stub responses. Follow the style of existing [lib/api.test.ts](frontend/src/lib/api.test.ts) and [lib/auth.test.ts](frontend/src/lib/auth.test.ts).
  - **Components and pages:** Add component or page-level tests where they add value: key elements render, loading/error/empty states, and critical user flows. Use Vitest with the project’s existing setup (e.g. Svelte Testing Library if present).

**Rule:** When adding a new API helper, page, or component with non-trivial behaviour, add the corresponding test in the same change.

## Before considering work done

Run **frontend quality control** (use the frontend-quality-control skill):

- From repo root: `cd frontend && bun run lint && bun run test`
- `bun run lint` must pass (svelte-check and ESLint)
- `bun run test` must pass (all Vitest tests)

Do not submit frontend changes with failing lint or tests.

## Checklist before submitting frontend changes

- [ ] New or changed API helpers and shared logic have unit tests.
- [ ] New or changed pages/components have tests where they add value (e.g. render, loading/error/empty).
- [ ] `bun run lint` and `bun run test` in `frontend/` both pass (use the frontend-quality-control skill).
