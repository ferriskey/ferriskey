use crate::infrastructure::realm::repositories::smtp_config_postgres_repository::SMTP_PASSWORD_PURPOSE;
use crate::infrastructure::repositories::credential_repository::OTP_SECRET_PURPOSE;
use crate::infrastructure::repositories::keystore_repository::PRIVATE_KEY_PURPOSE;

pub(super) const ALIAS: &str = "{a}";
pub(super) const REALM: &str = "{realm}";

#[derive(Debug)]
pub(super) struct TableSpec {
    pub name: &'static str,
    pub scope: &'static str,
    pub conflict: Option<&'static [&'static str]>,
    pub dropped: &'static [&'static str],
    pub tree_parent: Option<&'static str>,
}

impl TableSpec {
    const fn new(name: &'static str, scope: &'static str) -> Self {
        Self {
            name,
            scope,
            conflict: None,
            dropped: &[],
            tree_parent: None,
        }
    }

    const fn matched_on(mut self, conflict: &'static [&'static str]) -> Self {
        self.conflict = Some(conflict);
        self
    }

    const fn without(mut self, dropped: &'static [&'static str]) -> Self {
        self.dropped = dropped;
        self
    }

    const fn parents_first(mut self, parent: &'static str) -> Self {
        self.tree_parent = Some(parent);
        self
    }

    pub fn scope_on(&self, alias: &str, realm: &str) -> String {
        self.scope.replace(ALIAS, alias).replace(REALM, realm)
    }
}

const IN_REALM: &str = "{a}.realm_id = {realm}";

pub(super) const TABLES: &[TableSpec] = &[
    TableSpec::new("realms", "{a}.id = {realm}"),
    TableSpec::new("jwt_keys", IN_REALM),
    TableSpec::new("jwt_keys_superseded", IN_REALM),
    TableSpec::new("email_templates", IN_REALM),
    TableSpec::new("portal_layouts", IN_REALM),
    TableSpec::new(
        "portal_themes",
        "{a}.realm_id = {realm} AND ({a}.layout_id IS NULL OR {a}.layout_id IN (SELECT id FROM portal_layouts WHERE realm_id = {realm}))",
    ),
    TableSpec::new(
        "realm_settings",
        "{a}.realm_id = {realm} \
         AND ({a}.reset_password_template_id IS NULL OR {a}.reset_password_template_id IN (SELECT id FROM email_templates WHERE realm_id = {realm})) \
         AND ({a}.magic_link_template_id IS NULL OR {a}.magic_link_template_id IN (SELECT id FROM email_templates WHERE realm_id = {realm})) \
         AND ({a}.email_verification_template_id IS NULL OR {a}.email_verification_template_id IN (SELECT id FROM email_templates WHERE realm_id = {realm})) \
         AND ({a}.portal_theme_id IS NULL OR {a}.portal_theme_id IN (SELECT id FROM portal_themes WHERE realm_id = {realm}))",
    ),
    TableSpec::new("password_policy", IN_REALM),
    TableSpec::new("smtp_configs", IN_REALM),
    TableSpec::new("user_federation_providers", IN_REALM),
    TableSpec::new("identity_providers", IN_REALM),
    TableSpec::new("clients", IN_REALM),
    TableSpec::new(
        "redirect_uris",
        "{a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "post_logout_redirect_uris",
        "{a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "client_web_origins",
        "{a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "client_saml_configs",
        "{a}.realm_id = {realm} AND {a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "client_saml_attribute_mappers",
        "{a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "roles",
        "{a}.realm_id = {realm} AND ({a}.client_id IS NULL OR {a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm}))",
    ),
    TableSpec::new("client_scopes", IN_REALM),
    TableSpec::new(
        "client_scope_attributes",
        "{a}.client_scope_id IN (SELECT id FROM client_scopes WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "client_scope_protocol_mappers",
        "{a}.client_scope_id IN (SELECT id FROM client_scopes WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "client_scope_mappings",
        "{a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm}) \
         AND {a}.client_scope_id IN (SELECT id FROM client_scopes WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "token_exchange_policies",
        "{a}.realm_id = {realm} AND {a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "realm_resource_owners",
        "{a}.realm_id = {realm} AND {a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "users",
        "{a}.realm_id = {realm} \
         AND ({a}.client_id IS NULL OR {a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})) \
         AND ({a}.federation_provider_id IS NULL OR {a}.federation_provider_id IN (SELECT id FROM user_federation_providers WHERE realm_id = {realm}))",
    ),
    TableSpec::new(
        "credentials",
        "{a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "user_required_actions",
        "{a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "user_attributes",
        "{a}.realm_id = {realm} AND {a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "user_role",
        "{a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm}) \
         AND {a}.role_id IN (SELECT id FROM roles WHERE realm_id = {realm})",
    )
    .matched_on(&["user_id", "role_id"])
    .without(&["id"]),
    TableSpec::new(
        "user_federation_mappings",
        "{a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm}) \
         AND {a}.provider_id IN (SELECT id FROM user_federation_providers WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "identity_provider_links",
        "{a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm}) \
         AND {a}.identity_provider_id IN (SELECT id FROM identity_providers WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "user_token_revocations",
        "{a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "realm_maintenance_whitelist",
        "{a}.realm_id = {realm} \
         AND ({a}.user_id IS NULL OR {a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})) \
         AND ({a}.role_id IS NULL OR {a}.role_id IN (SELECT id FROM roles WHERE realm_id = {realm}))",
    ),
    TableSpec::new(
        "client_maintenance_whitelist",
        "{a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm}) \
         AND ({a}.user_id IS NULL OR {a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})) \
         AND ({a}.role_id IS NULL OR {a}.role_id IN (SELECT id FROM roles WHERE realm_id = {realm}))",
    ),
    TableSpec::new("organizations", IN_REALM),
    TableSpec::new(
        "organization_attributes",
        "{a}.organization_id IN (SELECT id FROM organizations WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "organization_groups",
        "{a}.organization_id IN (SELECT id FROM organizations WHERE realm_id = {realm})",
    )
    .parents_first("parent_group_id"),
    TableSpec::new(
        "organization_group_attributes",
        "{a}.group_id IN (SELECT g.id FROM organization_groups g JOIN organizations o ON o.id = g.organization_id WHERE o.realm_id = {realm})",
    ),
    TableSpec::new(
        "organization_group_members",
        "{a}.group_id IN (SELECT g.id FROM organization_groups g JOIN organizations o ON o.id = g.organization_id WHERE o.realm_id = {realm}) \
         AND {a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "organization_group_roles",
        "{a}.group_id IN (SELECT g.id FROM organization_groups g JOIN organizations o ON o.id = g.organization_id WHERE o.realm_id = {realm}) \
         AND {a}.role_id IN (SELECT id FROM roles WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "organization_members",
        "{a}.organization_id IN (SELECT id FROM organizations WHERE realm_id = {realm}) \
         AND {a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "organization_member_roles",
        "{a}.organization_member_id IN (SELECT m.id FROM organization_members m JOIN organizations o ON o.id = m.organization_id WHERE o.realm_id = {realm}) \
         AND {a}.role_id IN (SELECT id FROM roles WHERE realm_id = {realm})",
    ),
    TableSpec::new("webhooks", IN_REALM),
    TableSpec::new(
        "webhook_subscribers",
        "{a}.webhook_id IN (SELECT id FROM webhooks WHERE realm_id = {realm})",
    ),
    TableSpec::new("authz_realm_states", IN_REALM),
    TableSpec::new("authz_schemas", IN_REALM),
    TableSpec::new("authz_policy_templates", IN_REALM),
    TableSpec::new("authz_policies", IN_REALM),
    TableSpec::new("authz_relation_tuples", IN_REALM),
    TableSpec::new(
        "consent_sessions",
        "{a}.realm_id = {realm} \
         AND {a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm}) \
         AND {a}.client_id IN (SELECT id FROM clients WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "consent_session_scopes",
        "{a}.consent_session_id IN (SELECT id FROM consent_sessions WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "user_sessions",
        "{a}.realm_id = {realm} AND {a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "session_participants",
        "{a}.realm_id = {realm} AND {a}.session_id IN (SELECT id FROM user_sessions WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "session_token_revocations",
        "{a}.session_id IN (SELECT id FROM user_sessions WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "refresh_tokens",
        "{a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new(
        "access_tokens",
        "{a}.realm_id = {realm} AND {a}.user_id IN (SELECT id FROM users WHERE realm_id = {realm})",
    ),
    TableSpec::new("security_events", IN_REALM),
    TableSpec::new("security_event_chain_heads", IN_REALM),
];

