DELETE FROM sessions;

ALTER TABLE sessions
ADD COLUMN access_token VARCHAR NOT NULL UNIQUE;
