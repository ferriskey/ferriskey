-- Add up migration script here
CREATE TABLE user_attributes (
    id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    realm_id   UUID        NOT NULL REFERENCES realms (id) ON DELETE CASCADE,
    key        VARCHAR(255) NOT NULL,
    value      TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_user_attributes PRIMARY KEY (id),
    CONSTRAINT unique_user_attribute_key UNIQUE (user_id, key)
);

CREATE INDEX idx_user_attributes_user_id ON user_attributes (user_id);
CREATE INDEX idx_user_attributes_realm_key ON user_attributes (realm_id, key);
