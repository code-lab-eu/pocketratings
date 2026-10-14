---
name: backend-dependency-maintenance
description: >-
  Audit and update Pocket Ratings backend Rust dependencies, remove unnecessary
  crates, reduce enabled features, and identify upstream fixes for duplicate
  transitive versions. Use for backend dependency maintenance or Cargo dependency
  cleanup. Excludes Rust edition management.
---

# Backend Dependency Maintenance

Maintain `backend/Cargo.toml` and `backend/Cargo.lock` together. For an
audit-only request, report findings without changing dependencies. For an update
request, carry the authorized changes through verification. Rust edition
management is outside this skill's scope.

## Establish the baseline

Inspect the working tree, manifest, lockfile, backend source and tests, build
scripts, Dockerfile, and CI configuration. Preserve unrelated changes. Record
resolved versions separately from manifest version requirements.

Read `docs/backend-dependencies.md` first: it records deliberate version holds,
retained duplicate versions, and upstream issues and pull requests already
found. Re-check its linked tickets instead of repeating the search.

Run Cargo commands from `backend/`. Start with:

```bash
cargo tree --locked --depth 1
cargo tree --locked --duplicates
cargo tree --locked -e features
cargo update --dry-run
```

Check current upstream releases as well: the update preview is constrained by
existing requirements and does not reveal every available upgrade. Use
`cargo outdated` when available, or consult crates.io and upstream release
notes. Record the current and proposed versions, breaking changes, affected call
sites, and compatibility constraints for each candidate. Check minimum Rust
version requirements against the compilers used locally, in CI, and in Docker;
report incompatibilities rather than bypassing them with
`--ignore-rust-version`.

Run `cargo audit` against the lockfile and inspect advisory details. Use
existing `cargo deny` policy if the project has one. Report unavailable checks
explicitly; missing tools or stale advisory data do not establish a clean
result. Check tool availability before using optional Cargo subcommands and
follow environment permissions when installation or network access is needed.

## Find dependencies to remove

Use `cargo machete` to identify candidates, then verify each against source,
tests, macros, generated code, build scripts, renamed crates, and
target-specific usage. Its findings can contain false positives. Move a
dependency to `dev-dependencies` only when production code and build scripts do
not need it.

Also look for functionality now provided by the standard library or an existing
dependency. Compare semantics and maintenance cost before replacing a crate;
avoid reimplementing substantial library functionality merely to shrink the
dependency count. Propose changes to established implementation patterns before
making them, as required by `AGENTS.md`.

For transitive dependencies, trace the parent with `cargo tree -i <crate>` and
change the responsible parent version or feature. Do not manually delete
lockfile entries or assume removing a direct dependency removes it from the
resolved graph.

## Investigate duplicate transitive versions

For each duplicate from the baseline tree, trace every version separately with
`cargo tree --locked -i <crate>@<version>`. Inspect the resolved parents'
manifests for their actual version requirements, features, and dependency kinds.
Record the paths from Pocket Ratings to those parents. Distinguish production
dependencies from build, test, and target-specific dependencies; lockfile
presence alone does not establish that both versions ship in the production
binary.

Classify the cause: an older lockfile selection, an outdated parent with an
available upgrade, a restrictive parent requirement, or genuinely incompatible
API generations. Try an ordinary compatible update or released parent upgrade
first. Multiple versions can be legitimate; do not make zero duplicates a goal
that overrides compatibility or maintainability.

When a parent constraint blocks consolidation, check its current upstream
branch, releases, issues, and pull requests for an existing fix. Identify
whether a version update, a wider compatible range, or a code migration is
needed. Default caret requirements already allow compatible updates; increasing
their minimum alone does not widen the range. Do not extend a range across
incompatible versions without evidence that the parent supports them. Consider
public types and traits, feature changes, minimum Rust requirements, and
intentional pins such as matched procedural macro and runtime versions.

Report promising upstream contributions with the parent repository, exact
blocking requirement, proposed change, compatibility evidence, existing upstream
work, and expected benefit to this project's graph. Rank by practical impact,
including production use, advisories, and maintenance cost. Identify any other
parents that would still block consolidation.

When contribution preparation is requested, work in a separate upstream checkout
and follow its contribution guidelines. Test the parent at representative old
and new supported dependency versions, relevant features, and its supported
minimum Rust version. Verify integration in an isolated Pocket Ratings checkout;
a temporary path or git patch can test the modified parent. Cargo patches still
must satisfy version requirements. Keep temporary overrides out of the final
project change unless explicitly requested. Prepare a focused patch and PR text
with test results and before/after dependency paths. Publish a PR, issue, or
comment only when the user has explicitly authorized that external action.

## Reduce dependency features

Read each candidate crate's feature documentation for the version being used.
Match explicit and default features to actual application needs. Consider
`default-features = false` with an explicit feature list where justified.

Trace who enables a feature with `cargo tree -e features -i <crate>`. Features
are additive: another dependency can enable a feature disabled on our direct
edge. The root package's `--no-default-features` flag does not disable every
dependency's defaults. Inspect production and test graphs separately, using
`cargo tree -e normal,build` for production dependency analysis.

Check runtime effects as well as compilation: serialization formats, TLS,
database drivers and embedded migrations, CLI behavior, and runtime services can
depend on features. A successful test build can mask features supplied by dev
dependencies. Verify the production release build separately. Reinspect the
resolved feature graph after changes; report actual reductions, including
features that remain enabled through another path.

## Apply and verify

Prioritize relevant security fixes. Group compatible updates where practical;
isolate breaking upgrades and dependency replacements so failures can be traced.
Treat pre-1.0 version changes according to Cargo's compatibility rules. Read
upstream migration notes and search all usage sites before adapting code.

Use `cargo update -p <crate>` for updates within existing requirements. For
upgrades outside them, edit the manifest requirement and resolve the lockfile
with Cargo. Review transitive changes and duplicates as well as direct versions;
avoid unrelated lockfile churn in a targeted update.

Follow the `rust-backend-workflow` skill for code changes. Write a failing
regression test before changing behavior or adapting behavior-sensitive usage.
Existing tests can verify manifest-only changes that preserve behavior; do not
add tests that merely assert dependency versions or manifest text.

Review every changed file for duplication, complexity, proportionality, and
carried-forward problems before QC. Update affected docs when behavior,
configuration, or setup changes. For each completed phase, run the full
`backend-quality-control` skill, including format, strict Clippy, tests, and
coverage. Also run `cargo build --release --locked` from `backend/` and
`./scripts/ascii-punctuation.sh check` from the repository root. Rebuild the
backend container when changes affect native libraries or container build
compatibility. A compile-only check is insufficient to complete an update.

Update `docs/backend-dependencies.md` with every hold, retained duplicate, and
upstream ticket this pass found or resolved, including the check date and the
ticket status. Remove entries that no longer apply.

Report updated versions, removed or relocated dependencies, feature reductions,
duplicate versions resolved or retained with reasons, upstream contribution
candidates, remaining advisories, and verification results. Include deferred
work and distinguish measured graph or build improvements from expected ones.

## References

Consult these sources for command behavior and feature analysis:

- [Cargo update](https://doc.rust-lang.org/cargo/commands/cargo-update.html).
- [Cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html).
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html).
- [cargo-machete](https://github.com/bnjbvr/cargo-machete).
- [RustSec](https://rustsec.org/).
- [Version
  requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html).
- [Dependency
  resolution](https://doc.rust-lang.org/cargo/reference/resolver.html).
- [Dependency
  overrides](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html).
