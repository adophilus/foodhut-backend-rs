ALTER TABLE transactions
ADD COLUMN ref VARCHAR NOT NULL DEFAULT GEN_RANDOM_UUID();

ALTER TABLE transactions
ALTER COLUMN ref DROP DEFAULT;
