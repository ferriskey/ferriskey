-- Add down migration script here
ALTER TABLE user_role
DROP CONSTRAINT IF EXISTS unique_user_id_role_id;