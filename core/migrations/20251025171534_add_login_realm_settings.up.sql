-- Add up migration script here
ALTER TABLE realm_settings
ADD COLUMN IF NOT EXISTS user_registration_enabled BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN IF NOT EXISTS forgot_password_enabled BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN IF NOT EXISTS remember_me_enabled BOOLEAN NOT NULL DEFAULT false;
