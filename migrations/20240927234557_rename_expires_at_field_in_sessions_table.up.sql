DELETE FROM sessions;

ALTER TABLE sessions 
RENAME COLUMN expires_at TO access_token_expires_at;
