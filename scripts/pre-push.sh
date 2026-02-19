#!/bin/sh
#
# Pre-push checks: run backend and frontend quality checks.
# Exit non-zero to abort the push when used as a git hook.
#
# Run on demand from repo root:
#   ./scripts/pre-push.sh
#
# Install as a git pre-push hook (runs automatically before every push):
#   ln -sf ../../scripts/pre-push.sh .git/hooks/pre-push
#

set -e
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT" || exit 1

# --- Backend ---
echo "=== Backend: format check ==="
(cd backend && cargo fmt --check) || {
	echo "Run: cd backend && cargo fmt"
	exit 1
}
echo "=== Backend: clippy (pedantic, all targets) ==="
(cd backend && cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings)
echo "=== Backend: test ==="
(cd backend && cargo test)

# --- Frontend ---
echo "=== Frontend: lint ==="
if ! (cd frontend && bun run lint); then
	echo "Lint failed. Running ESLint --fix..."
	(cd frontend && bun run lint:fix)
	echo "Re-run pre-push after reviewing and committing fixes."
	exit 1
fi
echo "=== Frontend: test ==="
(cd frontend && bun run test)

echo "Pre-push checks passed."
exit 0