#[cfg(test)]
pub(super) const NOT_EXPORTED: &[&str] = &[
    "_sqlx_migrations",
    "data_migrations",
    "auth_sessions",
    "broker_auth_sessions",
    "device_auth_sessions",
    "magic_links",
    "password_reset_tokens",
    "email_verification_tokens",
    "login_action_tokens",
    "account_elevations",
    "otp_enrollments",
    "passkey_registrations",
    "webhook_deliveries",
    "compass_flows",
    "compass_flow_steps",
];

#[derive(Debug)]
pub(super) struct SealedColumn {
    pub table: &'static str,
    pub column: &'static str,
    pub purpose: &'static str,
    pub only_when: Option<(&'static str, &'static str)>,
}

pub(super) const SEALED_COLUMNS: &[SealedColumn] = &[
    SealedColumn {
        table: "jwt_keys",
        column: "private_key",
        purpose: PRIVATE_KEY_PURPOSE,
        only_when: None,
    },
    SealedColumn {
        table: "credentials",
        column: "secret_data",
        purpose: OTP_SECRET_PURPOSE,
        only_when: Some(("credential_type", "otp")),
    },
    SealedColumn {
        table: "smtp_configs",
        column: "password",
        purpose: SMTP_PASSWORD_PURPOSE,
        only_when: None,
    },
];

pub(super) fn spec_of(table: &str) -> Option<&'static TableSpec> {
    TABLES.iter().find(|spec| spec.name == table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realm_transfer::bundle::BUNDLE_TABLES;
    use crate::infrastructure::secrets_backfill::sealed_columns;

    #[test]
    fn the_registry_follows_the_bundle_order() {
        let names: Vec<&str> = TABLES.iter().map(|spec| spec.name).collect();

        assert_eq!(names, BUNDLE_TABLES);
    }

    #[test]
    fn no_table_is_both_exported_and_left_out() {
        for table in NOT_EXPORTED {
            assert!(spec_of(table).is_none(), "{table}");
        }
    }

    #[test]
    fn every_scope_is_bound_to_the_realm() {
        for spec in TABLES {
            assert!(spec.scope.contains(ALIAS), "{}", spec.name);
            assert!(spec.scope.contains(REALM), "{}", spec.name);
        }
    }

    #[test]
    fn every_secret_the_backfill_seals_in_an_exported_table_is_opened_at_export() {
        for (table, column) in sealed_columns() {
            if spec_of(table).is_none() {
                assert!(NOT_EXPORTED.contains(&table), "{table}");
                continue;
            }
            assert!(
                SEALED_COLUMNS
                    .iter()
                    .any(|sealed| sealed.table == table && sealed.column == column),
                "{table}.{column}"
            );
        }
    }
}
