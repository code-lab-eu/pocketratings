---
name: backend-coverage-improvement
description: Run backend coverage, identify files with low or missing coverage ranked by impact (uncovered lines), and add tests to improve coverage. Use when the user asks to improve backend coverage, find low-coverage areas, or add tests for under-covered code.
---

# Backend coverage improvement

Use this skill when the user wants to improve backend test coverage, find
which parts of the codebase need more coverage, or add tests for
under-covered code.

## Workflow

### 1. Generate coverage data

From the repo root, run coverage and write LCOV (so the report script can
parse it):

```bash
cd backend && cargo llvm-cov --lcov --output-path lcov.info -- --skip server_start_and_stop_via_cli
```

Requires `cargo-llvm-cov` (`cargo install cargo-llvm-cov`). Use the same
`--skip server_start_and_stop_via_cli` as in backend QC and pre-push.

### 2. Get ranked list of targets

From the repo root:

```bash
python3 scripts/backend-coverage-report.py backend/lcov.info
```

Or omit the path to use the default `backend/lcov.info`:

```bash
python3 scripts/backend-coverage-report.py
```

The script prints a table: **path** (under `backend/src`), **line_cov%**,
**uncovered** (count of lines not covered), **lines** (total). Files are
**sorted by uncovered count descending** so the highest-impact targets
appear first.

### 3. Choose targets and add coverage

Pick one or more files from the top of the list (or all files below the
project’s line-coverage threshold; see `scripts/backend-coverage.sh` for
the current threshold). For each target:

- Add or extend **unit tests** (in the same crate, `#[cfg(test)]` or
  `tests/` as appropriate) and/or **API/integration tests** (in
  `backend/tests/` or API module tests) to cover the missing lines.
- Follow the **rust-backend-workflow** skill: TDD, no `unwrap`/`unsafe`,
  proper error handling, full test coverage expectations for new code.

### 4. Confirm coverage still passes

Run the backend coverage script from the repo root to ensure the
threshold is still met:

```bash
./scripts/backend-coverage.sh
```

Re-run the report script after adding tests to see the updated ranking and
confirm the chosen files’ coverage improved.

## References

- **backend-quality-control** — How to run format, lint, and test; same
  test filter (`--skip server_start_and_stop_via_cli`).
- **rust-backend-workflow** — Test style, TDD, no unwrap/unsafe, error
  handling when writing backend Rust code.
- **scripts/backend-coverage.sh** — Enforced line-coverage threshold (CI
  and pre-push).
