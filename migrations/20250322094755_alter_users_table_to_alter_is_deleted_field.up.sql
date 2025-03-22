ALTER TABLE
  users
ADD COLUMN
  deleted_at TIMESTAMP;

UPDATE
  users
SET
  deleted_at = NOW()
WHERE
  is_deleted = TRUE;

ALTER TABLE
users
DROP COLUMN
  is_deleted;
