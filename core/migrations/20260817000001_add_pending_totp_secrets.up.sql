-- Back-compat no-op migration.
--
-- Version 20260817000001 previously introduced `pending_totp_secrets`, but the
-- OTP flow has since been consolidated onto the older `otp_enrollments`
-- storage. Keep this migration slot present so databases that already recorded
-- the version can still migrate cleanly and fresh installs preserve the
-- historical sequence without recreating the obsolete table.
SELECT 1;
