-- Add up migration script here
ALTER TABLE orders
DROP column cart_id;
