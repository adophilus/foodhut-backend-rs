CREATE TABLE carts (
  id VARCHAR PRIMARY KEY,
  items JSON NOT NULL,
  status VARCHAR NOT NULL,
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);

