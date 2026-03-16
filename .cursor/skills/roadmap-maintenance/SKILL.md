---
name: roadmap-maintenance
description: Maintains docs/roadmap.md. Covers cleanup (remove finished work, reorder, label, estimate), and auditing (validate pending tasks against spec and architecture, fix the roadmap). Use when the user asks for roadmap maintenance, prioritization, or alignment audit.
---

# Roadmap maintenance

Apply this workflow when the user asks to update, maintain, or audit the
roadmap.

## Cleanup

1. **Remove finished work** — Delete any Planned item marked " — DONE" (or
   similar). Remove the entire "Completed" section if present. Do not move
   finished tasks to another section.

2. **Remove references** — In remaining items, delete or shorten references
   (e.g. "Blocked by", "Ties to") that point to completed work.

3. **Ensure story point estimates** — Every item must have an SP estimate in
   the **first line of the body**, e.g. `**2 sp.**`. Scale: 1 = tiny, 2 =
   small (half day), 3 = medium (1-2 days), 4 = large (week), 5 = very
   large.

4. **Label scope** — Every title ends with [FE], [BE], or [FE+BE].

5. **Identify blocking** — Check if any task depends on another. Rank
   blockers by how many tasks they unblock.

6. **Reorder** — (1) Blocking tasks; (2) tasks marked **Important**;
   (3) low-hanging fruit (1-2 SP); (4) remaining. Renumber sequentially.
   Keep "Distant future" unchanged.

## Audit

When the user asks for an audit, validate that each pending roadmap task
aligns with the project's spec ([docs/spec.md](docs/spec.md)), API design
([docs/api.md](docs/api.md)), and established codebase patterns. The goal is
to catch task descriptions that would lead an implementer to introduce
changes that deviate from the overall vision.

For each pending task, check:

- **API design consistency** — Does the proposed API change follow the
  existing conventions? (e.g. optional query params with no conditional
  defaults; consistent error codes; documented in api.md.)
- **Frontend patterns** — Does it match the design system, data-flow
  patterns, and component conventions already in use?
- **Scope and audience** — Does it stay within the product's scope (single
  user or family; no multi-tenant; EUR only; etc.)?
- **Accurate references** — Do function names, endpoint paths, and
  component names in the task description match what actually exists in the
  codebase?
- **Dependencies** — Does the task depend on or overlap with another task
  in a way that should be noted (e.g. "do X before Y")?

Fix any issues directly in the roadmap: correct inaccurate descriptions,
add clarifications, note ordering constraints. Do not create a separate
audit report file.

## Conventions

- **FE** = frontend only; **BE** = backend/CLI/DB only; **FE+BE** = both.
- Story points: 1-5 scale; low-hanging fruit = 1-2 SP.
- Optional marker in task body: **Important** (for reorder group 2).

## Doc rule

Follow [readme-docs-line-width](.cursor/rules/readme-docs-line-width.mdc)
(80-char wrap in docs).
