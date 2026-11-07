ALTER TABLE realm_settings
    DROP COLUMN IF EXISTS account_chooser_enabled,
    DROP COLUMN IF EXISTS account_chooser_show_email;
