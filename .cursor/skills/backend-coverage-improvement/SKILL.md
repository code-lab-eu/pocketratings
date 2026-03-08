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

The script prints two sections:

1. **High-value targets (api/, auth/, db/, domain/)** — API layer,
   authentication, database layer, and domain logic. These are critical for
   application behavior, security, and correctness. Sorted by uncovered
   count descending.
2. **Other (cli/, main, config/, etc.)** — CLI wiring, entry points, config.
   Often harder to test and less critical; deprioritize these.

Each table has **path**, **line_cov%**, **uncovered**, **lines**.

### 3. Choose targets and add coverage

**Prefer the high-value section.** Choose one or more files from the **top
of the high-value targets** list (api/, auth/, db/, domain/). These are the
most critical for the application. Only if that section is empty or you need
more coverage should you consider the "Other" list; avoid prioritizing
cli/, main.rs, and config/. Do not pick files far down the list. For each
target:

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
