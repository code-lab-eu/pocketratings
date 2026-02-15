CREATE TABLE IF NOT EXISTS products (
    id          TEXT    NOT NULL PRIMARY KEY,
    category_id TEXT    NOT NULL REFERENCES categories(id),
    brand       TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER
);
