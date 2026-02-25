-- Add up migration script here
ALTER TABLE client_scopes
ADD COLUMN IF NOT EXISTS "default_scope_type" VARCHAR(255) DEFAULT 'NONE' NOT NULL;

-- remove the column "is_default" if it exists
ALTER TABLE client_scopes
DROP COLUMN IF EXISTS "is_default";


ALTER TABLE client_scope_mappings
ADD COLUMN IF NOT EXISTS "default_scope_type" VARCHAR(255) DEFAULT 'NONE' NOT NULL;

ALTER TABLE client_scope_mappings
DROP COLUMN IF EXISTS "is_default";

ALTER TABLE client_scope_mappings
DROP COLUMN IF EXISTS "is_optional";
