CREATE TABLE session_participants (
    session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,
    realm_id UUID NOT NULL,
    client_id TEXT NOT NULL,
    issuer TEXT NOT NULL,
    PRIMARY KEY (session_id, client_id, issuer)
);

INSERT INTO session_participants (session_id, realm_id, client_id, issuer)
SELECT DISTINCT s.id, t.realm_id, t.claims ->> 'azp', t.claims ->> 'iss'
FROM access_tokens t
JOIN user_sessions s ON s.id::text = t.claims ->> 'sid'
WHERE t.claims ->> 'azp' IS NOT NULL AND t.claims ->> 'iss' IS NOT NULL
ON CONFLICT DO NOTHING;

CREATE TABLE session_token_revocations (
    session_id UUID PRIMARY KEY REFERENCES user_sessions(id) ON DELETE CASCADE,
    revoked_before TIMESTAMP NOT NULL
);
