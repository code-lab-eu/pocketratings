---
name: frontend-development
description: Enforces tests and quality control when writing or modifying Pocket Ratings frontend (Svelte/SvelteKit) code. Use when working on the frontend, adding features, new routes, API client helpers, or components.
---

# Frontend Development Workflow

Apply this workflow whenever writing or changing code in `frontend/` (Pocket Ratings).

## Tests are required

- **Do not add or change production code without corresponding tests.** This includes: new or changed API client functions in `lib/api.ts`, shared logic (e.g. merging products and reviews), new or changed types used by the API, and meaningful UI (pages and components).
- **Tests describe the expected outcome.** Write tests so they specify the behaviour the code should meet (e.g. given these inputs, expect this response or this UI state). The tests are the specification; the implementation should make them pass.
- **Prefer writing tests first.** Ideally write the test(s) that define the desired behaviour before writing the production code; otherwise add them in the same change as the implementation. New behaviour without a test is incomplete work.
- **Phased plans:** When work is split into phases, each phase must include its own test coverage. Do not defer tests to a later phase. Ideally write tests for that phase first, then implement.
- **Styling-only work does not need new tests.** Do **not** add or extend tests
  whose sole purpose is to assert CSS, design tokens in `layout.css`, font
  sizes, class names for appearance, or other purely visual details. Manual or
  visual review is enough. The existing test suite must still pass after your
  edits.
- **What to test:**
  - **API client and pure logic:** Unit tests (Vitest) for new helpers in `lib/api.ts` and any shared logic. Use mocked `fetch` or stub responses. Follow the style of existing [lib/api.test.ts](frontend/src/lib/api.test.ts) and [lib/auth.test.ts](frontend/src/lib/auth.test.ts).
  - **Components and pages:** Add component or page-level tests where they add value: key elements render, loading/error/empty states, and critical user flows. Use Vitest with the project's existing setup (e.g. Svelte Testing Library if present).

**Rule:** When adding a new API helper, page, or component with non-trivial behaviour, add the corresponding test in the same change.

## Before considering work done

**Do not mark frontend work complete until frontend quality control has been run and passed.** Run **frontend quality control** (use the frontend-quality-control skill):

- From repo root: `cd frontend && bun run lint && bun run test`
- `bun run lint` must pass (svelte-check and ESLint)
- `bun run test` must pass (all Vitest tests)

Do not submit frontend changes with failing lint or tests. See the **task-completion-qa** rule.

## Checklist before submitting frontend changes

- [ ] New or changed API helpers and shared logic have unit tests.
- [ ] New or changed pages/components have tests where they add value (e.g. render, loading/error/empty).
- [ ] `bun run lint` and `bun run test` in `frontend/` both pass (use the frontend-quality-control skill).
