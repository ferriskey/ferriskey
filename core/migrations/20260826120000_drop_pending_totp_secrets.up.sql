-- The `pending_totp_secrets` table was superseded by `otp_enrollments`
-- (see the OTP self-service consolidation). This migration drops the obsolete
-- table — including on databases that already recorded version
-- 20260817000001, whose original file is preserved byte-for-byte so `sqlx`
-- checksum validation still passes.
DROP TABLE IF EXISTS pending_totp_secrets;
