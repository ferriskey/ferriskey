ALTER TABLE user_sessions DROP COLUMN IF EXISTS authenticated_at;
ALTER TABLE user_sessions DROP COLUMN IF EXISTS persistent;
ALTER TABLE auth_sessions DROP COLUMN IF EXISTS remember_me;
