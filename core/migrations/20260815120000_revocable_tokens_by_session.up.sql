CREATE INDEX idx_access_tokens_claims_sid ON access_tokens ((claims ->> 'sid'));

ALTER TABLE refresh_tokens ADD COLUMN session_id UUID;

CREATE INDEX idx_refresh_tokens_session_id ON refresh_tokens (session_id);
