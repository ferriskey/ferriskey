-- The "remember me" choice made on the login page, carried by the
-- authorization request until the login completes (possibly several steps
-- later, after an MFA challenge).
ALTER TABLE auth_sessions
    ADD COLUMN IF NOT EXISTS remember_me BOOLEAN NOT NULL DEFAULT FALSE;

-- Whether the SSO cookie outlives the browser. Existing sessions default to a
-- browser-session cookie the next time it is re-issued.
ALTER TABLE user_sessions
    ADD COLUMN IF NOT EXISTS persistent BOOLEAN NOT NULL DEFAULT FALSE;

-- When the user last authenticated interactively in this session, which
-- `max_age` is measured against. Until now that was always `created_at`.
ALTER TABLE user_sessions
    ADD COLUMN IF NOT EXISTS authenticated_at TIMESTAMPTZ;
UPDATE user_sessions SET authenticated_at = created_at WHERE authenticated_at IS NULL;
ALTER TABLE user_sessions
    ALTER COLUMN authenticated_at SET NOT NULL,
    ALTER COLUMN authenticated_at SET DEFAULT NOW();
