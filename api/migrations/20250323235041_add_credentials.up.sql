-- Add up migration script here
-- Add up migration script here
CREATE TABLE credentials (
    id UUID PRIMARY KEY,
    salt VARCHAR(255),
    credential_type VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    user_label VARCHAR(255),
    secret_data TEXT NOT NULL,
    credential_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);


-- Unique constraint for user_id and credential_type
ALTER TABLE credentials ADD CONSTRAINT unique_credential_type_per_user_id UNIQUE (user_id, credential_type);