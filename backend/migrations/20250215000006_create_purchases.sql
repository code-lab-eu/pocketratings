CREATE TABLE IF NOT EXISTS purchases (
    id           TEXT    NOT NULL PRIMARY KEY,
    user_id      TEXT    NOT NULL REFERENCES users(id),
    product_id   TEXT    NOT NULL REFERENCES products(id),
    location_id  TEXT    NOT NULL REFERENCES locations(id),
    quantity     INTEGER NOT NULL DEFAULT 1,
    price        TEXT    NOT NULL,
    purchased_at INTEGER NOT NULL,
    deleted_at   INTEGER
);
