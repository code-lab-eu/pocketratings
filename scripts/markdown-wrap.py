#!/usr/bin/env python3
#
# Check or fix the 80-character wrapping of markdown prose, as described in
# the markdown-conventions skill: lines are filled up to 80 characters and
# never break earlier than needed.
#
# Usage (from repo root):
#   ./scripts/markdown-wrap.py check [--since REF] [FILE ...]
#   ./scripts/markdown-wrap.py fix FILE LINE [LINE ...]
#
# check  Report problems and exit 1 if there are any. Without FILE arguments,
#        checks every tracked file the convention applies to (README.md,
#        AGENTS.md, docs/**/*.md, .agents/skills/*/SKILL.md). With --since,
#        only checks paragraphs that contain a line changed since REF
#        (committed, staged, or unstaged), so existing problems elsewhere in
#        a file are not reported.
# fix    Rewrap the paragraphs or list items that contain the given 1-based
#        line numbers.
#
# A prose block is a paragraph or a single list item. Headings, tables,
# fenced code blocks, YAML front matter, and HTML lines are skipped. Inline
# code spans are never broken, so a line that consists of a single code span
# may exceed 80 characters. A line ending in a hard line break (two spaces or
# a backslash) may end early.
#

import argparse
import re
import subprocess
import sys
import textwrap
from pathlib import Path

WIDTH = 80
SCOPE = re.compile(r"^(README\.md|AGENTS\.md|docs/.*\.md|\.agents/skills/[^/]+/SKILL\.md)$")
LIST_MARKER = re.compile(r"^(\s*)([-*+]|\d+[.)]) ")
FENCE = re.compile(r"^\s*(```|~~~)")
CODE_SPAN = re.compile(r"(`+)(?:(?!\1).)+?\1")
# Placeholder for spaces inside code spans, so they are not break points.
NBSP = "\x00"


def is_prose(stripped: str) -> bool:
    """Whether a non-code line takes part in wrapping."""
    if not stripped:
        return False
    return not re.match(r"(#|\||<|---|\*\*\*|___)", stripped)


def blocks(lines: list[str]) -> list[tuple[int, int]]:
    """Return 0-based inclusive (start, end) ranges of prose blocks."""
    result: list[tuple[int, int]] = []
    start = None
    in_fence = False
    in_front_matter = bool(lines) and lines[0] == "---"
    for i, line in enumerate(lines):
        stripped = line.strip()
        if in_front_matter:
            if i > 0 and line == "---":
                in_front_matter = False
            continue
        if FENCE.match(line):
            in_fence = not in_fence
            prose = False
        else:
            prose = not in_fence and is_prose(stripped)
        # Every list item starts its own block. In valid markdown a wrapped
        # continuation line never starts with a list marker.
        if prose and start is not None and LIST_MARKER.match(line):
            result.append((start, i - 1))
            start = None
        if prose and start is None:
            start = i
        elif not prose and start is not None:
            result.append((start, i - 1))
            start = None
    if start is not None:
        result.append((start, len(lines) - 1))
    return result


def protect(text: str) -> str:
    """Replace spaces inside code spans so they are not break points."""
    return CODE_SPAN.sub(lambda m: m.group(0).replace(" ", NBSP), text)


def prefix_of(line: str) -> str:
    """Leading indentation plus list marker, if any."""
    match = LIST_MARKER.match(line)
    if match:
        return match.group(0)
    return line[: len(line) - len(line.lstrip())]


def tokens(line: str) -> list[str]:
    """Unbreakable tokens of a line after its prefix."""
    return protect(line[len(prefix_of(line)):]).split()


def hard_break(line: str) -> bool:
    return line.endswith("  ") or line.endswith("\\")


def check_block(lines: list[str], start: int, end: int) -> list[tuple[int, str]]:
    """Return (0-based line, message) problems in one block."""
    problems: list[tuple[int, str]] = []
    for i in range(start, end + 1):
        line = lines[i]
        if len(line) > WIDTH and len(tokens(line)) > 1:
            problems.append((i, f"{len(line)} characters (over {WIDTH})"))
        if i < end and not hard_break(line):
            following = tokens(lines[i + 1])
            if following and len(line) + 1 + len(following[0]) <= WIDTH:
                word = following[0].replace(NBSP, " ")
                problems.append(
                    (i, f"breaks early ({len(line)} characters; '{word}' fits)")
                )
    return problems


def rewrap(block_lines: list[str]) -> list[str]:
    """Greedily rewrap one block, keeping its list marker and indentation."""
    prefix = prefix_of(block_lines[0])
    indent = " " * len(prefix)
    text = " ".join(line.strip() for line in block_lines)
    marker = prefix.strip()
    if marker and text.startswith(marker + " "):
        text = text[len(marker) + 1:]
    wrapped = textwrap.fill(
        protect(text),
        width=WIDTH,
        initial_indent=prefix,
        subsequent_indent=indent,
        break_long_words=False,
        break_on_hyphens=False,
    )
    return wrapped.replace(NBSP, " ").split("\n")


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], capture_output=True, text=True, check=True
    ).stdout


def changed_lines(path: str, since: str) -> set[int]:
    """0-based lines of the working-tree file that changed since a ref."""
    changed: set[int] = set()
    for hunk in re.finditer(r"^@@ .* \+(\d+)(?:,(\d+))? @@", git("diff", "-U0", since, "--", path), re.M):
        first, count = int(hunk.group(1)), int(hunk.group(2) or "1")
        changed.update(range(first - 1, first - 1 + count))
    return changed


def default_files() -> list[str]:
    return [f for f in git("ls-files").splitlines() if SCOPE.match(f)]


def cmd_check(args: argparse.Namespace) -> int:
    count = 0
    for path in args.files or default_files():
        lines = Path(path).read_text(encoding="utf-8").split("\n")
        wanted = changed_lines(path, args.since) if args.since else None
        for start, end in blocks(lines):
            if wanted is not None and not any(start <= n <= end for n in wanted):
                continue
            for i, message in check_block(lines, start, end):
                print(f"{path}:{i + 1}: {message}")
                count += 1
    if count:
        print(f"{count} wrapping problem(s); fix with ./scripts/markdown-wrap.py fix FILE LINE ...")
        return 1
    print("Markdown wrap check passed.")
    return 0


def cmd_fix(args: argparse.Namespace) -> int:
    path = Path(args.file)
    lines = path.read_text(encoding="utf-8").split("\n")
    wanted = {n - 1 for n in args.lines}
    targets = [(s, e) for s, e in blocks(lines) if any(s <= n <= e for n in wanted)]
    for start, end in reversed(targets):
        lines[start:end + 1] = rewrap(lines[start:end + 1])
    path.write_text("\n".join(lines), encoding="utf-8")
    print(f"Rewrapped {len(targets)} block(s) in {path}.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Check or fix markdown wrapping.")
    sub = parser.add_subparsers(dest="command", required=True)
    check = sub.add_parser("check", help="report wrapping problems")
    check.add_argument("--since", metavar="REF", help="only paragraphs changed since REF")
    check.add_argument("files", nargs="*", metavar="FILE")
    check.set_defaults(run=cmd_check)
    fix = sub.add_parser("fix", help="rewrap the blocks containing the given lines")
    fix.add_argument("file", metavar="FILE")
    fix.add_argument("lines", nargs="+", type=int, metavar="LINE")
    fix.set_defaults(run=cmd_fix)
    args = parser.parse_args()
    return args.run(args)


if __name__ == "__main__":
    sys.exit(main())
