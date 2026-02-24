## Pocket Ratings frontend styles

This project uses Tailwind via `src/routes/layout.css` plus a small layer of semantic utilities and Svelte components to keep styling consistent.

### Semantic utilities (`pr-*` classes)

Defined in `src/routes/layout.css`:

- **Text**
  - `.pr-text-body`: primary body text (`text-gray-900 dark:text-gray-50`).
  - `.pr-text-muted`: secondary text (`text-gray-600 dark:text-gray-200`), used for empty states and helper copy.
  - `.pr-text-subtle`: tertiary/metadata text (`text-gray-500 dark:text-gray-300`), e.g. dates or small notes.
  - `.pr-text-label`: form labels (`text-sm font-medium text-gray-700 dark:text-gray-200`).

- **Headings**
  - `.pr-heading-page`: page titles (was `mb-4 text-2xl font-semibold text-gray-900 dark:text-gray-50`).
  - `.pr-heading-section`: section headings (was `mb-3 text-lg font-medium text-gray-900 dark:text-gray-50`).

- **Links**
  - `.pr-link-inline`: inline link with underline, used in text paragraphs and “Back to …” links.
  - `.pr-link-muted`: muted navigation link (e.g. “← Manage”, “← Home”).

- **Buttons**
  - `.pr-btn-primary`: primary action button (dark background in light mode, inverted in dark mode).
  - `.pr-btn-secondary`: secondary/outline button used for “Cancel” actions and similar.

- **Cards and list items**
  - `.pr-card`: generic card/list-item container used for products, categories, locations, etc.
  - `.pr-list-item-link`: list row link (min height, underline on hover) used inside cards.

- **Form controls**
  - `.pr-input`: shared text/select/textarea input style with consistent border, radius, and focus ring in both light and dark mode.

Layout-related utilities (spacing, flex, grid, widths) stay inline in components and pages.

### Shared components

- `lib/Button.svelte`
  - Props:
    - `variant`: `'primary' | 'secondary' | 'link'` (default: `'primary'`).
    - `href` (optional): when provided, renders an `<a>`; otherwise renders a `<button>`.
    - `type`: `'button' | 'submit' | 'reset'` (default: `'button'`).
    - `disabled`: standard disabled flag for the `<button>` case.
  - Applies:
    - `pr-btn-primary` for `variant="primary"`.
    - `pr-btn-secondary` for `variant="secondary"`.
    - `pr-link-inline` for `variant="link"`.

- `lib/PageHeading.svelte`
  - Props:
    - `tag`: `'h1' | 'h2' | 'h3'` (default: `'h1'`).
  - Renders a heading with `pr-heading-page`.
  - Optional `description` slot is rendered below using `pr-text-muted`.

- `lib/SectionHeading.svelte`
  - Props:
    - `tag`: `'h2' | 'h3' | 'h4'` (default: `'h2'`).
  - Renders a heading with `pr-heading-section`.

### Usage guidelines

- **New pages and sections**
  - Use `PageHeading` for main page titles and `SectionHeading` for section titles instead of ad-hoc `<h1>`/`<h2>` classes.
  - Use `.pr-text-muted` and `.pr-text-subtle` for secondary/tertiary text rather than raw `text-gray-*` classes.

- **Buttons and links**
  - Prefer `Button` for primary and secondary actions.
  - Use `.pr-link-muted` for back-links in headers (e.g. “← Manage”) and `.pr-link-inline` for inline links in text.

- **Forms**
  - Use `.pr-text-label` for labels and `.pr-input` for all text inputs, selects, and textareas to keep forms consistent across routes.

When adding new UI, prefer these utilities/components over direct color/typography Tailwind classes so that light/dark mode and overall styling stay consistent.

