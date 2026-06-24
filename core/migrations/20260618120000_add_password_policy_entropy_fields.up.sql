ALTER TABLE password_policy
    ADD COLUMN IF NOT EXISTS min_entropy_bits INTEGER NOT NULL DEFAULT 80,
    ADD COLUMN IF NOT EXISTS forbid_common    BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS check_breached   BOOLEAN NOT NULL DEFAULT false;

-- Bring existing rows' min_length and require_* in line with CNIL 2022-100 defaults
-- (only updates rows that still carry the original permissive defaults)
UPDATE password_policy
SET
    min_length        = 12,
    require_uppercase = true,
    require_lowercase = true,
    require_number    = true,
    require_special   = true
WHERE
    min_length        = 8
    AND require_uppercase = false
    AND require_lowercase = false
    AND require_number    = false
    AND require_special   = false;
