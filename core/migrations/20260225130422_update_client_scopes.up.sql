-- Add up migration script here
ALTER TABLE client_scopes
ADD COLUMN IF NOT EXISTS "default_scope_type" VARCHAR(255) DEFAULT 'NONE' NOT NULL;

-- Migrate existing data before dropping
UPDATE client_scopes
SET default_scope_type = CASE
    WHEN COALESCE(is_default, FALSE) = TRUE THEN 'DEFAULT'
    ELSE 'OPTIONAL'
END;

-- remove the column "is_default" if it exists
ALTER TABLE client_scopes
DROP COLUMN IF EXISTS "is_default";


ALTER TABLE client_scope_mappings
ADD COLUMN IF NOT EXISTS "default_scope_type" VARCHAR(255) DEFAULT 'NONE' NOT NULL;

-- Migrate existing data before dropping
UPDATE client_scope_mappings SET default_scope_type = 'DEFAULT' WHERE is_default = TRUE;
UPDATE client_scope_mappings SET default_scope_type = 'OPTIONAL' WHERE is_optional = TRUE;

ALTER TABLE client_scope_mappings
DROP COLUMN IF EXISTS "is_default";

ALTER TABLE client_scope_mappings
DROP COLUMN IF EXISTS "is_optional";
