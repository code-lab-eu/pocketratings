# Backend dependency notes

Findings from backend dependency maintenance that a future audit would otherwise
have to rediscover: deliberate version holds, retained duplicate crates, and
upstream issues or contributions worth following. Maintained by the
**backend-dependency-maintenance** skill. Update the entries below at the end of
every maintenance pass: re-check each linked ticket, then change its status or
remove the entry once it no longer applies.

Last reviewed: 2026-10-11 (after the sqlx 0.9 and argon2 0.6 upgrades).

## Deliberate holds

### jsonwebtoken stays on 9.x

- **Current:** 9.3.1, which uses `ring` as its crypto backend.
- **Why not 10 or 11:** since 10.0 the crate requires choosing a backend
  feature. `rust_crypto` unconditionally enables `rsa`, `p256`, `p384`,
  `ed25519-dalek`, `hmac`, `sha2` and `rand`, and `rsa` 0.9.x carries
  RUSTSEC-2023-0071 (Marvin timing attack, no fix). `aws_lc_rs` replaces one C
  and assembly backend with another and has no benefit for this project, which
  only uses HS256.
- **Revisit when:** `rust_crypto` no longer depends on a vulnerable `rsa`, or
  can be built without RSA.
- **Upstream (Keats/jsonwebtoken), checked 2026-10-11:**
  - [#495](https://github.com/Keats/jsonwebtoken/issues/495) (open): bump `rsa`
    to 0.10.
  - [#502](https://github.com/Keats/jsonwebtoken/pull/502) (open): moves the
    `rust_crypto` RSA backend to `rsa` 0.10.0-rc.17. Not verified whether `rsa`
    0.10 fixes the Marvin advisory.
  - [#536](https://github.com/Keats/jsonwebtoken/issues/536) (open): update the
    RustCrypto dependency stack.
- **Possible contribution:** split `rust_crypto` into per-algorithm-family
  features so an HMAC-only build avoids `rsa`. No existing issue or PR proposes
  this. Open an issue first to ask the maintainer. Do not post without the
  user's authorization.
- **Effect of staying:** `getrandom` 0.2 remains in the graph via `ring` (see
  the duplicates below).

## Retained duplicate versions

Production and build graph (`cargo tree --locked --duplicates -e normal,build
--target all`), checked 2026-10-11.

| Crate | Versions | Cause | Resolution |
|-------|----------|-------|------------|
| `getrandom` | 0.2, 0.4 | 0.2 via `ring` (jsonwebtoken 9) | Goes away when jsonwebtoken moves off `ring` |
| `sha2`, `digest`, `crypto-common`, `block-buffer`, `cpufeatures` | 0.10 and 0.11 | `sqlx-core` 0.9 requires `sha2` 0.10; the backend uses 0.11 | Upstream sqlx, see below |
| `hashbrown` | 0.16, 0.17 | `sqlx-core` 0.9 via `hashlink` and `indexmap` | Wait for sqlx or hashlink release |
| `syn` | 2, 3 | Proc-macro crates are mid-migration to `syn` 3 (build-time only) | Resolves as the ecosystem moves |

### sqlx-core and sha2 0.10

- **Cause:** `sqlx-core` 0.9.0 pins `sha2` 0.10 for migration checksums.
- **Upstream (transact-rs/sqlx, renamed from launchbadge/sqlx), checked
  2026-10-11:**
  - [#4418](https://github.com/transact-rs/sqlx/issues/4418) (open): make
    migration and checksum support fully opt-in. Related, but does not by itself
    move `sha2` to 0.11.
  - [#4288](https://github.com/transact-rs/sqlx/issues/4288) (closed):
    `sqlx-core` depends on a different `sha2` version than the workspace. Not
    verified whether it covers 0.11.
- **Possible contribution:** bump `sqlx-core` to `sha2` 0.11, if upstream has
  not already done so. Search open pull requests before starting.

## Advisories

`cargo audit` was clean on 2026-10-11 (269 crates, no vulnerabilities or
warnings). Re-run it every pass: advisories appear against unchanged lockfiles.

## Feature choices

- **sqlx:** `default-features = false` with `runtime-tokio`, `sqlite-bundled`,
  `migrate` and `macros`. `macros` is required by `sqlx::migrate!`. The code
  does not use the `any` driver, JSON support, SQLite load extensions,
  deserialization or unlock-notify.
- **argon2:** default features. `getrandom` supplies the salt, so the backend
  generates no salt itself.
- **Random bytes:** reset tokens use `getrandom::fill` directly. `rand_core`
  0.10 has no operating-system generator of its own.
