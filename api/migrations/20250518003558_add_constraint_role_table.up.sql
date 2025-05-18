-- Add up migration script here

-- Unique (name, realm_id) constraint
ALTER TABLE roles
ADD CONSTRAINT unique_name_realm_id UNIQUE (name, realm_id);