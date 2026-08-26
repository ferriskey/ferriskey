use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
use utoipa::openapi::RefOr;
use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
use utoipa::{PartialSchema, ToSchema};
use uuid::Uuid;

use crate::common::app_errors::CoreError;
use crate::generate_timestamp;
use crate::realm::RealmId;

pub const MAX_SP_ENTITY_ID_LENGTH: usize = 1024;
pub const MAX_ACS_URL_LENGTH: usize = 2048;
pub const MAX_SAML_ATTRIBUTE_NAME_LENGTH: usize = 255;
pub const MAX_USER_ATTRIBUTE_KEY_LENGTH: usize = 255;

fn is_forbidden_in_an_xml_attribute_value(character: char) -> bool {
    character.is_control()
        || character.is_whitespace()
        || matches!(character, '<' | '>' | '&' | '"' | '\'')
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidSamlConfig {
    #[error("a service provider entity id cannot be empty")]
    EmptyEntityId,

    #[error("`{0}` is not an absolute URI")]
    EntityIdIsNotAUri(String),

    #[error("a service provider entity id is limited to 1024 characters")]
    EntityIdTooLong,

    #[error("an assertion consumer service url cannot be empty")]
    EmptyAcsUrl,

    #[error("`{0}` is not an absolute URL")]
    AcsUrlIsNotAUrl(String),

    #[error("an assertion consumer service url is limited to 2048 characters")]
    AcsUrlTooLong,

    #[error("an assertion consumer service url must be http or https, got `{0}`")]
    AcsUrlUnsupportedScheme(String),

    #[error("an assertion consumer service url carries no credentials")]
    AcsUrlHasCredentials,

    #[error("an assertion consumer service url carries no fragment")]
    AcsUrlHasFragment,

    #[error("an assertion consumer service url carries no wildcard host")]
    AcsUrlHasWildcardHost,

    #[error("`{0}` is not a supported name id format")]
    UnsupportedNameIdFormat(String),
}

impl From<InvalidSamlConfig> for CoreError {
    fn from(error: InvalidSamlConfig) -> Self {
        CoreError::InvalidSamlConfig(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidSamlAttributeMapper {
    #[error("a saml attribute name cannot be empty")]
    EmptyName,

    #[error("a saml attribute name is limited to 255 characters")]
    NameTooLong,

    #[error("`{0}` carries a character that cannot appear in a saml attribute name")]
    NameHasForbiddenCharacter(String),

    #[error("`{0}` is not a supported attribute name format")]
    UnsupportedNameFormat(String),

    #[error("`{0}` is not a supported attribute source")]
    UnsupportedSource(String),

    #[error("a user attribute key cannot be empty")]
    EmptyUserAttributeKey,

    #[error("a user attribute key is limited to 255 characters")]
    UserAttributeKeyTooLong,

    #[error("`{0}` carries a character that cannot appear in a user attribute key")]
    UserAttributeKeyHasForbiddenCharacter(String),
}

impl From<InvalidSamlAttributeMapper> for CoreError {
    fn from(error: InvalidSamlAttributeMapper) -> Self {
        CoreError::InvalidSamlAttributeMapper(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SpEntityId(String);

impl SpEntityId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialSchema for SpEntityId {
    fn schema() -> RefOr<Schema> {
        String::schema()
    }
}

impl ToSchema for SpEntityId {}

impl FromStr for SpEntityId {
    type Err = InvalidSamlConfig;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.trim();

        if value.is_empty() {
            return Err(InvalidSamlConfig::EmptyEntityId);
        }

        if value.chars().count() > MAX_SP_ENTITY_ID_LENGTH {
            return Err(InvalidSamlConfig::EntityIdTooLong);
        }

        Url::parse(value).map_err(|_| InvalidSamlConfig::EntityIdIsNotAUri(value.to_string()))?;

        Ok(Self(value.to_string()))
    }
}

impl TryFrom<String> for SpEntityId {
    type Error = InvalidSamlConfig;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<SpEntityId> for String {
    fn from(value: SpEntityId) -> Self {
        value.0
    }
}

impl fmt::Display for SpEntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AcsUrl(String);

impl AcsUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialSchema for AcsUrl {
    fn schema() -> RefOr<Schema> {
        String::schema()
    }
}

impl ToSchema for AcsUrl {}

impl FromStr for AcsUrl {
    type Err = InvalidSamlConfig;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.trim();

        if value.is_empty() {
            return Err(InvalidSamlConfig::EmptyAcsUrl);
        }

        if value.chars().count() > MAX_ACS_URL_LENGTH {
            return Err(InvalidSamlConfig::AcsUrlTooLong);
        }

        let url =
            Url::parse(value).map_err(|_| InvalidSamlConfig::AcsUrlIsNotAUrl(value.to_string()))?;

        match url.scheme() {
            "http" | "https" => {}
            other => {
                return Err(InvalidSamlConfig::AcsUrlUnsupportedScheme(
                    other.to_string(),
                ));
            }
        }

        if !url.username().is_empty() || url.password().is_some() {
            return Err(InvalidSamlConfig::AcsUrlHasCredentials);
        }

        if url.fragment().is_some() {
            return Err(InvalidSamlConfig::AcsUrlHasFragment);
        }

        match url.host_str() {
            None => return Err(InvalidSamlConfig::AcsUrlIsNotAUrl(value.to_string())),
            Some(host) if host.contains('*') => {
                return Err(InvalidSamlConfig::AcsUrlHasWildcardHost);
            }
            Some(_) => {}
        }

        Ok(Self(url.to_string()))
    }
}

impl TryFrom<String> for AcsUrl {
    type Error = InvalidSamlConfig;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<AcsUrl> for String {
    fn from(value: AcsUrl) -> Self {
        value.0
    }
}

impl fmt::Display for AcsUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub enum NameIdFormat {
    #[default]
    EmailAddress,
    Persistent,
    Transient,
    Unspecified,
}

impl NameIdFormat {
    pub const EMAIL_ADDRESS_URN: &'static str =
        "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress";
    pub const PERSISTENT_URN: &'static str = "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent";
    pub const TRANSIENT_URN: &'static str = "urn:oasis:names:tc:SAML:2.0:nameid-format:transient";
    pub const UNSPECIFIED_URN: &'static str =
        "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified";

    pub fn as_urn(&self) -> &'static str {
        match self {
            Self::EmailAddress => Self::EMAIL_ADDRESS_URN,
            Self::Persistent => Self::PERSISTENT_URN,
            Self::Transient => Self::TRANSIENT_URN,
            Self::Unspecified => Self::UNSPECIFIED_URN,
        }
    }
}

impl PartialSchema for NameIdFormat {
    fn schema() -> RefOr<Schema> {
        ObjectBuilder::new()
            .schema_type(Type::String)
            .enum_values(Some([
                NameIdFormat::EMAIL_ADDRESS_URN,
                NameIdFormat::PERSISTENT_URN,
                NameIdFormat::TRANSIENT_URN,
                NameIdFormat::UNSPECIFIED_URN,
            ]))
            .into()
    }
}

impl ToSchema for NameIdFormat {}

impl FromStr for NameIdFormat {
    type Err = InvalidSamlConfig;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            Self::EMAIL_ADDRESS_URN | "email" | "emailAddress" => Ok(Self::EmailAddress),
            Self::PERSISTENT_URN | "persistent" => Ok(Self::Persistent),
            Self::TRANSIENT_URN | "transient" => Ok(Self::Transient),
            Self::UNSPECIFIED_URN | "unspecified" => Ok(Self::Unspecified),
            other => Err(InvalidSamlConfig::UnsupportedNameIdFormat(
                other.to_string(),
            )),
        }
    }
}

impl TryFrom<String> for NameIdFormat {
    type Error = InvalidSamlConfig;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<NameIdFormat> for String {
    fn from(value: NameIdFormat) -> Self {
        value.as_urn().to_string()
    }
}

impl fmt::Display for NameIdFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_urn())
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub enum SamlAttributeNameFormat {
    #[default]
    Basic,
    Uri,
    Unspecified,
}

impl SamlAttributeNameFormat {
    pub const BASIC_URN: &'static str = "urn:oasis:names:tc:SAML:2.0:attrname-format:basic";
    pub const URI_URN: &'static str = "urn:oasis:names:tc:SAML:2.0:attrname-format:uri";
    pub const UNSPECIFIED_URN: &'static str =
        "urn:oasis:names:tc:SAML:2.0:attrname-format:unspecified";

    pub fn as_urn(&self) -> &'static str {
        match self {
            Self::Basic => Self::BASIC_URN,
            Self::Uri => Self::URI_URN,
            Self::Unspecified => Self::UNSPECIFIED_URN,
        }
    }
}

impl PartialSchema for SamlAttributeNameFormat {
    fn schema() -> RefOr<Schema> {
        ObjectBuilder::new()
            .schema_type(Type::String)
            .enum_values(Some([
                SamlAttributeNameFormat::BASIC_URN,
                SamlAttributeNameFormat::URI_URN,
                SamlAttributeNameFormat::UNSPECIFIED_URN,
            ]))
            .into()
    }
}

impl ToSchema for SamlAttributeNameFormat {}

impl FromStr for SamlAttributeNameFormat {
    type Err = InvalidSamlAttributeMapper;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            Self::BASIC_URN | "basic" => Ok(Self::Basic),
            Self::URI_URN | "uri" => Ok(Self::Uri),
            Self::UNSPECIFIED_URN | "unspecified" => Ok(Self::Unspecified),
            other => Err(InvalidSamlAttributeMapper::UnsupportedNameFormat(
                other.to_string(),
            )),
        }
    }
}

impl TryFrom<String> for SamlAttributeNameFormat {
    type Error = InvalidSamlAttributeMapper;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<SamlAttributeNameFormat> for String {
    fn from(value: SamlAttributeNameFormat) -> Self {
        value.as_urn().to_string()
    }
}

impl fmt::Display for SamlAttributeNameFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_urn())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SamlAttributeName(String);

impl SamlAttributeName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialSchema for SamlAttributeName {
    fn schema() -> RefOr<Schema> {
        String::schema()
    }
}

impl ToSchema for SamlAttributeName {}

impl FromStr for SamlAttributeName {
    type Err = InvalidSamlAttributeMapper;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.trim();

        if value.is_empty() {
            return Err(InvalidSamlAttributeMapper::EmptyName);
        }

        if value.chars().count() > MAX_SAML_ATTRIBUTE_NAME_LENGTH {
            return Err(InvalidSamlAttributeMapper::NameTooLong);
        }

        if value.chars().any(is_forbidden_in_an_xml_attribute_value) {
            return Err(InvalidSamlAttributeMapper::NameHasForbiddenCharacter(
                value.to_string(),
            ));
        }

        Ok(Self(value.to_string()))
    }
}

