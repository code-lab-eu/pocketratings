-- Product variations: one per product; purchases reference variation_id.
-- 1. Create product_variations table.
-- 2. Backfill one default variation per product (label '', unit 'none').
-- 3. Add nullable variation_id to purchases, backfill, then recreate purchases with NOT NULL.

CREATE TABLE IF NOT EXISTS product_variations (
    id          TEXT    NOT NULL PRIMARY KEY,
    product_id  TEXT    NOT NULL REFERENCES products(id),
    label       TEXT    NOT NULL,
    unit        TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER
);

-- One default variation per product (use product.updated_at for created_at/updated_at).
INSERT INTO product_variations (id, product_id, label, unit, created_at, updated_at, deleted_at)
SELECT
    lower(
        hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)), 2)
        || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2)
        || '-' || hex(randomblob(6))
    ),
    id,
    '',
    'none',
    updated_at,
    updated_at,
    NULL
FROM products;

-- Add variation_id to purchases (nullable first).
ALTER TABLE purchases ADD COLUMN variation_id TEXT REFERENCES product_variations(id);

-- Backfill: set variation_id to the product's single variation (one per product from step 2).
UPDATE purchases
SET variation_id = (
    SELECT id FROM product_variations pv
    WHERE pv.product_id = purchases.product_id
    ORDER BY pv.created_at
    LIMIT 1
);

-- Recreate purchases so variation_id is NOT NULL (SQLite cannot alter column to NOT NULL).
CREATE TABLE purchases_new (
    id           TEXT    NOT NULL PRIMARY KEY,
    user_id      TEXT    NOT NULL REFERENCES users(id),
    product_id   TEXT    NOT NULL REFERENCES products(id),
    location_id  TEXT    NOT NULL REFERENCES locations(id),
    quantity     INTEGER NOT NULL DEFAULT 1,
    price        TEXT    NOT NULL,
    purchased_at INTEGER NOT NULL,
    deleted_at   INTEGER,
    variation_id TEXT    NOT NULL REFERENCES product_variations(id)
);

INSERT INTO purchases_new
SELECT id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at, variation_id
FROM purchases;

DROP TABLE purchases;

ALTER TABLE purchases_new RENAME TO purchases;
