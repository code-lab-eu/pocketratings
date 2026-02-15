CREATE TABLE IF NOT EXISTS locations (
    id         TEXT    NOT NULL PRIMARY KEY,
    name       TEXT    NOT NULL,
    deleted_at INTEGER
);
