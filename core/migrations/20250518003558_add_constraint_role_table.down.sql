-- Add down migration script here

ALTER TABLE roles
DROP CONSTRAINT unique_name_realm_id;
