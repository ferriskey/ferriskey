-- Add up migration script here
CREATE TABLE realm_settings
(
    id                        UUID PRIMARY KEY,
    realm_id                  UUID        NOT NULL,
    default_signing_algorithm VARCHAR(255),
    updated_at                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE realm_settings
    ADD CONSTRAINT fk_realm_settings_realm_id
        FOREIGN KEY (realm_id)
            REFERENCES realms (id)
            ON DELETE CASCADE;
