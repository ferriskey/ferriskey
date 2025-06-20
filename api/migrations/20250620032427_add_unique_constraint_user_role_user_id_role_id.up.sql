-- Add up migration script here
ALTER TABLE user_role
ADD CONSTRAINT unique_user_id_role_id UNIQUE (user_id, role_id);