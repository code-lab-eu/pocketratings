---
name: roadmap-maintenance
description: Maintains docs/roadmap.md: removes finished tasks and the Completed section entirely, removes references to finished work, ensures every item has a story point (SP) estimate, identifies blocking tasks, labels tasks with [FE]/[BE]/[FE+BE], and reorders by blocking then important then low-hanging fruit (1–2 SP). Use when updating the roadmap, cleaning completed work, or when the user asks for roadmap maintenance or prioritization.
---

# Roadmap maintenance

Apply this workflow when the user asks to update or maintain the roadmap, remove
finished items, reorder, add FE/BE labels, or estimate story points.

## Steps

1. **Remove finished work** — Remove the entire "Completed" section from the
   roadmap. Delete any Planned item that is marked with " — DONE" or "Done" (or
   similar) or clearly completed; do not move them to another section (finished
   tasks are removed completely).

2. **Remove references** — In remaining Planned items, delete or shorten
   "Blocked by"/"Ties to" (or similar) that point to completed work.

3. **Ensure story point estimates** — Every Planned item must have an SP
   estimate. Put it in the **first line of the task body**, e.g. `**2 sp.**`
   then the goal text. Scale: 1 = tiny (hours), 2 = small (half day), 3 =
   medium (1–2 days), 4 = large (week), 5 = very large (multi-week). Add or
   correct estimates for any item that lacks one.

4. **Label scope** — For every Planned task, set [FE], [BE], or [FE+BE] in the
   task title from Goal/Tasks (frontend-only, backend-only, or both).

5. **Identify blocking** — For each task, check if another Planned task depends
   on it (e.g. "reusable search" before "search by category"). List "A blocks B"
   and rank blockers by how many tasks they unblock.

6. **Reorder** — Planned order: (1) Blocking tasks, ranked by number of tasks
   they block; (2) Tasks marked **Important**; (3) **Low-hanging fruit** (items
   with **1 or 2 SP**); (4) Remaining. Then **renumber all Planned tasks
   sequentially** (1, 2, 3, …) so there are no gaps. Keep "Distant future"
   unchanged.

## Conventions

- **FE** = only frontend changes; **BE** = only backend/CLI/DB; **FE+BE** = both.
- Story points: 1–5 scale; low-hanging fruit = 1–2 SP.
- Optional marker in task body: **Important** (for reorder group 2).

## Doc rule

Follow [readme-docs-line-width](.cursor/rules/readme-docs-line-width.mdc)
(80-char wrap in docs).
