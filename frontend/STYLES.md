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
  - `.pr-heading-page`: page titles (`1.75rem` / semibold; evolved from Tailwind
    `text-2xl` and earlier `mb-4 text-2xl font-semibold text-gray-900 dark:text-gray-50`).
  - `.pr-heading-section`: section headings (was `mb-3 text-lg font-medium text-gray-900 dark:text-gray-50`).

- **Product detail**
  - `.pr-product-brand`: brand line under the product name (prominent sans, semibold;
    category navigation is via breadcrumb only).

- **Links**
  - `.pr-link-inline`: inline link with underline, used in text paragraphs and "Back to …" links.
    Has `cursor: pointer` and `:focus-visible` using the focus ring tokens.
  - `.pr-link-muted`: muted navigation link (e.g. "← Manage", "← Home"). Has `cursor: pointer`
    and `:focus-visible` using the focus ring tokens.

- **Focus ring (design tokens)**
  - Focus indicators are controlled by `--pr-focus-ring-color`, `--pr-focus-ring-width`, and
    `--pr-focus-ring-offset` in `:root` and `html.dark`. Buttons, links, and interactive cards
    use `:focus-visible` so keyboard users get a clear focus outline (WCAG 2.4.7); mouse users
    do not see the outline on click.

- **Buttons**
  - `.pr-btn-primary`: primary action button (dark background in light mode, inverted in dark
    mode). Has `cursor: pointer` and `:focus-visible`.
  - `.pr-btn-secondary`: secondary/outline button used for "Cancel" actions and similar. Has
    `cursor: pointer` and `:focus-visible`.
  - `.pr-btn-icon`: icon-sized or header-style button or icon link (transparent background, hover
    tint). Has a 44px minimum height and width (minimum touch target size, WCAG 2.5.5), `cursor:
    pointer`, and `:focus-visible`. Combine with `.pr-link-muted` or semantic colour classes as
    needed. Icon buttons get it from `IconButton`; icon links (the header Menu link, `EditLink`)
    use it directly.
  - All button classes use a short transition; motion is disabled when the user prefers
    reduced motion (`prefers-reduced-motion: reduce`).

- **Cards and list items**
  - `.pr-card`: generic card/list-item container used for products, categories, locations, etc.
    Has `cursor: pointer` and `:focus-visible` when used as a link or clickable surface. Cards
    use a short transition and a slight hover lift and shadow; motion is disabled when the
    user prefers reduced motion (`prefers-reduced-motion: reduce`).
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
    - `onclick` (optional): click handler for the `<button>` case (e.g. Cancel on an inline form).
  - Applies:
    - `pr-btn-primary` for `variant="primary"`.
    - `pr-btn-secondary` for `variant="secondary"`.
    - `pr-link-inline` for `variant="link"`.

- `lib/EditLink.svelte`
  - Props: `href` (already resolved by the caller), `label`.
  - Icon link with the lucide `Pen` icon, `pr-btn-icon` with `pr-link-muted` (which supplies the
    44px minimum tap target), and the accessible name "Edit {label}". Used for the edit action in
    `ManageListRow` and next to the product name on the product detail page.

- `lib/IconButton.svelte`
  - Props:
    - `label` (required): the accessible name (`aria-label`).
    - `onclick`, `title`, `disabled`.
    - `pressed` and `expanded` (optional): `aria-pressed` and `aria-expanded`; each attribute is
      omitted when the prop is undefined.
    - `tone`: `'muted' | 'danger'` (default: `'muted'`). `muted` uses `pr-link-muted`; `danger` is
      the red text for destructive actions such as delete.
    - `class` (optional): positioning only (absolute placement, margins).
    - `children`: the icon.
  - Always renders a `<button type="button">` with `pr-btn-icon`, so it has the 44px minimum tap
    target. Used for the theme toggle and Log out in the header, the `PasswordField` toggle, the
    delete action in `ManageListRow`, and the expand/collapse chevron in `CategoryLinkList`.

- `lib/InlineFormToggle.svelte`
  - Props: `label`, `headingId`, `open` (bindable), `onoutroend` (optional); the form is the child
    content.
  - Closed: a `pr-link-inline` button showing `label`. Open: a `pr-inline-form` panel that slides in
    and out (`inlineFormMotion`), headed by an `<h3 id={headingId}>` showing `label`. The child form
    uses `headingId` for `aria-labelledby`; `onoutroend` runs after the panel has closed.

- `lib/PageHeading.svelte`
  - Props:
    - `tag`: `'h1' | 'h2' | 'h3'` (default: `'h1'`).
  - Renders a heading with `pr-heading-page`.
  - Optional `description` slot is rendered below using `pr-text-muted`.

- `lib/PasswordField.svelte`
  - Props: `id`, `label`, `value` (bindable), `required`, `autocomplete`.
  - Password input (`.pr-input`) with an `IconButton` show/hide toggle inside the field: lucide
    `Eye`/`EyeOff`, `pressed` (`aria-pressed`), and an accessible name that flips between "Show
    password" and "Hide password".

- `lib/SectionHeading.svelte`
  - Props:
    - `tag`: `'h2' | 'h3' | 'h4'` (default: `'h2'`).
  - Renders a heading with `pr-heading-section`.

### Usage guidelines

- **New pages and sections**
  - Use `PageHeading` for main page titles and `SectionHeading` for section titles instead of ad-hoc `<h1>`/`<h2>` classes.
  - Use `.pr-text-muted` and `.pr-text-subtle` for secondary/tertiary text rather than raw `text-gray-*` classes.

- **Buttons and links**
  - Use `Button` for primary, secondary, and link-style actions, with `onclick` for actions on the
    page and `href` for navigation. Do not write `pr-btn-primary`, `pr-btn-secondary`, or
    `pr-link-inline` on a raw `<button>`.
  - Use `IconButton` for every icon-only button. Use `.pr-btn-icon` directly only on icon links,
    with `EditLink` for edit; never put `.pr-btn-icon` on a raw `<button>`.
  - Use `.pr-link-muted` for back-links in headers (e.g. "← Manage") and `.pr-link-inline` for inline links in text.

- **Forms**
  - Use `.pr-text-label` for labels and `.pr-input` for all text inputs, selects, and textareas to keep forms consistent across routes.

When adding new UI, prefer these utilities/components over direct color/typography Tailwind classes so that light/dark mode and overall styling stay consistent.

