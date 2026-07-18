ALTER TABLE realm_settings
    ADD COLUMN seawatch_pii_mode VARCHAR(20) NOT NULL DEFAULT 'off',
    ADD COLUMN seawatch_pseudo_key VARCHAR(255);
