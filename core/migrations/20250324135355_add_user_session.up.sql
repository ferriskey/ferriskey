-- Add up migration script here
CREATE TABLE user_sessions ( 
    id UUID PRIMARY KEY, 
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE, 
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    user_agent VARCHAR(255),
    ip_address VARCHAR(255), 
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), 
    expires_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_realm_id ON user_sessions(realm_id);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
