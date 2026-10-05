DROP TRIGGER IF EXISTS authz_forget_group ON organization_groups;
DROP TRIGGER IF EXISTS authz_forget_organization ON organizations;
DROP TRIGGER IF EXISTS authz_forget_role ON roles;
DROP TRIGGER IF EXISTS authz_forget_client ON clients;
DROP TRIGGER IF EXISTS authz_forget_user ON users;
DROP FUNCTION IF EXISTS authz_forget_entity();

DROP TABLE IF EXISTS authz_relation_tuples;
DROP TABLE IF EXISTS authz_policies;
DROP TABLE IF EXISTS authz_policy_templates;
DROP TABLE IF EXISTS authz_schemas;
DROP TABLE IF EXISTS authz_realm_states;
