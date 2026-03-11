-- Create password_policy table
CREATE TABLE password_policy (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id    UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    min_length          INTEGER NOT NULL DEFAULT 8,
    require_uppercase   BOOLEAN NOT NULL DEFAULT false,
    require_lowercase   BOOLEAN NOT NULL DEFAULT false,
    require_number      BOOLEAN NOT NULL DEFAULT false,
    require_special     BOOLEAN NOT NULL DEFAULT false,
    max_age_days        INTEGER,          -- NULL means no expiry
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(realm_id)    -- one policy per realm
);

-- Insert default policy row for any existing realms
INSERT INTO password_policy (realm_id)
SELECT id FROM realms
ON CONFLICT (realm_id) DO NOTHING;