impl TryFrom<String> for SamlAttributeName {
    type Error = InvalidSamlAttributeMapper;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<SamlAttributeName> for String {
    fn from(value: SamlAttributeName) -> Self {
        value.0
    }
}

impl fmt::Display for SamlAttributeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct UserAttributeKey(String);

impl UserAttributeKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialSchema for UserAttributeKey {
    fn schema() -> RefOr<Schema> {
        String::schema()
    }
}

impl ToSchema for UserAttributeKey {}

impl FromStr for UserAttributeKey {
    type Err = InvalidSamlAttributeMapper;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.trim();

        if value.is_empty() {
            return Err(InvalidSamlAttributeMapper::EmptyUserAttributeKey);
        }

        if value.chars().count() > MAX_USER_ATTRIBUTE_KEY_LENGTH {
            return Err(InvalidSamlAttributeMapper::UserAttributeKeyTooLong);
        }

        if value.chars().any(is_forbidden_in_an_xml_attribute_value) {
            return Err(
                InvalidSamlAttributeMapper::UserAttributeKeyHasForbiddenCharacter(
                    value.to_string(),
                ),
            );
        }

        Ok(Self(value.to_string()))
    }
}

impl TryFrom<String> for UserAttributeKey {
    type Error = InvalidSamlAttributeMapper;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<UserAttributeKey> for String {
    fn from(value: UserAttributeKey) -> Self {
        value.0
    }
}

impl fmt::Display for UserAttributeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SamlAttributeSource {
    UserId,
    Username,
    Email,
    FirstName,
    LastName,
    UserAttribute(UserAttributeKey),
}

impl SamlAttributeSource {
    pub const USER_PREFIX: &'static str = "user:";
    pub const ATTRIBUTE_PREFIX: &'static str = "attribute:";
}

impl PartialSchema for SamlAttributeSource {
    fn schema() -> RefOr<Schema> {
        String::schema()
    }
}

impl ToSchema for SamlAttributeSource {}

impl FromStr for SamlAttributeSource {
    type Err = InvalidSamlAttributeMapper;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.trim();

