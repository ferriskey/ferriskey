CREATE TABLE realm_resource_owners (
  realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
  uri TEXT NOT NULL,
  client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (realm_id, uri)
);
CREATE INDEX realm_resource_owners_client_id_idx ON realm_resource_owners(client_id);

ALTER TABLE client_scopes
  ADD COLUMN dynamic_registration_allowed BOOLEAN NOT NULL DEFAULT FALSE;
