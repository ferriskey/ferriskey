-- Add up migration script here
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY,
    jti UUID UNIQUE NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
