ALTER TABLE transactions
ADD COLUMN type VARCHAR;

UPDATE transactions
SET type = 'CREDIT';

ALTER TABLE transactions
ALTER COLUMN type SET NOT NULL;