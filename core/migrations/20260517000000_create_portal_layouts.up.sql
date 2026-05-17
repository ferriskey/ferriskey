CREATE TABLE portal_layouts (
  id UUID PRIMARY KEY,
  realm_id UUID NOT NULL,
  name VARCHAR(255) NOT NULL,
  tree JSONB NOT NULL DEFAULT '[]'::jsonb,
  is_default BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_portal_layouts_realm
    FOREIGN KEY (realm_id)
    REFERENCES realms (id)
    ON DELETE CASCADE
);

CREATE INDEX idx_portal_layouts_realm_id
  ON portal_layouts (realm_id);

-- At most one default layout per realm.
CREATE UNIQUE INDEX uq_portal_layouts_realm_default
  ON portal_layouts (realm_id)
  WHERE is_default = TRUE;
