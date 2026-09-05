---
name: frontend-quality-control
description: Run lint and test for the Pocket Ratings frontend with svelte-check, ESLint, and Vitest. Use when verifying frontend code quality, before submitting frontend changes, or when the user asks to run QC, lint, or check that the frontend passes.
---

# Frontend Quality Control

Run these checks whenever you need to verify the Pocket Ratings frontend passes quality control. Use this skill when confirming frontend changes are ready, before marking work done, or when the user asks to run QC, lint, or check the frontend.

Run from the repo root with `cd frontend` or set `working-directory: frontend` in CI. Always use **bun** (not npm).

## Order of checks

### 1. Lint

Runs SvelteKit sync, svelte-check (type checking), and ESLint:

```bash
cd frontend
bun run lint
```

Fix any type or lint errors before proceeding.

### 2. Test

```bash
cd frontend
bun run test
```

All Vitest tests (unit and component) must pass.

## One-shot (all checks)

```bash
cd frontend && bun run lint && bun run test
```

## Done when

- `bun run lint` exits 0 (svelte-check and ESLint pass).
- `bun run test` exits 0 (all Vitest tests pass).
