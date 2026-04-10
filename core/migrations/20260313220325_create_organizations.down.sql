-- Add down migration script here
DROP INDEX idx_organization_members_user_id;
DROP INDEX idx_organization_members_org_id;
DROP TABLE organization_members;

DROP INDEX idx_organization_attributes_org_id;
DROP TABLE organization_attributes;

DROP INDEX idx_organizations_domain;
DROP INDEX idx_organizations_realm_id;
DROP TABLE organizations;
