---
name: rust-backend-workflow
description: Enforces TDD, full test coverage (unit + CLI + REST), no unwrap/unsafe, and proper error handling when writing or modifying Pocket Ratings backend Rust code. Use when working on the backend, adding features, implementing API or CLI, or writing Rust code in this project.
---

# Rust Backend Workflow

Apply this workflow whenever writing or changing code in `backend/` (Pocket Ratings).

## Test-driven development

- **Tests are required for all new functionality.** Do not add or change production code without corresponding tests. This includes: new or changed database functions, new CLI commands, new REST endpoints, and new domain logic.
- **Prefer writing tests first.** When adding a feature, write the test(s) that define the desired behaviour before (or as the first step of) implementing. Then implement until the tests pass. If you implement first, add the tests in the same change and treat missing tests as incomplete work.
- Run `cargo test` (or use the build-and-test skill) to confirm tests pass before considering the change done.
- **Run `cargo fmt`** in `backend/` after completing a code editing task so the codebase stays consistently formatted.

## Test coverage

Three layers are required:

1. **Database / persistence tests** — For every new or changed function in `db/` (e.g. `list_all`, `insert`, `get_by_email`). Use integration tests in `backend/tests/` (e.g. `user_db_test.rs`) that create a temp DB, run migrations, and assert on query results. Cover both success and relevant edge cases (e.g. empty list, filtered vs unfiltered).
2. **CLI tests** — For every CLI command (e.g. `user register`, `user list`, `category create`). Test by invoking the CLI entry point with arguments and captured stdout/stderr; assert on exit code and output. Use integration tests in `backend/tests/` that call the CLI API with a temp DB. Cover success and error cases (e.g. duplicate email, invalid input, `--output json`).
3. **REST endpoint tests** — For every API route under `/api/v1/`. Test with HTTP requests (e.g. using the test server or a client). Cover success and error cases (401/403, 400 validation, 404). Use integration tests in `backend/tests/` that start the app or a test router and send requests.

**Rule:** When adding a new DB function, CLI command, or REST endpoint, add the corresponding test in the same change. New behaviour without a test is not done.

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

- [ ] **Tests first or in same change:** New/changed DB functions have tests in `backend/tests/*_db_test.rs`; new CLI commands have tests in `backend/tests/cli_*_test.rs`; new REST endpoints have HTTP tests. Prefer writing tests before implementing.
- [ ] New/changed behaviour has unit and/or integration tests (no production code without a test).
- [ ] No `unwrap()`/`expect()` in production code; no `unsafe`.
- [ ] Errors use `Result` and thiserror/anyhow; API and CLI map errors appropriately.
- [ ] `cargo test` and `cargo clippy` (with project flags) pass.
- [ ] `cargo fmt` has been run in `backend/` (formatting is up to date).
