---
name: build-and-test
description: Run build, lint, and test commands for Pocket Ratings backend (Rust) and frontend (Svelte). Use when the user asks to build, lint, test, or check code quality for either the backend or frontend.
---

# Build and Test

- **Backend:** Use the **backend-build-and-test** skill for build, lint, and test. If `cargo` fails with "no default is configured", run `rustup default stable` (with network) then retry.
- **Frontend:** Use the **frontend-build-and-test** skill for install, build, lint, test, and dev. Always use `bun` (not npm) for frontend commands.
