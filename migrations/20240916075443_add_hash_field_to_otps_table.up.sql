CREATE EXTENSION IF NOT EXISTS pgcrypto;

ALTER TABLE otps
ADD COLUMN hash VARCHAR(64) UNIQUE;

DELETE FROM otps;

ALTER TABLE otps
ALTER COLUMN hash SET NOT NULL;