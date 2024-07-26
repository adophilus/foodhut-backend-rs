CREATE TABLE transactions (
  id VARCHAR PRIMARY KEY NOT NULL,
  amount NUMERIC NOT NULL,
  note VARCHAR,
  wallet_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);
