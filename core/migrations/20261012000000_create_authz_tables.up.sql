-- Tenant authorization: the rules a realm writes for its own applications.

-- Every write to a realm's schema, policies or templates bumps `version` in the
-- same transaction, and only if it still holds the version the writer read.
CREATE TABLE authz_realm_states (
    realm_id   UUID PRIMARY KEY REFERENCES realms(id) ON DELETE CASCADE,
    enabled    BOOLEAN NOT NULL DEFAULT FALSE,
    version    BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE authz_schemas (
    id         UUID PRIMARY KEY,
    realm_id   UUID NOT NULL UNIQUE REFERENCES realms(id) ON DELETE CASCADE,
    namespace  VARCHAR(64) NOT NULL,
    source     TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE authz_policy_templates (
    id          UUID PRIMARY KEY,
    realm_id    UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    name        VARCHAR(128) NOT NULL,
    description TEXT,
    source      TEXT NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT now(),
    updated_at  TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE (realm_id, name),
    UNIQUE (realm_id, id)
);

-- A policy is static Cedar or a link to a template of the same realm, never
-- both. The composite key keeps a policy from linking another realm's template.
CREATE TABLE authz_policies (
    id             UUID PRIMARY KEY,
    realm_id       UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    name           VARCHAR(128) NOT NULL,
    description    TEXT,
    origin         VARCHAR(16) NOT NULL CHECK (origin IN ('builtin', 'generated', 'custom')),
    enabled        BOOLEAN NOT NULL,
    source         TEXT,
    template_id    UUID,
    template_links JSONB,
    created_at     TIMESTAMP NOT NULL DEFAULT now(),
    updated_at     TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE (realm_id, name),
    CHECK ((source IS NULL) <> (template_id IS NULL)),
    CHECK ((template_id IS NULL) = (template_links IS NULL)),
    FOREIGN KEY (realm_id, template_id)
        REFERENCES authz_policy_templates (realm_id, id) ON DELETE NO ACTION
);

CREATE INDEX idx_authz_policies_template ON authz_policies (realm_id, template_id);

-- `subject_relation` is '' rather than NULL so it can sit in the primary key.
CREATE TABLE authz_relation_tuples (
    realm_id         UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    object_type      VARCHAR(128) NOT NULL,
    object_id        VARCHAR(255) NOT NULL,
    relation         VARCHAR(64) NOT NULL,
    subject_type     VARCHAR(128) NOT NULL,
    subject_id       VARCHAR(255) NOT NULL,
    subject_relation VARCHAR(64) NOT NULL DEFAULT '',
    created_at       TIMESTAMP NOT NULL DEFAULT now(),
    PRIMARY KEY (realm_id, object_type, object_id, relation, subject_type, subject_id, subject_relation)
);

CREATE INDEX idx_authz_relation_tuples_subject
    ON authz_relation_tuples (subject_type, subject_id);
CREATE INDEX idx_authz_relation_tuples_object
    ON authz_relation_tuples (object_type, object_id);

-- A deleted FerrisKey entity must not leave tuples behind: a later entity with
-- the same id would inherit them. A trigger rather than application code, so
-- rows removed by an ON DELETE CASCADE (an organization's groups, a group's
-- subgroups) are covered too.
CREATE FUNCTION authz_forget_entity() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM authz_relation_tuples
     WHERE (subject_type = TG_ARGV[0] AND subject_id = OLD.id::text)
        OR (object_type = TG_ARGV[0] AND object_id = OLD.id::text);
    RETURN OLD;
END;
$$;

CREATE TRIGGER authz_forget_user AFTER DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION authz_forget_entity('FerrisKey::User');
CREATE TRIGGER authz_forget_client AFTER DELETE ON clients
    FOR EACH ROW EXECUTE FUNCTION authz_forget_entity('FerrisKey::Client');
CREATE TRIGGER authz_forget_role AFTER DELETE ON roles
    FOR EACH ROW EXECUTE FUNCTION authz_forget_entity('FerrisKey::Role');
CREATE TRIGGER authz_forget_organization AFTER DELETE ON organizations
    FOR EACH ROW EXECUTE FUNCTION authz_forget_entity('FerrisKey::Organization');
CREATE TRIGGER authz_forget_group AFTER DELETE ON organization_groups
    FOR EACH ROW EXECUTE FUNCTION authz_forget_entity('FerrisKey::Group');
