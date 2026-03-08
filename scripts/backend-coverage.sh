#!/bin/sh
#
# Run backend coverage with cargo-llvm-cov and enforce a line-coverage
# threshold. Used by CI and pre-push. Threshold is calibrated from current
# coverage (as of task 5) and should be raised as coverage improves.
#
# Run from repo root:
#   ./scripts/backend-coverage.sh
#
# Requires: cargo-llvm-cov (cargo install cargo-llvm-cov)
#

set -e
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT/backend" || exit 1

# Line coverage threshold (%). Pinned at 88% after high-value coverage improvements.
FAIL_UNDER_LINES=88

exec cargo llvm-cov --fail-under-lines "$FAIL_UNDER_LINES" -- --skip server_start_and_stop_via_cli
