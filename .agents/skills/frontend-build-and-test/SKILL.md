---
name: frontend-build-and-test
description: Run install, build, lint, test, and dev for the Pocket Ratings frontend (Svelte). Use when the user asks to build, lint, or test the frontend, or verify frontend before submit.
---

# Frontend Build and Test

Use this skill when running frontend commands. Run from the repo root with `cd frontend` or set `working-directory: frontend` in CI. Always use **bun** (not npm).

## Install dependencies

```bash
cd frontend
bun install
```

## Build

```bash
cd frontend
bun run build
```

## Lint

Runs SvelteKit sync, `svelte-check` (type checking), and ESLint:

```bash
cd frontend
bun run lint
```

## Test

Vitest (unit and component tests); no E2E:

```bash
cd frontend
bun run test
```

## Dev server

```bash
cd frontend
bun run dev
```
