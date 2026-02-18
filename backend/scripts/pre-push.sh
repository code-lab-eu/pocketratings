#!/bin/sh
#
# Pre-push checks: run backend format, Clippy (pedantic), and tests.
# Exit non-zero to abort the push when used as a git hook.
#
# Run on demand from repo root:
#   ./backend/scripts/pre-push.sh
#
# Install as a git pre-push hook (so it runs automatically before every push):
#   ln -sf ../../backend/scripts/pre-push.sh .git/hooks/pre-push
#

set -e
cd "$(git rev-parse --show-toplevel)/backend" || exit 1

echo "Running cargo fmt --check..."
if ! cargo fmt --check; then
  cargo fmt
  exit 1
fi

echo "Running cargo clippy (pedantic)..."
cargo clippy -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings

echo "Running cargo test..."
cargo test

echo "Pre-push checks passed."
exit 0
