CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id         TEXT    NOT NULL PRIMARY KEY,
    user_id    TEXT    NOT NULL REFERENCES users(id),
    token_hash TEXT    NOT NULL UNIQUE,
    expires_at INTEGER NOT NULL,
    used_at    INTEGER,
    created_at INTEGER NOT NULL
);
