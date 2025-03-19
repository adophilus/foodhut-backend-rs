ALTER TABLE transactions
ADD COLUMN purpose JSONB NOT NULL DEFAULT '{"type":"other"}'::JSONB;

ALTER TABLE transactions
ALTER COLUMN purpose
DROP DEFAULT;
