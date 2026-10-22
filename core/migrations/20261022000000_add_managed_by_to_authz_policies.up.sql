ALTER TABLE authz_policies ADD COLUMN managed_by VARCHAR(64);

CREATE INDEX idx_authz_policies_managed_by ON authz_policies (realm_id, managed_by);
