CREATE TABLE wallets (
  id VARCHAR PRIMARY KEY NOT NULL,
  balance NUMERIC NOT NULL,
  metadata JSON NOT NULL,
  owner_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);
