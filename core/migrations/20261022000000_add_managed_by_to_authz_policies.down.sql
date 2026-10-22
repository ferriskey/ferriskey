DROP INDEX IF EXISTS idx_authz_policies_managed_by;

ALTER TABLE authz_policies DROP COLUMN IF EXISTS managed_by;
