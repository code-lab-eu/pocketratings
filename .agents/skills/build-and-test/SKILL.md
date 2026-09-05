---
name: build-and-test
description: Run build, lint, and test commands for Pocket Ratings backend (Rust) and frontend (Svelte). Use when the user asks to build, lint, test, or check code quality for either the backend or frontend.
---

# Build and Test

- **Backend:** Use **backend-build-and-test** for build, lint, or test when the user asks for those. For task completion or full QC (format + lint + test + coverage), use **backend-quality-control**. Always use `--skip server_start_and_stop_via_cli` when running `cargo test`. If `cargo` fails with "no default is configured", run `rustup default stable` (with network) then retry.
- **Frontend:** Use the **frontend-build-and-test** skill for install, build, lint, test, and dev. For full QC (lint + test), use **frontend-quality-control**. Always use `bun` (not npm) for frontend commands.
