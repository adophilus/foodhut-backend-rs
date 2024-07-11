CREATE TABLE orders (
  id VARCHAR PRIMARY KEY,
  cart_id VARCHAR NOT NULL,
  status VARCHAR NOT NULL,
  sub_total NUMERIC NOT NULL,
  delivery_fee NUMERIC NOT NULL,
  service_fee NUMERIC NOT NULL,
  payment_method VARCHAR NOT NULL,
  total NUMERIC NOT NULL,
  delivery_address VARCHAR NOT NULL,
  dispatch_rider_note VARCHAR NOT NULL,
  owner_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);

