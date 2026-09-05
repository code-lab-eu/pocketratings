---
name: frontend-styling
description: The Pocket Ratings design system - pr-* design tokens, pr-* semantic utility classes, shared Svelte components, dark mode, and focus rings. Use before writing or changing any markup or CSS under frontend/src/, including layout.css, and before picking colors, typography, spacing, or Tailwind classes for the UI.
---

# Frontend styling

`frontend/STYLES.md` is the authority for this design system. Read it
before touching UI, and update it when you add a utility or component.

The implementation lives in `frontend/src/routes/layout.css`: design
tokens in `:root` and `html.dark`, semantic utilities in
`@layer utilities`, and component classes in `@layer components`.

## Rules

- **Never hardcode a color.** Use the `--pr-*` tokens (`--pr-surface`,
  `--pr-text-primary`, `--pr-text-muted`, `--pr-border`,
  `--pr-primary-500`, and so on). The palette is "Warm Amber & Charcoal".
- **Dark mode is the `.dark` class on `<html>`**, wired up by
  `@custom-variant dark (&:where(.dark, .dark *))`. There is no
  `data-theme` attribute. Any new token needs a value in both `:root` and
  `html.dark`.
- **Prefer the `.pr-*` semantic utilities over raw Tailwind color and
  typography classes.** Use `.pr-text-body`, `.pr-text-muted`,
  `.pr-text-subtle`, `.pr-text-label`, `.pr-input`, `.pr-card`,
  `.pr-panel`, `.pr-link-inline`, `.pr-link-muted` rather than
  `text-gray-*`. Layout utilities (spacing, flex, grid, widths) stay
  inline as Tailwind classes.
- **Prefer the shared components over ad-hoc markup**: `lib/Button.svelte`
  (variants `primary`, `secondary`, `link`), `lib/PageHeading.svelte` for
  page titles, `lib/SectionHeading.svelte` for section titles. Check
  `frontend/src/lib/` before writing a new one - there are also
  `InputField`, `Select`, `TextareaField`, `EmptyState`, `FormError`,
  `BackLink`, `Breadcrumb`, and more.
- **Focus rings use `:focus-visible`** with `--pr-focus-ring-color`,
  `--pr-focus-ring-width`, and `--pr-focus-ring-offset`, so keyboard users
  get a visible indicator (WCAG 2.4.7) and mouse users do not see an
  outline on click. Every new interactive surface needs one.
- **Honour `prefers-reduced-motion: reduce`.** Transitions and animations
  are disabled in the media block at the bottom of `layout.css`; add new
  ones there too.

## Tests

Styling-only changes need no new tests: do not write tests that assert CSS,
tokens, font sizes, or appearance-only class names. The existing suite must
still pass. Behaviour changes still follow the test-first rule in
AGENTS.md - see the **frontend-development** skill.
