ALTER TABLE password_policy
    DROP COLUMN IF EXISTS min_entropy_bits,
    DROP COLUMN IF EXISTS forbid_common,
    DROP COLUMN IF EXISTS check_breached;
