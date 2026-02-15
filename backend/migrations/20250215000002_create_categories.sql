CREATE TABLE IF NOT EXISTS categories (
    id         TEXT    NOT NULL PRIMARY KEY,
    parent_id  TEXT    REFERENCES categories(id),
    name       TEXT    NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER
);

-- Category name is unique per parent, among active (non-deleted) rows.
-- SQLite treats NULLs as distinct in UNIQUE, so we need two partial indexes.
CREATE UNIQUE INDEX idx_categories_name_root
    ON categories(name)
    WHERE parent_id IS NULL AND deleted_at IS NULL;

CREATE UNIQUE INDEX idx_categories_name_parent
    ON categories(parent_id, name)
    WHERE parent_id IS NOT NULL AND deleted_at IS NULL;
