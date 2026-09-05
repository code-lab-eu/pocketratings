---
name: roadmap-maintenance
description: Conventions and workflows for docs/roadmap.md. Use when editing docs/roadmap.md - adding a task, updating a task, marking a task DONE after finishing planned work - and when the user asks for roadmap maintenance, cleanup, prioritization, or an alignment audit.
---

# Roadmap maintenance

Apply this workflow when adding to, updating, maintaining, or auditing
`docs/roadmap.md`.

## Quick edits

For adding or updating single tasks. Do not run the full cleanup workflow
below for these.

- **New task:** Add at the **end** of the Planned list. Use the **next
  sequential number** (e.g. if the last task is 14, the new task is
  `### 15.`). The title must end with `[FE]`, `[BE]`, or `[FE+BE]`. The
  first line of the task body must be `**N sp.**` (rough estimate, 1-5)
  followed by the goal or first sentence. Do not reorder.
- **Mark task complete:** Append ` — DONE` to that task's title (em dash,
  matching the existing entries). Do not
  remove it; removal happens during cleanup. Do this without the user
  having to ask whenever you finish implementing a planned roadmap task.
- **Update an existing task:** Change goal, tasks, or SP as needed; keep
  the title label and SP format. If the scope changes, update
  `[FE]`/`[BE]`/`[FE+BE]`.
- **Leave alone:** the list order, DONE items, and the "Distant future"
  section.

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

Follow the **markdown-conventions** skill (80-char wrap, ASCII punctuation).
