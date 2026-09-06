//! Password reset token persistence.
//!
//! Rows hold only the SHA-256 digest of a token (see [`crate::auth::reset_token`]), never the raw
//! value. Used and expired rows are kept; nothing in the app purges them.

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::auth::reset_token::TOKEN_TTL_SECONDS;
use crate::db::DbError;

/// Store a new reset token for a user, valid for [`TOKEN_TTL_SECONDS`] from `now`.
///
/// Existing tokens for the user are left untouched, so earlier links keep working until they
/// expire, are used, or the user's password changes (see [`invalidate_all_for_user`]).
///
/// # Errors
///
/// Returns [`DbError`] on query failure.
pub async fn create(
    pool: &SqlitePool,
    user_id: Uuid,
    token_hash: &str,
    now: i64,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, used_at, created_at) VALUES (?, ?, ?, ?, NULL, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id.to_string())
    .bind(token_hash)
    .bind(now + TOKEN_TTL_SECONDS)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Whether a token can still be used: it exists, is unused, is not expired, and belongs to an
/// active (not soft-deleted) user.
///
/// # Errors
///
/// Returns [`DbError`] on query failure.
pub async fn is_valid(pool: &SqlitePool, token_hash: &str, now: i64) -> Result<bool, DbError> {
    let row = sqlx::query(
        "SELECT t.id FROM password_reset_tokens t \
         JOIN users u ON u.id = t.user_id \
         WHERE t.token_hash = ? AND t.used_at IS NULL AND t.expires_at > ? AND u.deleted_at IS NULL",
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

/// Mark every outstanding (unused) reset token of a user as used at `now`.
///
/// Call this whenever the user's password changes, so an older link cannot undo the new
/// password. Tokens of other users are untouched.
///
/// # Errors
///
/// Returns [`DbError`] on query failure.
pub async fn invalidate_all_for_user(
    executor: impl sqlx::SqliteExecutor<'_>,
    user_id: Uuid,
    now: i64,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE password_reset_tokens SET used_at = ? WHERE user_id = ? AND used_at IS NULL",
    )
    .bind(now)
    .bind(user_id.to_string())
    .execute(executor)
    .await?;
    Ok(())
}

/// Consume a reset token and set the user's password, in one transaction.
///
/// Marking the token used and updating the password either both happen or neither does, so a
/// token can never be spent without the password changing. Any other outstanding link for the
/// same user is invalidated in the same transaction.
///
/// Returns `false` when the token is unknown, already used, or expired; the password is then
/// left unchanged.
///
/// # Errors
///
/// Returns [`DbError`] on query failure, or when the token belongs to a user that is no longer
/// active.
pub async fn consume_and_set_password(
    pool: &SqlitePool,
    token_hash: &str,
    password_hash: &str,
    now: i64,
) -> Result<bool, DbError> {
    let mut tx = pool.begin().await?;

    let marked = sqlx::query(
        "UPDATE password_reset_tokens SET used_at = ? WHERE token_hash = ? AND used_at IS NULL AND expires_at > ?",
    )
    .bind(now)
    .bind(token_hash)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    if marked.rows_affected() == 0 {
        tx.rollback().await?;
        return Ok(false);
    }

    let user_id: String =
        sqlx::query("SELECT user_id FROM password_reset_tokens WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_one(&mut *tx)
            .await?
            .get("user_id");
    let user_id = Uuid::parse_str(&user_id).map_err(|e| DbError::InvalidData(e.to_string()))?;

    crate::db::user::update_password(&mut *tx, user_id, password_hash).await?;
    invalidate_all_for_user(&mut *tx, user_id, now).await?;

    tx.commit().await?;
    Ok(true)
}
