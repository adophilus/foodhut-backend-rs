ALTER TABLE orders
ADD COLUMN owner_id VARCHAR;

UPDATE orders
SET owner_id = carts.owner_id
FROM carts
WHERE orders.cart_id = carts.id;

ALTER TABLE orders
ALTER COLUMN owner_id SET NOT NULL;