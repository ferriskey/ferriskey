ALTER TABLE client_scopes DROP COLUMN dynamic_registration_allowed;

DROP INDEX realm_resource_owners_client_id_idx;
DROP TABLE realm_resource_owners;
