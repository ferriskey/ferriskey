-- Self-service account security (#1479) needs two pieces of short-lived server state
-- that no existing table can hold.
--
-- `account_elevations` is the proof that a signed-in user re-authenticated. It is a
-- window rather than a ticket: reusable until `expires_at`, so one re-authentication
-- covers a two-step flow (start an enrolment, then confirm it) without asking for the
-- password again in the middle. What keeps that safe is the binding: a proof is tied
-- to a user, a realm AND the session that minted it, so a leaked id is useless from
-- anywhere else. `proof` records which credential was presented, because removing a
-- second factor must not be authorised by that same second factor.
CREATE TABLE account_elevations (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  realm_id UUID NOT NULL,
  session_id UUID NOT NULL,
  proof VARCHAR(16) NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_account_elevations_user
    FOREIGN KEY (user_id)
    REFERENCES users (id)
    ON DELETE CASCADE,

  CONSTRAINT fk_account_elevations_realm
    FOREIGN KEY (realm_id)
    REFERENCES realms (id)
    ON DELETE CASCADE
);

-- Lookup is always "this proof, for this caller".
CREATE INDEX idx_account_elevations_user_id ON account_elevations (user_id);

-- Supports purging expired rows without a sequential scan.
CREATE INDEX idx_account_elevations_expires_at ON account_elevations (expires_at);

-- `passkey_registrations` holds the WebAuthn registration challenge between the two
-- halves of a self-service enrolment. The login flow parks its challenge on the auth
-- session (`auth_sessions.webauthn_challenge`), which does not exist here: an account
-- console operates on an established session, not on a login in progress.
--
-- Single-use, unlike an elevation: a challenge answered once must not be answerable
-- twice, and `consumed_at` is what the adapter's compare-and-swap keys on.
CREATE TABLE passkey_registrations (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  state JSONB NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  consumed_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_passkey_registrations_user
    FOREIGN KEY (user_id)
    REFERENCES users (id)
    ON DELETE CASCADE
);

CREATE INDEX idx_passkey_registrations_user_id ON passkey_registrations (user_id);

CREATE INDEX idx_passkey_registrations_expires_at ON passkey_registrations (expires_at);

-- A passkey list that shows no last-used date is not usable: someone with three keys
-- cannot tell which one they lost. `user_label` already exists on the table and is
-- what carries the name.
ALTER TABLE credentials
  ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMP NULL;
