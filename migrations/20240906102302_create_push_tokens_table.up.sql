CREATE TABLE push_tokens (
    id VARCHAR PRIMARY KEY NOT NULL,
    token VARCHAR NOT NULL,
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP
);