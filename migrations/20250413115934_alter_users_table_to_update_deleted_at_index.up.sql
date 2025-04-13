DROP INDEX IF EXISTS unique_email_if_not_deleted;
DROP INDEX IF EXISTS unique_phone_number_if_not_deleted;

CREATE UNIQUE INDEX unique_email_if_not_deleted
ON users(email)
WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX unique_phone_number_if_not_deleted
ON users(phone_number)
WHERE deleted_at IS NULL;
