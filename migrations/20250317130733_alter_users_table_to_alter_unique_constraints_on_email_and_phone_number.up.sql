ALTER TABLE users
DROP CONSTRAINT users_email_key; 

ALTER TABLE users
DROP CONSTRAINT users_phone_number_key;

CREATE UNIQUE INDEX unique_email_if_not_deleted
ON users(email)
WHERE is_deleted = FALSE;

CREATE UNIQUE INDEX unique_phone_number_if_not_deleted
ON users(phone_number)
WHERE is_deleted = FALSE;
