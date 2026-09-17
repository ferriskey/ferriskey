ALTER TABLE realm_settings
    ADD COLUMN default_locale    TEXT   NOT NULL DEFAULT 'en',
    ADD COLUMN supported_locales TEXT[] NOT NULL DEFAULT '{en}';

ALTER TABLE users
    ADD COLUMN locale TEXT NULL;
