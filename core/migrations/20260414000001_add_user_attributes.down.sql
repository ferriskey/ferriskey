-- Add down migration script here
DROP INDEX IF EXISTS idx_user_attributes_realm_key;
DROP INDEX IF EXISTS idx_user_attributes_user_id;
DROP TABLE IF EXISTS user_attributes;
