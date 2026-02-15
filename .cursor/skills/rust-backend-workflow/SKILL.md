---
name: rust-backend-workflow
description: Enforces TDD, full test coverage (unit + CLI + REST), no unwrap/unsafe, and proper error handling when writing or modifying Pocket Ratings backend Rust code. Use when working on the backend, adding features, implementing API or CLI, or writing Rust code in this project.
---

# Rust Backend Workflow

Apply this workflow whenever writing or changing code in `backend/` (Pocket Ratings).

## Test-driven development

- **Every feature has a test.** Write or update the test first (or alongside) the implementation; do not add production code without a corresponding test.
- Run `cargo test` (or use the build-and-test skill) to confirm tests pass before considering the change done.

## Test coverage

Three layers are required:

1. **Unit tests** — For domain logic, validation, and pure functions. Place in the same module (`#[cfg(test)] mod tests { ... }`) or in `backend/tests/` for integration-style tests.
2. **CLI tests** — For every CLI command (e.g. `user register`, `category create`, `purchase list`). Test by invoking the binary (or the CLI entry point) with arguments and asserting on exit code and stdout/stderr. Use integration tests in `backend/tests/` that run `cargo run -- ...` or call the CLI API.
3. **REST endpoint tests** — For every API route under `/api/v1/`. Test with HTTP requests (e.g. using the test server or a client). Cover success and error cases (401/403, 400 validation, 404). Use integration tests in `backend/tests/` that start the app or a test router and send requests.

When adding a new CLI command or REST endpoint, add the corresponding test in the same change.

## Safe code — no unwrap or unsafe

- **Do not use `unwrap()`, `expect()`, `unwrap_or_else()` on `Result`/`Option` in production code.** Use `?` to propagate errors or handle with `match`/`if let` and return a proper error.
- **Do not use `unsafe`** unless there is a documented, justified exception (none expected for this project).
- In tests, `unwrap()` or `expect()` is acceptable only to assert invariants (e.g. "this must be Some in this test"); prefer asserting on the `Result`/`Option` when possible.

## Proper error handling

- **Library code**: Use `Result<T, E>` with custom error types. Use **thiserror** for `E` so errors are descriptive and chainable. Propagate with `?`; do not swallow errors.
- **Binary / entry points**: Use **anyhow** (or similar) for context when appropriate; convert library errors into HTTP status codes or CLI exit codes and user-facing messages.
- **API**: Map errors to appropriate status codes (400, 403, 404, 500) and a consistent JSON body (e.g. `{ "error": "..." }`). Do not leak internal details in production.
- **CLI**: Map errors to non-zero exit codes and clear stderr messages. Do not panic.

## Checklist before submitting backend changes

- [ ] New/changed behavior has unit and/or integration tests.
- [ ] New CLI commands have CLI tests; new REST endpoints have REST tests.
- [ ] No `unwrap()`/`expect()` in production code; no `unsafe`.
- [ ] Errors use `Result` and thiserror/anyhow; API and CLI map errors appropriately.
- [ ] `cargo test` and `cargo clippy` (with project flags) pass.
