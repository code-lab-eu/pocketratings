---
name: clippy-overrides
description: Policy for silencing Rust compiler or Clippy diagnostics in backend/. Use before adding or widening any #[allow(...)], #![allow(...)], or #[expect(...)] attribute in a .rs file, or when Clippy warnings block a build.
---

# Clippy overrides and `allow(...)` policy

## Default

Fix the underlying code (imports, ownership, lifetimes, error handling)
rather than silencing the diagnostic with `#[allow(...)]`,
`#![allow(...)]`, or `#[allow(clippy::...)]`.

## Approval requirement (strict)

Adding or expanding any `allow(...)` to bypass a warning or lint requires
**explicit developer approval**. If an override seems necessary, stop and
ask the developer before making the change.

## If approval is granted

- Keep the override **as narrow as possible**: a single item or module.
  Crate-level `#![allow]` is a last resort.
- Prefer `#[expect(...)]` where it applies, so new occurrences still fail
  CI.
- Put a short, specific rationale next to the override: which warning, why
  it is acceptable here, and the approval reference.

## Common cases to fix instead of overriding

- **`unused_imports`**: remove the unused import, import only what is
  needed, or use fully-qualified paths.
- **`doc_markdown`**: fix the doc formatting by wrapping identifiers in
  backticks.
- **`needless_pass_by_value`**: pass by reference when the value is not
  consumed.
