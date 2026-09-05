---
name: markdown-conventions
description: How to edit markdown in this repo - 80-character wrap, ASCII punctuation, and a replace strategy that survives Unicode quotes. Use when editing README.md, any file under docs/ (spec.md, api.md, roadmap.md, plans), AGENTS.md, or a SKILL.md under .agents/skills/, and whenever an exact-string replace in a markdown file fails to match.
---

# Markdown conventions

Applies to `README.md`, files under `docs/` (including `docs/plans/`),
`AGENTS.md`, and `SKILL.md` files under `.agents/skills/`.

## Line width

Wrap prose and list text to **80 characters** per line. Break a long
sentence or list item onto the next line rather than exceeding 80
characters. This keeps docs readable in narrow terminals and keeps diffs
minimal.

**Code blocks** are exempt: a long command or URL may be split with a
shell line continuation where that does not change behaviour, but leave it
long if breaking would harm copy-paste.

## Punctuation

Write new content with ASCII punctuation: straight double quote `"`,
straight single quote `'`, hyphen-minus `-`. En and em dashes are
tolerated (the check script ignores them), but curly quotes (U+2018,
U+2019, U+201C, U+201D) are rejected by
`./scripts/ascii-punctuation.sh check` on pre-push and in CI.

## Replace strategy

Exact-string replace often fails in these files because existing text may
contain Unicode punctuation - curly quotes, the right single quote in
"product's", an em dash - that looks like ASCII in the editor but does not
match.

1. **Choose boundaries that avoid ambiguous punctuation.** Match on
   headings, list markers, or lines that are clearly ASCII (e.g.
   `### 1. Title` or `- Document in`). Keep the line containing the curly
   quote outside the matched span.
2. **Replace minimally.** If you must change a line that has special
   characters, match only an ASCII substring of it, or a unique block that
   excludes the problematic character.
3. **When a replace fails** (including a fuzzy match landing elsewhere),
   assume a Unicode or whitespace mismatch. Normalize, then retry the
   intended replace:

   ```bash
   ./scripts/ascii-punctuation.sh fix
   ```

   For a single file:

   ```bash
   perl -i -CSD -pe 's/[\x{201C}\x{201D}]/"/g; s/[\x{2018}\x{2019}]/chr(39)/ge' -- path/to/file.md
   ```

   Retrying the same replace with hex dumps or pasted Unicode wastes
   tokens and is brittle. Normalizing is faster than re-reading and
   rewriting the whole file.
4. **Fallback:** if normalization plus retry still fails, or the edit is
   very large, read the affected section and write it back with ASCII
   punctuation throughout.

## Complete edits

Fix the whole affected span in one go. A replace that changes only part of
a sentence, leaves a placeholder ("(Remove this later...)"), or leaves
duplicate phrasing is worse than no edit. If the span is too tangled for
one replace, read the section and write it back whole.
