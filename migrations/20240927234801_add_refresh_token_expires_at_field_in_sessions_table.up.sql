DELETE FROM sessions;

ALTER TABLE sessions
ADD COLUMN refresh_token_expires_at TIMESTAMP NOT NULL;
