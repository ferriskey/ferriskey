-- Add down migration script here


ALTER TABLE client_scope_mappings
ADD COLUMN IF NOT EXISTS "is_default" BOOLEAN DEFAULT FALSE NOT NULL;

ALTER TABLE client_scope_mappings
ADD COLUMN IF NOT EXISTS "is_optional" BOOLEAN DEFAULT FALSE NOT NULL;

UPDATE client_scope_mappings
SET
    is_default = (default_scope_type = 'DEFAULT'),
    is_optional = (default_scope_type = 'OPTIONAL')
WHERE default_scope_type IN ('DEFAULT', 'OPTIONAL');

ALTER TABLE client_scope_mappings
DROP COLUMN IF EXISTS "default_scope_type";


ALTER TABLE client_scopes
ADD COLUMN IF NOT EXISTS "is_default" BOOLEAN DEFAULT FALSE NOT NULL;

UPDATE client_scopes
SET
    is_default = (default_scope_type = 'DEFAULT')
WHERE default_scope_type = 'DEFAULT';

ALTER TABLE client_scopes
DROP COLUMN IF EXISTS "default_scope_type";
