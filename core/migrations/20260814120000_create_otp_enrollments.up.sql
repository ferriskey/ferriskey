-- FK-003: `setup_otp` generated a TOTP secret, returned it to the client and forgot
-- it. `verify_otp` then took the secret back *from the request body* and verified
-- the submitted code against that same secret — a tautology. There was no server
-- state to compare against, so the defect was structural, not incidental.
--
-- This table is that missing state: the candidate secret lives here between
-- `setup_otp` and `verify_otp`, with a short TTL and single-use semantics, and is
-- deleted once promoted to a real `credentials` row.
--
-- `secret` is stored as plaintext base32, deliberately: the *active* OTP secret in
-- `credentials.secret_data` is already plaintext, so encrypting only the transient
-- copy would buy nothing while the durable one is readable. Encrypting both is
-- worth doing and is tracked separately.
CREATE TABLE otp_enrollments (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  secret TEXT NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  consumed_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_user
    FOREIGN KEY (user_id)
    REFERENCES users (id)
    ON DELETE CASCADE
);

-- Lookup is always "the live enrolment for this user".
CREATE INDEX idx_otp_enrollments_user_id ON otp_enrollments (user_id);

-- Supports purging expired rows without a sequential scan.
CREATE INDEX idx_otp_enrollments_expires_at ON otp_enrollments (expires_at);
