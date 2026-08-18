-- FK-011: the partial-authentication step token was a bearer credential with no
-- server-side state: never persisted, so `revoke_token` no-opped for it and
-- `verify_token` skipped its revocation check. It stayed replayable until `exp`,
-- even after the user had finished the step it was minted for.
--
-- This table is that missing state. One row per issued step token, keyed by the
-- token's own `jti`, consumed by compare-and-swap the moment the step it
-- authorises actually completes.
--
-- Consumption is deliberately NOT on first use: `setup-otp` (GET) and
-- `verify-otp` (POST) legitimately share one token, so consuming on first touch
-- would make OTP enrolment impossible. It is also not on every POST: a mistyped
-- OTP code would then cost the user a full re-login.
CREATE TABLE login_action_tokens (
  jti UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  realm_id UUID NOT NULL,
  auth_session_id UUID NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  consumed_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_login_action_token_user
    FOREIGN KEY (user_id)
    REFERENCES users (id)
    ON DELETE CASCADE
);

-- Supports purging expired rows without a sequential scan.
CREATE INDEX idx_login_action_tokens_expires_at ON login_action_tokens (expires_at);
