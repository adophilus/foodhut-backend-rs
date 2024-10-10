ALTER TABLE order_items ADD COLUMN temp_id SERIAL;

ALTER TABLE order_items DROP COLUMN id;

ALTER TABLE order_items RENAME COLUMN temp_id TO id;

ALTER TABLE order_items ADD PRIMARY KEY (id);