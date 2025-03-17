DROP INDEX unique_phone_number_if_not_deleted;

DROP INDEX unique_email_if_not_deleted;

ALTER TABLE users
ADD CONSTRAINT users_phone_number_key UNIQUE(phone_number);

ALTER TABLE users
ADD CONSTRAINT users_email_key UNIQUE(email); 

