CREATE TABLE users (
  id VARCHAR PRIMARY KEY,
  email VARCHAR UNIQUE NOT NULL,
  phone_number VARCHAR UNIQUE NOT NULL,
  is_verified BOOLEAN NOT NULL,
  first_name VARCHAR NOT NULL,
  last_name VARCHAR NOT NULL,
  profile_picture JSON,
  birthday DATE NOT NULL,
  referral_code VARCHAR,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);
