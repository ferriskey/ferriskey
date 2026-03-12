ALTER TABLE realm_settings
    ADD COLUMN access_token_lifetime_secs INTEGER NOT NULL DEFAULT 300,
    ADD COLUMN refresh_token_lifetime_secs INTEGER NOT NULL DEFAULT 86400,
    ADD COLUMN id_token_lifetime_secs INTEGER NOT NULL DEFAULT 300,
    ADD COLUMN temporary_token_lifetime_secs INTEGER NOT NULL DEFAULT 300;

ALTER TABLE clients
    ADD COLUMN access_token_lifetime_secs INTEGER DEFAULT NULL,
    ADD COLUMN refresh_token_lifetime_secs INTEGER DEFAULT NULL,
    ADD COLUMN id_token_lifetime_secs INTEGER DEFAULT NULL,
    ADD COLUMN temporary_token_lifetime_secs INTEGER DEFAULT NULL;
