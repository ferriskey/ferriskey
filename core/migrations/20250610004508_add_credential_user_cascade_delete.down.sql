-- Add down migration script here

ALTER TABLE credentials
    DROP CONSTRAINT IF EXISTS credentials_user_id_fkey;

ALTER TABLE credentials
    ADD CONSTRAINT credentials_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id);