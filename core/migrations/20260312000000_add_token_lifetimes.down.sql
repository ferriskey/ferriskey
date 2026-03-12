ALTER TABLE realm_settings
    DROP COLUMN access_token_lifetime_secs,
    DROP COLUMN refresh_token_lifetime_secs,
    DROP COLUMN id_token_lifetime_secs,
    DROP COLUMN temporary_token_lifetime_secs;

ALTER TABLE clients
    DROP COLUMN access_token_lifetime_secs,
    DROP COLUMN refresh_token_lifetime_secs,
    DROP COLUMN id_token_lifetime_secs,
    DROP COLUMN temporary_token_lifetime_secs;
