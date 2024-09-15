CREATE TABLE password_reset (
    id VARCHAR PRIMARY KEY NOT NULL,
    code VARCHAR NOT NULL,
    hash_proof VARCHAR NOT NULL UNIQUE,
    user_id VARCHAR NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP
);