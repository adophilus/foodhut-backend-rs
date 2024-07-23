CREATE TABLE kitchens (
  id VARCHAR PRIMARY KEY,
  name VARCHAR NOT NULL,
  type VARCHAR NOT NULL,
  address VARCHAR NOT NULL,
  phone_number VARCHAR NOT NULL,
  opening_time VARCHAR NOT NULL,
  closing_time VARCHAR NOT NULL,
  preparation_time VARCHAR NOT NULL,
  delivery_time VARCHAR NOT NULL,
  cover_image JSON,
  rating NUMERIC NOT NULL,
  owner_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);
