ALTER TABLE users
    DROP COLUMN locale;

ALTER TABLE realm_settings
    DROP COLUMN supported_locales,
    DROP COLUMN default_locale;
