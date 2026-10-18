ALTER TABLE users
    ADD COLUMN IF NOT EXISTS federation_provider_id UUID;

ALTER TABLE users
    ADD CONSTRAINT users_federation_provider_id_fkey
    FOREIGN KEY (federation_provider_id) REFERENCES user_federation_providers(id) ON DELETE SET NULL
    NOT VALID;
