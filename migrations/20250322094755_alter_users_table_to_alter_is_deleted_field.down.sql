ALTER TABLE
  users
ADD COLUMN
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE
  users
SET
  is_deleted = TRUE
WHERE
  deleted_at IS NOT NULL;

ALTER TABLE
users
DROP COLUMN
  deleted_at;
