ALTER TABLE realm_settings
ADD COLUMN IF NOT EXISTS login_aliases TEXT[] NOT NULL DEFAULT '{username}';
