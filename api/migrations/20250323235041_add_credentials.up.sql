-- Add up migration script here
-- Add up migration script here
CREATE TABLE credentials (
    id UUID PRIMARY KEY,
    salt VARCHAR(255),
    credential_type VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    user_label VARCHAR(255),
    secret_data TEXT NOT NULL,
    credential_data TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
