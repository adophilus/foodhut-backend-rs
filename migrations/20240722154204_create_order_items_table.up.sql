CREATE TABLE order_items (
  id SERIAL PRIMARY KEY NOT NULL,
  status VARCHAR NOT NULL,
  price NUMERIC NOT NULL,
  meal_id VARCHAR NOT NULL,
  order_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);