        if let Some(key) = value.strip_prefix(Self::ATTRIBUTE_PREFIX) {
            return UserAttributeKey::from_str(key).map(Self::UserAttribute);
        }

        match value.strip_prefix(Self::USER_PREFIX) {
            Some("id") => Ok(Self::UserId),
            Some("username") => Ok(Self::Username),
            Some("email") => Ok(Self::Email),
            Some("first_name") => Ok(Self::FirstName),
            Some("last_name") => Ok(Self::LastName),
            _ => Err(InvalidSamlAttributeMapper::UnsupportedSource(
                value.to_string(),
            )),
        }
    }
}

impl TryFrom<String> for SamlAttributeSource {
    type Error = InvalidSamlAttributeMapper;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<SamlAttributeSource> for String {
    fn from(value: SamlAttributeSource) -> Self {
        value.to_string()
    }
}

impl fmt::Display for SamlAttributeSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserId => write!(f, "{}id", Self::USER_PREFIX),
            Self::Username => write!(f, "{}username", Self::USER_PREFIX),
            Self::Email => write!(f, "{}email", Self::USER_PREFIX),
            Self::FirstName => write!(f, "{}first_name", Self::USER_PREFIX),
            Self::LastName => write!(f, "{}last_name", Self::USER_PREFIX),
            Self::UserAttribute(key) => write!(f, "{}{key}", Self::ATTRIBUTE_PREFIX),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SamlConfigSettings {
    pub sp_entity_id: SpEntityId,
    pub acs_url: AcsUrl,
    pub name_id_format: NameIdFormat,
    pub sign_assertions: bool,
    pub sign_documents: bool,
    pub want_authn_requests_signed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClientSamlConfig {
    pub client_id: Uuid,
    pub realm_id: RealmId,
    pub sp_entity_id: SpEntityId,
    pub acs_url: AcsUrl,
    pub name_id_format: NameIdFormat,
    pub sign_assertions: bool,
    pub sign_documents: bool,
    pub want_authn_requests_signed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ClientSamlConfig {
    pub fn new(realm_id: RealmId, client_id: Uuid, settings: SamlConfigSettings) -> Self {
        let (now, _) = generate_timestamp();

        Self {
            client_id,
            realm_id,
            sp_entity_id: settings.sp_entity_id,
            acs_url: settings.acs_url,
            name_id_format: settings.name_id_format,
            sign_assertions: settings.sign_assertions,
            sign_documents: settings.sign_documents,
            want_authn_requests_signed: settings.want_authn_requests_signed,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SamlAttributeMapperDefinition {
    pub name: SamlAttributeName,
    pub name_format: SamlAttributeNameFormat,
    pub source: SamlAttributeSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SamlAttributeMapper {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: SamlAttributeName,
    pub name_format: SamlAttributeNameFormat,
    pub source: SamlAttributeSource,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SamlAttributeMapper {
    pub fn new(client_id: Uuid, definition: SamlAttributeMapperDefinition) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            client_id,
            name: definition.name,
            name_format: definition.name_format,
            source: definition.source,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity_id(value: &str) -> SpEntityId {
        SpEntityId::from_str(value).expect("test fixture must be a valid entity id")
    }

    fn acs(value: &str) -> AcsUrl {
        AcsUrl::from_str(value).expect("test fixture must be a valid acs url")
    }

    fn attribute_name(value: &str) -> SamlAttributeName {
        SamlAttributeName::from_str(value).expect("test fixture must be a valid attribute name")
    }

    #[test]
    fn an_entity_id_keeps_its_exact_bytes_because_saml_compares_them_verbatim() {
        assert_eq!(
            entity_id("https://Chat.Example.COM/saml/sp/1").as_str(),
            "https://Chat.Example.COM/saml/sp/1"
        );
    }

    #[test]
    fn an_entity_id_is_trimmed_of_surrounding_whitespace() {
        assert_eq!(
            entity_id("  https://chat.example.com/saml/sp/1  ").as_str(),
            "https://chat.example.com/saml/sp/1"
        );
    }

    #[test]
    fn an_entity_id_may_be_a_urn() {
        assert_eq!(
            entity_id("urn:federation:example").as_str(),
            "urn:federation:example"
        );
    }

    #[test]
    fn an_empty_entity_id_is_rejected() {
        assert_eq!(
            SpEntityId::from_str("   "),
            Err(InvalidSamlConfig::EmptyEntityId)
        );
    }

    #[test]
    fn a_relative_entity_id_is_rejected() {
        assert_eq!(
            SpEntityId::from_str("/saml/sp/1"),
            Err(InvalidSamlConfig::EntityIdIsNotAUri(
                "/saml/sp/1".to_string()
            ))
        );
    }

    #[test]
    fn an_overlong_entity_id_is_rejected() {
        let value = format!("https://chat.example.com/{}", "a".repeat(1024));

        assert_eq!(
            SpEntityId::from_str(&value),
            Err(InvalidSamlConfig::EntityIdTooLong)
        );
    }

    #[test]
    fn deserializing_an_entity_id_cannot_smuggle_a_relative_uri_past_validation() {
        assert!(serde_json::from_str::<SpEntityId>("\"/saml/sp/1\"").is_err());
    }

    #[test]
    fn an_acs_url_keeps_the_query_string_chatwoot_carries_its_account_id_in() {
        assert_eq!(
            acs("https://chat.example.com/omniauth/saml/callback?account_id=7").as_str(),
            "https://chat.example.com/omniauth/saml/callback?account_id=7"
        );
    }

    #[test]
    fn an_acs_url_is_normalized_by_the_url_parser() {
        assert_eq!(
            acs("HTTPS://Chat.Example.COM:443/omniauth/saml/callback").as_str(),
            "https://chat.example.com/omniauth/saml/callback"
        );
    }

    #[test]
    fn an_empty_acs_url_is_rejected() {
        assert_eq!(AcsUrl::from_str("  "), Err(InvalidSamlConfig::EmptyAcsUrl));
    }

    #[test]
    fn a_relative_acs_url_is_rejected() {
        assert_eq!(
            AcsUrl::from_str("/omniauth/saml/callback"),
            Err(InvalidSamlConfig::AcsUrlIsNotAUrl(
                "/omniauth/saml/callback".to_string()
            ))
        );
    }

    #[test]
    fn a_non_http_acs_url_is_rejected() {
        assert_eq!(
            AcsUrl::from_str("javascript:alert(1)"),
            Err(InvalidSamlConfig::AcsUrlUnsupportedScheme(
                "javascript".to_string()
            ))
        );
    }

    #[test]
    fn a_file_acs_url_is_rejected() {
        assert_eq!(
            AcsUrl::from_str("file:///etc/passwd"),
            Err(InvalidSamlConfig::AcsUrlUnsupportedScheme(
                "file".to_string()
            ))
        );
    }

    #[test]
    fn an_acs_url_carrying_credentials_is_rejected() {
        assert_eq!(
            AcsUrl::from_str("https://user:secret@chat.example.com/cb"),
            Err(InvalidSamlConfig::AcsUrlHasCredentials)
        );
    }

    #[test]
    fn an_acs_url_carrying_a_fragment_is_rejected() {
        assert_eq!(
            AcsUrl::from_str("https://chat.example.com/cb#top"),
            Err(InvalidSamlConfig::AcsUrlHasFragment)
        );
    }

    #[test]
    fn an_acs_url_carrying_a_wildcard_host_is_rejected() {
        assert_eq!(
            AcsUrl::from_str("https://*.example.com/cb"),
            Err(InvalidSamlConfig::AcsUrlHasWildcardHost)
        );
    }

    #[test]
    fn deserializing_an_acs_url_cannot_smuggle_a_javascript_scheme_past_validation() {
        assert!(serde_json::from_str::<AcsUrl>("\"javascript:alert(1)\"").is_err());
    }

    #[test]
    fn a_name_id_format_parses_from_its_urn() {
        assert_eq!(
            NameIdFormat::from_str(NameIdFormat::PERSISTENT_URN),
            Ok(NameIdFormat::Persistent)
        );
    }

    #[test]
    fn a_name_id_format_parses_from_its_short_alias() {
        assert_eq!(
            NameIdFormat::from_str("email"),
            Ok(NameIdFormat::EmailAddress)
        );
        assert_eq!(
            NameIdFormat::from_str("transient"),
            Ok(NameIdFormat::Transient)
        );
    }

    #[test]
    fn a_name_id_format_renders_back_to_its_urn() {
        assert_eq!(
            NameIdFormat::Transient.to_string(),
            "urn:oasis:names:tc:SAML:2.0:nameid-format:transient"
        );
    }

    #[test]
    fn an_unknown_name_id_format_is_rejected() {
        assert_eq!(
            NameIdFormat::from_str("kerberos"),
            Err(InvalidSamlConfig::UnsupportedNameIdFormat(
                "kerberos".to_string()
            ))
        );
    }

    #[test]
    fn a_name_id_format_is_documented_as_the_urns_it_serializes_to() {
        let schema = serde_json::to_value(NameIdFormat::schema())
            .expect("a utoipa schema serializes to json");

        assert_eq!(schema["type"], "string");
        assert_eq!(
            schema["enum"][0],
            "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"
        );
    }

    #[test]
    fn deserializing_a_name_id_format_rejects_what_parsing_rejects() {
        assert!(serde_json::from_str::<NameIdFormat>("\"kerberos\"").is_err());
    }

    #[test]
    fn an_attribute_name_format_parses_from_its_urn_and_its_alias() {
        assert_eq!(
            SamlAttributeNameFormat::from_str(SamlAttributeNameFormat::URI_URN),
            Ok(SamlAttributeNameFormat::Uri)
        );
        assert_eq!(
            SamlAttributeNameFormat::from_str("basic"),
            Ok(SamlAttributeNameFormat::Basic)
        );
    }

    #[test]
    fn an_unknown_attribute_name_format_is_rejected() {
        assert_eq!(
            SamlAttributeNameFormat::from_str("xacml"),
            Err(InvalidSamlAttributeMapper::UnsupportedNameFormat(
                "xacml".to_string()
            ))
        );
    }

    #[test]
    fn an_attribute_name_may_be_a_bare_word_as_chatwoot_expects() {
        assert_eq!(attribute_name("first_name").as_str(), "first_name");
    }

    #[test]
    fn an_attribute_name_may_be_a_claim_uri() {
        let name = "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress";

        assert_eq!(attribute_name(name).as_str(), name);
    }

    #[test]
    fn an_empty_attribute_name_is_rejected() {
        assert_eq!(
            SamlAttributeName::from_str("  "),
            Err(InvalidSamlAttributeMapper::EmptyName)
        );
    }

    #[test]
    fn an_overlong_attribute_name_is_rejected() {
        assert_eq!(
            SamlAttributeName::from_str(&"a".repeat(256)),
            Err(InvalidSamlAttributeMapper::NameTooLong)
        );
    }

    #[test]
    fn an_attribute_name_carrying_xml_syntax_is_rejected_at_the_boundary() {
        for value in ["a<b", "a>b", "a&b", "a\"b", "a'b"] {
            assert_eq!(
                SamlAttributeName::from_str(value),
                Err(InvalidSamlAttributeMapper::NameHasForbiddenCharacter(
                    value.to_string()
                )),
                "`{value}` must not reach an xml serializer"
            );
        }
    }

    #[test]
    fn an_attribute_name_carrying_inner_whitespace_is_rejected() {
        assert_eq!(
            SamlAttributeName::from_str("first name"),
            Err(InvalidSamlAttributeMapper::NameHasForbiddenCharacter(
                "first name".to_string()
            ))
        );
    }

    #[test]
    fn an_attribute_name_carrying_a_control_character_is_rejected() {
        assert_eq!(
            SamlAttributeName::from_str("first\u{0}name"),
            Err(InvalidSamlAttributeMapper::NameHasForbiddenCharacter(
                "first\u{0}name".to_string()
            ))
        );
    }

    #[test]
    fn deserializing_an_attribute_name_cannot_smuggle_xml_syntax_past_validation() {
        assert!(serde_json::from_str::<SamlAttributeName>("\"<script>\"").is_err());
    }

    #[test]
    fn every_built_in_source_round_trips_through_its_stored_form() {
        for source in [
            SamlAttributeSource::UserId,
            SamlAttributeSource::Username,
            SamlAttributeSource::Email,
            SamlAttributeSource::FirstName,
            SamlAttributeSource::LastName,
        ] {
            let stored = source.to_string();

            assert_eq!(SamlAttributeSource::from_str(&stored), Ok(source.clone()));
        }
    }

    #[test]
    fn a_built_in_source_is_stored_under_its_user_prefix() {
        assert_eq!(
            SamlAttributeSource::FirstName.to_string(),
            "user:first_name"
        );
    }

    #[test]
    fn a_custom_source_round_trips_through_its_stored_form() {
        let stored = SamlAttributeSource::from_str("attribute:department")
            .expect("a custom attribute key parses");

        assert_eq!(stored.to_string(), "attribute:department");
        assert_eq!(
            stored,
            SamlAttributeSource::UserAttribute(
                UserAttributeKey::from_str("department").expect("a valid key")
            )
        );
    }

    #[test]
    fn a_custom_source_key_may_itself_contain_the_separator() {
        let source =
            SamlAttributeSource::from_str("attribute:urn:oid:2.5.4.42").expect("a colon-rich key");

        assert_eq!(source.to_string(), "attribute:urn:oid:2.5.4.42");
    }

    #[test]
    fn an_unprefixed_source_is_rejected() {
        assert_eq!(
            SamlAttributeSource::from_str("email"),
            Err(InvalidSamlAttributeMapper::UnsupportedSource(
                "email".to_string()
            ))
        );
    }

    #[test]
    fn an_unknown_built_in_source_is_rejected() {
        assert_eq!(
            SamlAttributeSource::from_str("user:password"),
            Err(InvalidSamlAttributeMapper::UnsupportedSource(
                "user:password".to_string()
            ))
        );
    }

    #[test]
    fn a_custom_source_carrying_an_empty_key_is_rejected() {
        assert_eq!(
            SamlAttributeSource::from_str("attribute:"),
            Err(InvalidSamlAttributeMapper::EmptyUserAttributeKey)
        );
    }

    #[test]
    fn a_custom_source_carrying_xml_syntax_in_its_key_is_rejected() {
        assert_eq!(
            SamlAttributeSource::from_str("attribute:<script>"),
            Err(
                InvalidSamlAttributeMapper::UserAttributeKeyHasForbiddenCharacter(
                    "<script>".to_string()
                )
            )
        );
    }

    #[test]
    fn deserializing_a_source_cannot_smuggle_an_unknown_property_past_validation() {
        assert!(serde_json::from_str::<SamlAttributeSource>("\"user:password\"").is_err());
    }

    #[test]
    fn a_source_serializes_to_the_same_form_it_is_stored_as() {
        assert_eq!(
            serde_json::to_string(&SamlAttributeSource::LastName)
                .expect("serialization is infallible for a String"),
            "\"user:last_name\""
        );
    }

    #[test]
    fn a_rejected_config_reaches_the_caller_as_a_core_error_carrying_the_reason() {
        let error =
            AcsUrl::from_str("javascript:alert(1)").expect_err("a javascript url is rejected");
        let reason = error.to_string();

        let converted = CoreError::from(error);

        assert!(matches!(&converted, CoreError::InvalidSamlConfig(_)));
        assert!(converted.to_string().contains(&reason));
    }

    #[test]
    fn a_rejected_mapper_reaches_the_caller_as_a_core_error_carrying_the_reason() {
        let error = SamlAttributeName::from_str("<script>").expect_err("xml syntax is rejected");
        let reason = error.to_string();

        let converted = CoreError::from(error);

        assert!(matches!(
            &converted,
            CoreError::InvalidSamlAttributeMapper(_)
        ));
        assert!(converted.to_string().contains(&reason));
    }

    #[test]
    fn a_config_carries_the_realm_and_client_it_was_built_for() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let client_id = Uuid::new_v4();
        let config = ClientSamlConfig::new(
            realm_id,
            client_id,
            SamlConfigSettings {
                sp_entity_id: entity_id("https://chat.example.com/saml/sp/1"),
                acs_url: acs("https://chat.example.com/omniauth/saml/callback?account_id=1"),
                name_id_format: NameIdFormat::EmailAddress,
                sign_assertions: true,
                sign_documents: false,
                want_authn_requests_signed: false,
            },
        );

        assert_eq!(config.client_id, client_id);
        assert_eq!(
            config.realm_id, realm_id,
            "the realm is what scopes the entity id uniqueness, so it must survive construction"
        );
        assert_eq!(config.created_at, config.updated_at);
    }

    #[test]
    fn a_mapper_is_identified_by_a_time_ordered_uuid() {
        let mapper = SamlAttributeMapper::new(
            Uuid::new_v4(),
            SamlAttributeMapperDefinition {
                name: attribute_name("email"),
                name_format: SamlAttributeNameFormat::Basic,
                source: SamlAttributeSource::Email,
            },
        );

        assert_eq!(mapper.id.get_version_num(), 7);
    }
}
