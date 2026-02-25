-- Add down migration script here


ALTER TABLE client_scope_mappings
ADD COLUMN IF NOT EXISTS "is_default" BOOLEAN DEFAULT FALSE NOT NULL;

ALTER TABLE client_scope_mappings
ADD COLUMN IF NOT EXISTS "is_optional" BOOLEAN DEFAULT FALSE NOT NULL;

ALTER TABLE client_scope_mappings
DROP COLUMN IF EXISTS "default_scope_type";


ALTER TABLE client_scopes
ADD COLUMN IF NOT EXISTS "is_default" BOOLEAN DEFAULT FALSE NOT NULL;

ALTER TABLE client_scopes
DROP COLUMN IF EXISTS "default_scope_type";
