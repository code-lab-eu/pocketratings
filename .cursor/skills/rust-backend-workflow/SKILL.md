---
name: rust-backend-workflow
description: Enforces TDD, full test coverage (unit + CLI + REST), no unwrap/unsafe, and proper error handling when writing or modifying Pocket Ratings backend Rust code. Use when working on the backend, adding features, implementing API or CLI, or writing Rust code in this project.
---

# Rust Backend Workflow

Apply this workflow whenever writing or changing code in `backend/` (Pocket Ratings).

## Test-driven development

- **Tests are required for all new functionality.** Do not add or change production code without corresponding tests. This includes: new or changed database functions, new CLI commands, new REST endpoints, and new domain logic.
- **Prefer writing tests first.** When adding a feature, write the test(s) that define the desired behaviour before (or as the first step of) implementing. Then implement until the tests pass. If you implement first, add the tests in the same change and treat missing tests as incomplete work.
- Before considering the change done, run **backend quality control** (use the backend-quality-control skill: format, then Clippy in strict pedantic mode, then tests).

## Test coverage

Three layers are required:

1. **Database / persistence tests** — For every new or changed function in `db/` (e.g. `list_all`, `insert`, `get_by_email`). Use integration tests in `backend/tests/` (e.g. `user_db_test.rs`) that create a temp DB, run migrations, and assert on query results. Cover both success and relevant edge cases (e.g. empty list, filtered vs unfiltered).
2. **CLI tests** — For every CLI command (e.g. `user register`, `user list`, `category create`). Test by invoking the CLI entry point with arguments and captured stdout/stderr; assert on exit code and output. Use integration tests in `backend/tests/` that call the CLI API with a temp DB. Cover success and error cases (e.g. duplicate email, invalid input, `--output json`).
3. **REST endpoint tests** — For every API route under `/api/v1/`. Put **one endpoint per file** in `backend/src/api/` (e.g. `version.rs`). In that file: define the handler, response types, and a `route()` that returns a `Router` for this endpoint; add a `#[cfg(test)] mod tests` with in-process tests (e.g. `tower::ServiceExt::oneshot` on `route()`; assert status and response body). Do not start a real server or use TCP. The main `api/router.rs` composes the API by merging each endpoint’s `route()`. Cover success and error cases (401/403, 400 validation, 404).

**Rule:** When adding a new DB function, CLI command, or REST endpoint, add the corresponding test in the same change. New behaviour without a test is not done.

### Test helpers and duplication

- **Avoid copy-paste helpers.** If multiple tests need the same setup (e.g. `insert_user`, `insert_category`, `insert_product`, `insert_location`, or `test_pool` builders), prefer extracting shared helpers instead of duplicating them across modules.
- It is acceptable for:
  - **Unit/REST tests in `src/*`** to use a shared, `#[cfg(test)]` helper module (e.g. `crate::test_helpers`) for common fixtures.
  - **Integration tests in `backend/tests/*`** to have their own helper module(s) or functions as long as they are not mindlessly duplicated.
- When you notice duplication between endpoint tests (e.g. review vs purchase) or between multiple integration tests, **factor the common bits out** into a helper module in the same directory or a shared `test_helpers` module, and reuse it.

## Safe code — no unwrap or unsafe

- **Do not use `unwrap()`, `expect()`, `unwrap_or_else()` on `Result`/`Option` in production code.** Use `?` to propagate errors or handle with `match`/`if let` and return a proper error.
- **`unsafe` is strictly forbidden** in this project. The crate enforces this with `#![forbid(unsafe_code)]` in `lib.rs` and `main.rs`, so the build fails if anyone adds `unsafe`. Do not use `unsafe` in production code or in tests.
- In tests, `unwrap()` or `expect()` is acceptable only to assert invariants (e.g. "this must be Some in this test"); prefer asserting on the `Result`/`Option` when possible.

## Proper error handling

- **Library code**: Use `Result<T, E>` with custom error types. Use **thiserror** for `E` so errors are descriptive and chainable. Propagate with `?`; do not swallow errors.
- **Binary / entry points**: Use **anyhow** (or similar) for context when appropriate; convert library errors into HTTP status codes or CLI exit codes and user-facing messages.
- **API**: Map errors to appropriate status codes (400, 403, 404, 500) and a consistent JSON body (e.g. `{ "error": "..." }`). Do not leak internal details in production.
- **CLI**: Map errors to non-zero exit codes and clear stderr messages. Do not panic.

## Checklist before submitting backend changes

- [ ] **Tests first or in same change:** New/changed DB functions have tests in `backend/tests/*_db_test.rs`; new CLI commands have tests in `backend/tests/cli_*_test.rs`; new REST endpoints have tests in the same file as the handler (e.g. `api/version.rs` with a `#[cfg(test)] mod tests`). Prefer writing tests before implementing.
- [ ] New/changed behaviour has unit and/or integration tests (no production code without a test), and obvious test helper duplication has been factored into shared helpers where practical.
- [ ] No `unwrap()`/`expect()` in production code; no `unsafe` (strictly forbidden).
- [ ] Errors use `Result` and thiserror/anyhow; API and CLI map errors appropriately.
- [ ] **Backend quality control** has been run and passes (use the backend-quality-control skill: format, Clippy in strict pedantic mode, then tests).
