# Pocket Ratings

A small app for personal or family use to track product purchases: rate and
review what you buy, where you bought it, and what you paid.

- `backend/` - Rust (Axum REST API + CLI), SQLite.
- `frontend/` - SvelteKit with Tailwind v4, built and run with bun.
- Deployment - Docker Compose (`compose.yaml`) behind Caddy (`Caddyfile`).

Reference docs: `docs/spec.md` (behaviour, screens, data flow),
`docs/api.md` and `docs/api.http` (REST contract), `docs/roadmap.md`
(planned work), `README.md` (setup and run).

## Skills

Skills live under `.agents/skills/`; list that directory for the available
skills.

## Working agreement

**Ask when instructions conflict.** When something the user said conflicts
with a plan, spec, or doc, or when two parts of the same document point in
different directions, stop and ask which applies. State briefly what
conflicts ("You said X; the plan says Y"), ask which interpretation holds,
and act only on the answer. Do not assume an earlier instruction overrides
a plan, and do not silently drop part of a plan because of an earlier
"don't do X" without confirming X is still out of scope.

**Review quality before running QC.** After implementing or refactoring,
review every changed file for:

1. **Duplication** - blocks of markup, logic, or code that differ by only
   one or two values. Extract the variation (ternary, parameter, variable,
   snippet argument) and share the rest.
2. **Unnecessary complexity** - could a reader understand this in one
   pass? If a simpler structure achieves the same result, use it.
3. **Proportionality** - is the amount of code proportional to what it
   does? A 50-line block that varies one attribute is a smell.
4. **Carry-forward debt** - during refactoring, did you propagate an
   existing problem instead of fixing it? Refactoring must leave the code
   better, not the same.

QC verifies correctness; it does not measure quality. Both must pass.

## Test first

Before adding or changing behaviour, write or update a test that specifies
the expected outcome and watch it fail. Then implement until it passes.
Never implement first and add the test as an afterthought; if you already
did, add the test in the same change and treat the work as incomplete
until it passes.

Applies to new or changed UI (pages, components), API client helpers, API
endpoints, and DB/CLI behaviour. Use the **frontend-development** and
**rust-backend-workflow** skills for the details.

**Styling-only changes are exempt.** Purely visual work (CSS, typography,
spacing, colors, layout classes, design tokens in `layout.css`) needs no
new tests. TDD still applies to behaviour: what the user can do, what data
appears, navigation, validation, API usage.

**Phased plans:** each phase carries its own tests and its own full QC. Do
not defer tests to a later phase.

## Completion gate

A task that modifies code is not complete until the **full** quality
control for that area has been run and passed. A subset - only tests, only
coverage - does not count.

- **Backend:** use the **backend-quality-control** skill (format, strict
  pedantic Clippy, tests, coverage). All four must pass.
- **Frontend:** use the **frontend-quality-control** skill (lint, test).
  Both must pass.
- **Always:** `./scripts/ascii-punctuation.sh check` from the repo root.

This applies whenever a skill or instruction says to "run QC" or check
"before marking work done".

## Markdown and punctuation

Use ASCII straight quotes (`"` and `'`) in code, config, and docs. Curly
quotes (U+2018, U+2019, U+201C, U+201D) are rejected by
`./scripts/ascii-punctuation.sh check`, which runs on pre-push and in CI;
`./scripts/ascii-punctuation.sh fix` normalizes them. They are also the
main cause of failed exact-string replaces, since they look like ASCII in
the editor but do not match.

Finish every edit: a replace that changes only part of a sentence or
leaves duplicate or placeholder text is worse than no edit. Fix the whole
span in one go, or read the section and write it back.

Wrap prose in `README.md`, `docs/`, `AGENTS.md`, and skill files to 80
characters. The **markdown-conventions** skill has the full replace
strategy and the wrap rules.

## Keeping docs in sync

When you or the user make or implement a decision that changes
functionality - user-facing behaviour (flows, screens, routes, URIs), the
API contract (endpoints, request/response shape, query params), a product
or UX decision, or scope - update every artifact it affects:

| Artifact | When to update |
|----------|----------------|
| `docs/spec.md` | Behaviour, screens, data flow, frontend/backend contract |
| `docs/api.md` | REST endpoints, request/response, errors |
| `docs/api.http` | Runnable examples for any changed or new endpoint |
| `docs/roadmap.md` | New or changed planned work, deferred features |
| `README.md` | Setup, run instructions, high-level scope |
| Plan files (`docs/plans/`) | Route tables, phase scope, decisions, URIs |

When you finish implementing a planned roadmap task, mark it done in
`docs/roadmap.md` without the user having to ask (see the
**roadmap-maintenance** skill). If a decision is recorded in only one
place, such as a phase plan, add it to the main plan and the spec too.

## Frontend

- **bun only.** `bun install`, `bun run build`, `bun run lint`,
  `bun run test`, `bun run dev`, and `bun x` to run a CLI. Never npm or
  npx.
- **Fix the code rather than disabling ESLint.** Add `eslint-disable` or
  `eslint-disable-next-line` only when the rule genuinely cannot be
  satisfied, with a short comment saying why. Prefer line-level over
  file-level disables, and remove a disable when you touch a file and the
  code can be refactored to satisfy the rule.
