ALTER TABLE order_items
ADD COLUMN owner_id VARCHAR;

UPDATE order_items
SET owner_id = orders.owner_id
FROM orders
WHERE order_items.order_id = orders.id;

ALTER TABLE order_items
ALTER COLUMN owner_id SET NOT NULL;