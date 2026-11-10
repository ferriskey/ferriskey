ALTER TABLE realm_settings DROP COLUMN acr_loa_map;

ALTER TABLE auth_sessions
    DROP COLUMN acr_values,
    DROP COLUMN verified_factors;

ALTER TABLE user_sessions DROP COLUMN authentication_factors;
