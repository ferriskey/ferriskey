-- Add down migration script here
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_client_id_fkey;

ALTER TABLE users ADD CONSTRAINT users_client_id_fkey
FOREIGN KEY (client_id) REFERENCES clients(id);