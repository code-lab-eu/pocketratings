#!/usr/bin/env sh
#
# ASCII punctuation: fix or check for curly quotes.
# Replaces: " " ' ' (U+201C, U+201D, U+2018, U+2019) with " '
#
# Usage:
#   ./scripts/ascii-punctuation.sh fix   # replace in tracked text files
#   ./scripts/ascii-punctuation.sh check  # exit 1 if any found (for CI/hooks)
#
# See .cursor/rules/ascii-punctuation-no-broken-edits.mdc

set -e
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT" || exit 1

# Tracked text files (code, docs, config). Exclude lock files and generated.
LSFILES="git ls-files"
SUFFIXES='\.(md|mdc|rs|svelte|ts|tsx|js|jsx|json|yaml|yml|toml|http|sh)$'
FILES="$($LSFILES | grep -E "$SUFFIXES" | grep -v -E 'lock\.(json|yaml|yml)$' | grep -v 'sandbox\.json' || true)"

# Perl: curly quotes only (U+201C U+201D U+2018 U+2019) -> " '
PERL_MATCH='[\x{201C}\x{201D}\x{2018}\x{2019}]'
PERL_FIX="s/[\x{201C}\x{201D}]/\"/g; s/[\x{2018}\x{2019}]/chr(39)/ge"

case "${1:-}" in
fix)
	if [ -z "$FILES" ]; then exit 0; fi
	echo "$FILES" | xargs perl -i -CSD -pe "$PERL_FIX"
	echo "Replaced non-ASCII punctuation in tracked text files."
	;;
check)
	if [ -z "$FILES" ]; then echo "ASCII punctuation check passed."; exit 0; fi
	OUT=$(echo "$FILES" | xargs perl -CSD -n -e "print \"\$ARGV:\$.: \$_\" if /${PERL_MATCH}/" 2>/dev/null || true)
	if [ -n "$OUT" ]; then
		echo "Non-ASCII punctuation (use ./scripts/ascii-punctuation.sh fix):"
		echo "$OUT"
		exit 1
	fi
	echo "ASCII punctuation check passed."
	;;
*)
	echo "Usage: $0 fix|check" >&2
	exit 1
	;;
esac
