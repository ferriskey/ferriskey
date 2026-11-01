use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;

pub const BUNDLE_FORMAT: &str = "ferriskey.realm-bundle";
pub const BUNDLE_VERSION: BundleVersion = BundleVersion { major: 1, minor: 0 };
pub const BUNDLE_MEDIA_TYPE: &str = "application/vnd.ferriskey.realm-bundle";
pub const BUNDLE_EXTENSION: &str = "fkrb";

pub const BUNDLE_TABLES: &[&str] = &[
    "realms",
    "jwt_keys",
    "jwt_keys_superseded",
    "email_templates",
    "portal_layouts",
    "portal_themes",
    "realm_settings",
    "password_policy",
    "smtp_configs",
    "user_federation_providers",
    "identity_providers",
    "clients",
    "redirect_uris",
    "post_logout_redirect_uris",
    "client_web_origins",
    "client_saml_configs",
    "client_saml_attribute_mappers",
    "roles",
    "client_scopes",
    "client_scope_attributes",
    "client_scope_protocol_mappers",
    "client_scope_mappings",
    "token_exchange_policies",
    "realm_resource_owners",
    "users",
    "credentials",
    "user_required_actions",
    "user_attributes",
    "user_role",
    "user_federation_mappings",
    "identity_provider_links",
    "user_token_revocations",
    "realm_maintenance_whitelist",
    "client_maintenance_whitelist",
    "organizations",
    "organization_attributes",
    "organization_groups",
    "organization_group_attributes",
    "organization_group_members",
    "organization_group_roles",
    "organization_members",
    "organization_member_roles",
    "webhooks",
    "webhook_subscribers",
    "authz_realm_states",
    "authz_schemas",
    "authz_policy_templates",
    "authz_policies",
    "authz_relation_tuples",
    "consent_sessions",
    "consent_session_scopes",
    "user_sessions",
    "session_participants",
    "session_token_revocations",
    "refresh_tokens",
    "access_tokens",
    "security_events",
    "security_event_chain_heads",
];

