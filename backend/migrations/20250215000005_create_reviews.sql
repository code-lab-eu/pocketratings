CREATE TABLE IF NOT EXISTS reviews (
    id         TEXT    NOT NULL PRIMARY KEY,
    product_id TEXT    NOT NULL REFERENCES products(id),
    user_id    TEXT    NOT NULL REFERENCES users(id),
    rating     TEXT    NOT NULL,
    text       TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER
);
