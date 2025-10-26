-- Add down migration script here
ALTER TABLE realm_settings
DROP COLUMN IF EXISTS user_registration_enabled,
DROP COLUMN IF EXISTS forgot_password_enabled,
DROP COLUMN IF EXISTS remember_me_enabled;
