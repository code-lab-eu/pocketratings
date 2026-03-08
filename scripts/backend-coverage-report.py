#!/usr/bin/env python3
#
# Parse backend LCOV coverage and print a table of source files ranked by
# uncovered line count (highest impact first). Used by the backend-coverage-
# improvement skill so the AI can pick targets for adding tests.
#
# Usage (from repo root):
#   ./scripts/backend-coverage-report.py [path/to/lcov.info]
#
# If lcov.info path is omitted, you must run coverage first, e.g.:
#   cd backend && cargo llvm-cov --lcov --output-path lcov.info -- --skip server_start_and_stop_via_cli
# Then run this script with backend/lcov.info or from backend/ with lcov.info.
#

import sys
from pathlib import Path


def parse_lcov(path: Path) -> list[tuple[str, int, int]]:
    """Parse LCOV file; return list of (source_file, lines_found, lines_hit)."""
    records: list[tuple[str, int, int]] = []
    current_sf = None
    current_lf = None
    current_lh = None

    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if line.startswith("SF:"):
                current_sf = line[3:]
                current_lf = None
                current_lh = None
            elif line.startswith("LF:"):
                current_lf = int(line[3:])
            elif line.startswith("LH:"):
                current_lh = int(line[3:])
            elif line == "end_of_record" and current_sf is not None:
                if current_lf is not None and current_lh is not None:
                    records.append((current_sf, current_lf, current_lh))
                current_sf = None

    return records


def main() -> None:
    # Script lives in repo/scripts/; repo root is parent of scripts/
    repo_root = Path(__file__).resolve().parent.parent

    if len(sys.argv) >= 2:
        lcov_path = Path(sys.argv[1])
    else:
        lcov_path = repo_root / "backend" / "lcov.info"

    if not lcov_path.exists():
        print(
            f"Error: {lcov_path} not found. Run coverage first, e.g.:",
            file=sys.stderr,
        )
        print(
            "  cd backend && cargo llvm-cov --lcov --output-path lcov.info -- "
            "--skip server_start_and_stop_via_cli",
            file=sys.stderr,
        )
        sys.exit(1)

    records = parse_lcov(lcov_path)

    # Keep only backend src files; normalize path to src/...
    backend_src = "backend/src"
    rows: list[tuple[str, int, int, int]] = []
    for sf, lf, lh in records:
        if backend_src not in sf:
            continue
        idx = sf.index(backend_src) + len(backend_src) + 1
        short = sf[idx:] if idx <= len(sf) else sf
        uncovered = lf - lh
        pct = (100 * lh / lf) if lf else 0
        rows.append((short, int(pct), uncovered, lf))

    # Sort by uncovered count descending (highest impact first)
    rows.sort(key=lambda r: (-r[2], r[0]))

    # High-value paths: api/, auth/, db/, domain/ — critical for application behavior
    high_value_prefixes = ("api/", "auth/", "db/", "domain/")
    high_value = [r for r in rows if r[0].startswith(high_value_prefixes)]
    other = [r for r in rows if not r[0].startswith(high_value_prefixes)]

    def print_table(rows_block: list[tuple[str, int, int, int]], title: str) -> None:
        if not rows_block:
            return
        col_path = "path"
        col_pct = "line_cov%"
        col_uncovered = "uncovered"
        col_total = "lines"
        w_path = max(len(col_path), max(len(r[0]) for r in rows_block))
        print(title)
        print(f"{col_path:<{w_path}}  {col_pct:>8}  {col_uncovered:>9}  {col_total:>5}")
        print("-" * (w_path + 2 + 8 + 2 + 9 + 2 + 5))
        for short, pct, uncovered, lf in rows_block:
            print(f"{short:<{w_path}}  {pct:>7}%  {uncovered:>9}  {lf:>5}")
        print()

    print_table(high_value, "High-value targets (api/, auth/, db/, domain/) — by uncovered count:")
    print_table(other, "Other (cli/, main, config/, etc.) — by uncovered count:")


if __name__ == "__main__":
    main()
