CREATE TABLE meals (
  id VARCHAR PRIMARY KEY,
  name VARCHAR NOT NULL,
  description VARCHAR NOT NULL,
  price NUMERIC NOT NULL,
  rating NUMERIC NOT NULL,
  tags JSON NOT NULL,
  cover_image_url VARCHAR NOT NULL,
  is_available BOOLEAN NOT NULL,
  kitchen_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);
