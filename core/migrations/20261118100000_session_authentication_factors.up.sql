ALTER TABLE user_sessions
    ADD COLUMN authentication_factors JSONB NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE auth_sessions
    ADD COLUMN verified_factors JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN acr_values TEXT NULL;

ALTER TABLE realm_settings
    ADD COLUMN acr_loa_map JSONB NOT NULL DEFAULT '{}'::jsonb;
