CREATE TABLE kitchen_cities (
  id VARCHAR PRIMARY KEY NOT NULL,
  name VARCHAR NOT NULL,
  state VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL,
  updated_at TIMESTAMP
);
