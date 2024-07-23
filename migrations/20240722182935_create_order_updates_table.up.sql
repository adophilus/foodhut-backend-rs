CREATE TABLE order_updates (
  id SERIAL PRIMARY KEY NOT NULL,
  status VARCHAR NOT NULL,
  order_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);