const REALM_TABLE: &str = "realms";
const SIGNING_KEY_TABLE: &str = "jwt_keys";
const LAST_KEY_TABLE: &str = "jwt_keys_superseded";
const PASSPHRASE_LENGTH: std::ops::RangeInclusive<usize> = 16..=1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleRealm {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleHeader {
    pub format: String,
    pub version: BundleVersion,
    pub realm: BundleRealm,
    pub schema: i64,
    pub exported_at: DateTime<Utc>,
}

impl BundleHeader {
    pub fn new(realm: BundleRealm, schema: i64, exported_at: DateTime<Utc>) -> Self {
        Self {
            format: BUNDLE_FORMAT.to_string(),
            version: BUNDLE_VERSION,
            realm,
            schema,
            exported_at,
        }
    }

    pub fn check(&self, target_schema: i64) -> Result<(), CoreError> {
        if self.format != BUNDLE_FORMAT {
            return Err(invalid(format!("unknown format {:?}", self.format)));
        }
        if self.version.major != BUNDLE_VERSION.major {
            return Err(invalid(format!(
                "version {}.{} is not readable by this instance, which reads {}.x",
                self.version.major, self.version.minor, BUNDLE_VERSION.major
            )));
        }
        if self.schema > target_schema {
            return Err(invalid(format!(
                "the bundle comes from schema {}, newer than this instance ({target_schema}): upgrade it first",
                self.schema
            )));
        }
        if self.realm.name == "master" {
            return Err(CoreError::MasterRealmNotTransferable);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleRow {
    pub table: String,
    pub row: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleTrailer {
    pub rows: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BundleLine {
    Header(BundleHeader),
    Row(BundleRow),
    Trailer(BundleTrailer),
}

#[derive(Clone)]
pub struct BundlePassphrase(String);

impl BundlePassphrase {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for BundlePassphrase {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let printable = value.bytes().all(|byte| (0x20..=0x7e).contains(&byte));
        if printable && PASSPHRASE_LENGTH.contains(&value.len()) {
            Ok(Self(value))
        } else {
            Err(CoreError::InvalidBundlePassphrase)
        }
    }
}

impl fmt::Debug for BundlePassphrase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BundlePassphrase(..)")
    }
}

pub fn table_position(table: &str) -> Option<usize> {
    BUNDLE_TABLES.iter().position(|known| *known == table)
}

fn invalid(message: String) -> CoreError {
    CoreError::InvalidRealmBundle(message)
}

#[derive(Debug)]
pub struct BundleCheck {
    realm: BundleRealm,
    position: Option<usize>,
    counts: BTreeMap<&'static str, u64>,
    active_key: bool,
}

impl BundleCheck {
    pub fn new(header: &BundleHeader) -> Self {
        Self {
            realm: header.realm.clone(),
            position: None,
            counts: BTreeMap::new(),
            active_key: false,
        }
    }

    pub fn counts(&self) -> &BTreeMap<&'static str, u64> {
        &self.counts
    }

    pub fn row(&mut self, row: &BundleRow) -> Result<(), CoreError> {
        let position = table_position(&row.table)
            .ok_or_else(|| invalid(format!("table {:?} is not part of a bundle", row.table)))?;
        let table = BUNDLE_TABLES[position];

        match self.position {
            None if table != REALM_TABLE => {
                return Err(invalid("the first row must be the realm".to_string()));
            }
            Some(current) if position < current => {
                return Err(invalid(format!("table {table} comes out of order")));
            }
            _ => {}
        }
        if position > key_section_end() {
            self.require_keys()?;
        }
        self.position = Some(position);

        if table == REALM_TABLE {
            self.check_realm_row(row)?;
        }
        if let Some(realm_id) = row.row.get("realm_id")
            && realm_id.as_str() != Some(self.realm.id.to_string().as_str())
        {
            return Err(invalid(format!(
                "a row of {table} belongs to another realm"
            )));
        }
        if table == SIGNING_KEY_TABLE && is_active_key(&row.row) {
            self.active_key = true;
        }

        *self.counts.entry(table).or_default() += 1;
        Ok(())
    }

    pub fn trailer(&self, trailer: &BundleTrailer) -> Result<(), CoreError> {
        self.require_keys()?;

        let declared = trailer.rows.iter().filter(|(_, count)| **count > 0);
        let read = self.counts.iter().map(|(table, count)| (*table, *count));
        if !declared
            .map(|(table, count)| (table.as_str(), *count))
            .eq(read)
        {
            return Err(invalid(
                "the trailer does not match the rows read".to_string(),
            ));
        }
        Ok(())
    }

    fn require_keys(&self) -> Result<(), CoreError> {
        if self.active_key {
            Ok(())
        } else {
            Err(CoreError::RealmBundleWithoutKeys)
        }
    }

    fn check_realm_row(&self, row: &BundleRow) -> Result<(), CoreError> {
        if self.counts.contains_key(REALM_TABLE) {
            return Err(invalid("the bundle holds two realms".to_string()));
        }
        let id = row.row.get("id").and_then(Value::as_str);
        let name = row.row.get("name").and_then(Value::as_str);
        if id != Some(self.realm.id.to_string().as_str()) || name != Some(self.realm.name.as_str())
        {
            return Err(invalid(
                "the realm row does not match the header".to_string(),
            ));
        }
        Ok(())
    }
}

fn key_section_end() -> usize {
    table_position(LAST_KEY_TABLE).unwrap_or_default()
}

fn is_active_key(row: &Map<String, Value>) -> bool {
    ["rotated_at", "revoked_at"]
        .iter()
        .all(|column| row.get(*column).is_none_or(Value::is_null))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const REALM_ID: &str = "0190c2a4-0000-7000-8000-000000000001";

    fn header() -> BundleHeader {
        BundleHeader::new(
            BundleRealm {
                id: REALM_ID.parse().unwrap(),
                name: "acme".to_string(),
            },
            20261031181100,
            Utc::now(),
        )
    }

    fn row(table: &str, value: Value) -> BundleRow {
        BundleRow {
            table: table.to_string(),
            row: value.as_object().cloned().unwrap(),
        }
    }

    fn realm_row() -> BundleRow {
        row("realms", json!({ "id": REALM_ID, "name": "acme" }))
    }

    fn key_row(rotated_at: Value) -> BundleRow {
        row(
            "jwt_keys",
            json!({ "id": "k", "realm_id": REALM_ID, "rotated_at": rotated_at, "revoked_at": null }),
        )
    }

    fn user_row() -> BundleRow {
        row("users", json!({ "id": "u", "realm_id": REALM_ID }))
    }

    fn checked(rows: &[BundleRow]) -> Result<BundleCheck, CoreError> {
        let mut check = BundleCheck::new(&header());
        for row in rows {
            check.row(row)?;
        }
        Ok(check)
    }

    #[test]
    fn a_line_reads_back_as_written() {
        for line in [
            BundleLine::Header(header()),
            BundleLine::Row(realm_row()),
            BundleLine::Trailer(BundleTrailer {
                rows: BTreeMap::from([("realms".to_string(), 1)]),
            }),
        ] {
            let text = serde_json::to_string(&line).unwrap();
            assert_eq!(serde_json::from_str::<BundleLine>(&text).unwrap(), line);
        }
    }

    #[test]
    fn a_row_line_names_its_kind_and_table() {
        let text = serde_json::to_string(&BundleLine::Row(realm_row())).unwrap();

        assert!(text.starts_with(r#"{"kind":"row","table":"realms","row":"#));
    }

    #[test]
    fn a_complete_bundle_passes() {
        let check = checked(&[realm_row(), key_row(Value::Null), user_row(), user_row()]).unwrap();

        check
            .trailer(&BundleTrailer {
                rows: BTreeMap::from([
                    ("realms".to_string(), 1),
                    ("jwt_keys".to_string(), 1),
                    ("users".to_string(), 2),
                    ("clients".to_string(), 0),
                ]),
            })
            .unwrap();
    }

    #[test]
    fn a_bundle_without_an_active_key_is_refused_at_the_first_row_past_the_keys() {
        assert!(matches!(
            checked(&[realm_row(), user_row()]),
            Err(CoreError::RealmBundleWithoutKeys)
        ));
        assert!(matches!(
            checked(&[
                realm_row(),
                key_row(json!("2026-10-01T00:00:00")),
                user_row()
            ]),
            Err(CoreError::RealmBundleWithoutKeys)
        ));
    }

    #[test]
    fn a_bundle_of_a_realm_alone_is_refused_at_its_trailer() {
        let check = checked(&[realm_row()]).unwrap();

        assert!(matches!(
            check.trailer(&BundleTrailer::default()),
            Err(CoreError::RealmBundleWithoutKeys)
        ));
    }

    #[test]
    fn a_table_out_of_order_is_refused() {
        assert!(
            checked(&[
                realm_row(),
                key_row(Value::Null),
                user_row(),
                row("clients", json!({}))
            ])
            .is_err()
        );
    }

    #[test]
    fn the_realm_comes_first_and_once() {
        assert!(checked(&[key_row(Value::Null)]).is_err());
        assert!(checked(&[realm_row(), realm_row()]).is_err());
    }

    #[test]
    fn the_realm_row_must_match_the_header() {
        assert!(checked(&[row("realms", json!({ "id": REALM_ID, "name": "other" }))]).is_err());
    }

    #[test]
    fn a_row_of_another_realm_is_refused() {
        let foreign = row(
            "users",
            json!({ "id": "u", "realm_id": "0190c2a4-0000-7000-8000-000000000002" }),
        );

        assert!(checked(&[realm_row(), key_row(Value::Null), foreign]).is_err());
    }

    #[test]
    fn a_table_outside_the_bundle_is_refused() {
        assert!(
            checked(&[
                realm_row(),
                key_row(Value::Null),
                row("auth_sessions", json!({}))
            ])
            .is_err()
        );
    }

    #[test]
    fn a_trailer_that_disagrees_with_the_rows_is_refused() {
        let check = checked(&[realm_row(), key_row(Value::Null)]).unwrap();

        assert!(
            check
                .trailer(&BundleTrailer {
                    rows: BTreeMap::from([("realms".to_string(), 1)]),
                })
                .is_err()
        );
        assert!(
            check
                .trailer(&BundleTrailer {
                    rows: BTreeMap::from([
                        ("realms".to_string(), 1),
                        ("jwt_keys".to_string(), 1),
                        ("users".to_string(), 3),
                    ]),
                })
                .is_err()
        );
    }

    #[test]
    fn a_header_is_checked_against_the_target() {
        assert!(header().check(20261031181100).is_ok());
        assert!(header().check(20270101000000).is_ok());
        assert!(header().check(20260101000000).is_err());

        let mut other_major = header();
        other_major.version.major = 2;
        assert!(other_major.check(20261031181100).is_err());

        let mut newer_minor = header();
        newer_minor.version.minor = 7;
        assert!(newer_minor.check(20261031181100).is_ok());

        let mut master = header();
        master.realm.name = "master".to_string();
        assert!(matches!(
            master.check(20261031181100),
            Err(CoreError::MasterRealmNotTransferable)
        ));
    }

    #[test]
    fn a_passphrase_is_long_printable_ascii() {
        assert!(BundlePassphrase::try_from("correct horse battery".to_string()).is_ok());
        assert!(BundlePassphrase::try_from("short".to_string()).is_err());
        assert!(BundlePassphrase::try_from("correct\nhorse battery".to_string()).is_err());
        assert!(BundlePassphrase::try_from("cheval correct à pile".to_string()).is_err());
        assert!(BundlePassphrase::try_from("x".repeat(1025)).is_err());
    }

    #[test]
    fn a_passphrase_never_shows_in_debug_output() {
        let passphrase = BundlePassphrase::try_from("correct horse battery".to_string()).unwrap();

        assert!(!format!("{passphrase:?}").contains("horse"));
    }

    #[test]
    fn every_table_appears_once() {
        let mut tables = BUNDLE_TABLES.to_vec();
        tables.sort_unstable();
        tables.dedup();

        assert_eq!(tables.len(), BUNDLE_TABLES.len());
        assert_eq!(BUNDLE_TABLES[0], "realms");
        assert_eq!(BUNDLE_TABLES[1], "jwt_keys");
    }
}
