ALTER TABLE realm_settings
    ADD COLUMN account_chooser_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN account_chooser_show_email BOOLEAN NOT NULL DEFAULT FALSE;
