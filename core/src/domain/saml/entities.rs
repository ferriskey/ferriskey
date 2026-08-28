use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use ferriskey_saml::authn::{
    AbsoluteUri, AuthnError, Issuer, NameIdFormat as AssertionNameIdFormat, RequestId,
};
use ferriskey_saml::response::{
    AssertionAttribute, AssertionWindow, AttributeNameFormat, AuthnContextClassRef,
    ResponseDescriptor,
};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::authentication::entities::AuthSession;
use crate::domain::client::entities::saml::{
    AcsUrl, NameIdFormat, SamlAttributeMapper, SamlAttributeNameFormat, SamlAttributeSource,
    SpEntityId,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::user::entities::{User, UserAttribute};

pub const ASSERTION_LIFETIME_SECONDS: i64 = 300;
pub const ASSERTION_CLOCK_SKEW_SECONDS: i64 = 60;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SamlSsoError {
    #[error("the authn request could not be parsed: {0}")]
    MalformedAuthnRequest(String),

    #[error("`{requested}` cannot be an assertion consumer service url: {reason}")]
    UnusableAcsUrl { requested: String, reason: String },

    #[error(
        "the requested assertion consumer service url `{requested}` is not the `{registered}` registered for this service provider"
    )]
    AcsUrlMismatch {
        requested: String,
        registered: String,
    },

    #[error("no service provider is registered under the issuer `{0}` in this realm")]
    UnknownServiceProvider(String),

    #[error("the authorization code does not belong to a saml authentication")]
    NotASamlAuthentication,

    #[error("the authentication carries no authn request id to answer")]
    MissingAuthnRequestId,

    #[error("a `{format}` name id needs a {requirement} the user does not have")]
    MissingNameIdSource {
        format: &'static str,
        requirement: &'static str,
    },

    #[error("the assertion could not be built: {0}")]
    UnusableAssertion(String),
}

impl From<SamlSsoError> for CoreError {
    fn from(error: SamlSsoError) -> Self {
        match error {
            SamlSsoError::MalformedAuthnRequest(_) => CoreError::InvalidRequest,
            SamlSsoError::UnusableAcsUrl { .. } | SamlSsoError::AcsUrlMismatch { .. } => {
                CoreError::InvalidRedirectUri
            }
            SamlSsoError::UnknownServiceProvider(_) => CoreError::SamlConfigNotFound,
            SamlSsoError::NotASamlAuthentication => CoreError::InvalidAuthorizationCode,
            SamlSsoError::MissingAuthnRequestId => CoreError::InvalidSession,
            SamlSsoError::MissingNameIdSource { .. } => CoreError::InvalidUser,
            SamlSsoError::UnusableAssertion(_) => CoreError::InternalServerError,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StartSsoInput {
    pub realm_name: String,
    pub authn_request: String,
    pub relay_state: Option<String>,
    pub public_base_url: String,
}

#[derive(Debug, Clone)]
pub struct FinishSsoInput {
    pub realm_name: String,
    pub authorization_code: String,
    pub public_base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SamlAssertionDelivery {
    pub acs_url: AcsUrl,
    pub signed_response: String,
    pub relay_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectNameId {
    pub value: String,
    pub format: AssertionNameIdFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionBlueprint {
    pub response_id: RequestId,
    pub assertion_id: RequestId,
    pub in_response_to: RequestId,
    pub idp_entity_id: String,
    pub sp_entity_id: SpEntityId,
    pub acs_url: AcsUrl,
    pub name_id: SubjectNameId,
    pub session_index: String,
    pub attributes: Vec<AssertionAttribute>,
    pub issued_at: DateTime<Utc>,
}

pub fn idp_entity_id(public_base_url: &str, realm_name: &str) -> String {
    format!(
        "{}/realms/{realm_name}",
        public_base_url.trim_end_matches('/')
    )
}

pub fn sso_continue_url(public_base_url: &str, realm_name: &str) -> String {
    format!(
        "{}/realms/{realm_name}/protocol/saml/continue",
        public_base_url.trim_end_matches('/')
    )
}

pub fn generate_element_id() -> Result<RequestId, SamlSsoError> {
    RequestId::parse(&format!("_{}", Uuid::new_v4().simple())).map_err(unusable_assertion)
}

pub fn record_authn_request_id(id: &RequestId) -> Option<String> {
    Some(id.as_str().to_owned())
}

pub fn recorded_authn_request_id(auth_session: &AuthSession) -> Result<RequestId, SamlSsoError> {
    let recorded = auth_session
        .nonce
        .as_deref()
        .ok_or(SamlSsoError::MissingAuthnRequestId)?;

    RequestId::parse(recorded).map_err(|_| SamlSsoError::MissingAuthnRequestId)
}

pub fn resolve_assertion_consumer_service_url(
    requested: Option<&str>,
    registered: &AcsUrl,
) -> Result<AcsUrl, SamlSsoError> {
    match requested {
        None => Ok(registered.clone()),
        Some(requested) => {
            let candidate =
                AcsUrl::from_str(requested).map_err(|reason| SamlSsoError::UnusableAcsUrl {
                    requested: requested.to_owned(),
                    reason: reason.to_string(),
                })?;

            if candidate != *registered {
                return Err(SamlSsoError::AcsUrlMismatch {
                    requested: candidate.to_string(),
                    registered: registered.to_string(),
                });
            }

            Ok(candidate)
        }
    }
}

pub fn derive_name_id(
    format: NameIdFormat,
    user: &User,
    transient_identifier: &str,
) -> Result<SubjectNameId, SamlSsoError> {
    let (value, format) = match format {
        NameIdFormat::EmailAddress => (
            non_empty(user.email.as_deref())
                .filter(|_| user.email_verified)
                .ok_or(SamlSsoError::MissingNameIdSource {
                    format: "emailAddress",
                    requirement: "verified email address",
                })?,
            AssertionNameIdFormat::EmailAddress,
        ),
        NameIdFormat::Persistent => (user.id.to_string(), AssertionNameIdFormat::Persistent),
        NameIdFormat::Transient => (
            non_empty(Some(transient_identifier)).ok_or(SamlSsoError::MissingNameIdSource {
                format: "transient",
                requirement: "session identifier",
            })?,
            AssertionNameIdFormat::Transient,
        ),
        NameIdFormat::Unspecified => (
            non_empty(Some(user.username.as_str())).ok_or(SamlSsoError::MissingNameIdSource {
                format: "unspecified",
                requirement: "username",
            })?,
            AssertionNameIdFormat::Unspecified,
        ),
    };

    Ok(SubjectNameId { value, format })
}

pub fn needs_user_attributes(mappers: &[SamlAttributeMapper]) -> bool {
    mappers
        .iter()
        .any(|mapper| matches!(mapper.source, SamlAttributeSource::UserAttribute(_)))
}

pub fn build_assertion_attributes(
    mappers: &[SamlAttributeMapper],
    user: &User,
    user_attributes: &[UserAttribute],
) -> Vec<AssertionAttribute> {
    mappers
        .iter()
        .filter_map(|mapper| {
            let values = resolve_attribute_values(&mapper.source, user, user_attributes);

            (!values.is_empty()).then(|| AssertionAttribute {
                name: mapper.name.to_string(),
                name_format: assertion_name_format(mapper.name_format),
                values,
            })
        })
        .collect()
}

pub fn build_response_descriptor(
    blueprint: AssertionBlueprint,
) -> Result<ResponseDescriptor, SamlSsoError> {
    let issued_at = blueprint.issued_at;

    let window = AssertionWindow::new(
        issued_at - Duration::seconds(ASSERTION_CLOCK_SKEW_SECONDS),
        issued_at + Duration::seconds(ASSERTION_LIFETIME_SECONDS),
        issued_at + Duration::seconds(ASSERTION_LIFETIME_SECONDS),
    )
    .map_err(unusable_assertion)?;

    Ok(ResponseDescriptor {
        response_id: blueprint.response_id,
        assertion_id: blueprint.assertion_id,
        in_response_to: blueprint.in_response_to,
        issuer: Issuer::parse(&blueprint.idp_entity_id).map_err(unusable_assertion)?,
        destination: AbsoluteUri::parse("Destination", blueprint.acs_url.as_str())
            .map_err(unusable_assertion)?,
        audience: AbsoluteUri::parse("Audience", blueprint.sp_entity_id.as_str())
            .map_err(unusable_assertion)?,
        issue_instant: issued_at,
        authn_instant: issued_at,
        window,
        name_id: blueprint.name_id.value,
        name_id_format: blueprint.name_id.format,
        session_index: blueprint.session_index,
        authn_context: AuthnContextClassRef::PasswordProtectedTransport,
        attributes: blueprint.attributes,
    })
}

fn resolve_attribute_values(
    source: &SamlAttributeSource,
    user: &User,
    user_attributes: &[UserAttribute],
) -> Vec<String> {
    match source {
        SamlAttributeSource::UserId => vec![user.id.to_string()],
        SamlAttributeSource::Username => non_empty(Some(user.username.as_str()))
            .into_iter()
            .collect(),
        SamlAttributeSource::Email => non_empty(user.email.as_deref()).into_iter().collect(),
        SamlAttributeSource::FirstName => {
            non_empty(user.firstname.as_deref()).into_iter().collect()
        }
        SamlAttributeSource::LastName => non_empty(user.lastname.as_deref()).into_iter().collect(),
        SamlAttributeSource::UserAttribute(key) => user_attributes
            .iter()
            .filter(|attribute| attribute.key == key.as_str())
            .filter_map(|attribute| non_empty(Some(attribute.value.as_str())))
            .collect(),
    }
}

fn assertion_name_format(format: SamlAttributeNameFormat) -> AttributeNameFormat {
    match format {
        SamlAttributeNameFormat::Basic => AttributeNameFormat::Basic,
        SamlAttributeNameFormat::Uri => AttributeNameFormat::Uri,
        SamlAttributeNameFormat::Unspecified => AttributeNameFormat::Unspecified,
    }
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn unusable_assertion(reason: impl std::fmt::Display) -> SamlSsoError {
    SamlSsoError::UnusableAssertion(reason.to_string())
}

impl From<AuthnError> for SamlSsoError {
    fn from(error: AuthnError) -> Self {
        SamlSsoError::MalformedAuthnRequest(error.to_string())
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    use crate::domain::realm::entities::RealmId;

    pub fn acs_url(value: &str) -> AcsUrl {
        AcsUrl::from_str(value).expect("the fixture must be a valid acs url")
    }

    pub fn sp_entity_id(value: &str) -> SpEntityId {
        SpEntityId::from_str(value).expect("the fixture must be a valid entity id")
    }

    pub fn request_id(value: &str) -> RequestId {
        RequestId::parse(value).expect("the fixture must be a valid ncname")
    }

    pub fn mapper(name: &str, source: SamlAttributeSource) -> SamlAttributeMapper {
        SamlAttributeMapper::new(
            Uuid::new_v4(),
            crate::domain::client::entities::saml::SamlAttributeMapperDefinition {
                name: crate::domain::client::entities::saml::SamlAttributeName::from_str(name)
                    .expect("the fixture must be a valid attribute name"),
                name_format: SamlAttributeNameFormat::Basic,
                source,
            },
        )
    }

    pub fn user(realm_id: RealmId) -> User {
        let now = Utc::now();

        User {
            id: Uuid::new_v4(),
            realm_id,
            client_id: None,
            username: "alice".to_string(),
            firstname: Some("Alice".to_string()),
            lastname: Some("Liddell".to_string()),
            email: Some("alice@example.com".to_string()),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: None,
            required_actions: Vec::new(),
            created_at: now,
            updated_at: now,
            failed_login_attempts: 0,
            locked_until: None,
        }
    }

    pub fn user_attribute(user: &User, key: &str, value: &str) -> UserAttribute {
        let now = Utc::now();

        UserAttribute {
            id: Uuid::new_v4(),
            user_id: user.id,
            realm_id: user.realm_id,
            key: key.to_string(),
            value: value.to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    use crate::domain::realm::entities::RealmId;

    fn registered_acs() -> AcsUrl {
        acs_url("https://chat.example.com/omniauth/saml/callback?account_id=1")
    }

    #[test]
    fn an_absent_acs_url_falls_back_to_the_registered_one() {
        let registered = registered_acs();

        assert_eq!(
            resolve_assertion_consumer_service_url(None, &registered),
            Ok(registered)
        );
    }

    #[test]
    fn a_requested_acs_url_equal_to_the_registered_one_is_accepted() {
        let registered = registered_acs();

        assert_eq!(
            resolve_assertion_consumer_service_url(Some(registered.as_str()), &registered),
            Ok(registered)
        );
    }

    #[test]
    fn an_attacker_chosen_acs_url_is_rejected_instead_of_receiving_the_assertion() {
        let registered = registered_acs();

        assert_eq!(
            resolve_assertion_consumer_service_url(
                Some("https://evil.example.com/steal"),
                &registered
            ),
            Err(SamlSsoError::AcsUrlMismatch {
                requested: "https://evil.example.com/steal".to_string(),
                registered: registered.to_string(),
            })
        );
    }

    #[test]
    fn an_acs_url_that_only_shares_the_registered_host_is_rejected() {
        let registered = registered_acs();

        assert!(
            matches!(
                resolve_assertion_consumer_service_url(
                    Some("https://chat.example.com/omniauth/saml/callback?account_id=999"),
                    &registered
                ),
                Err(SamlSsoError::AcsUrlMismatch { .. })
            ),
            "the query string carries the tenant, so it is part of the identity of the endpoint"
        );
    }

    #[test]
    fn a_requested_acs_url_is_compared_after_the_normalization_the_registered_one_went_through() {
        let registered = registered_acs();

        assert_eq!(
            resolve_assertion_consumer_service_url(
                Some("HTTPS://Chat.Example.COM:443/omniauth/saml/callback?account_id=1"),
                &registered
            ),
            Ok(registered)
        );
    }

    #[test]
    fn a_requested_acs_url_that_could_never_have_been_registered_is_rejected_before_comparison() {
        let registered = registered_acs();

        assert!(matches!(
            resolve_assertion_consumer_service_url(Some("javascript:alert(1)"), &registered),
            Err(SamlSsoError::UnusableAcsUrl { .. })
        ));
    }

    #[test]
    fn a_rejected_acs_url_never_reaches_the_caller_as_anything_but_a_refusal() {
        let rejection = SamlSsoError::AcsUrlMismatch {
            requested: "https://evil.example.com/steal".to_string(),
            registered: "https://chat.example.com/cb".to_string(),
        };

        assert!(matches!(
            CoreError::from(rejection),
            CoreError::InvalidRedirectUri
        ));
    }

    #[test]
    fn the_idp_entity_id_is_the_realm_root_whatever_the_base_url_trailing_slash() {
        assert_eq!(
            idp_entity_id("https://auth.example.com/", "master"),
            "https://auth.example.com/realms/master"
        );
        assert_eq!(
            idp_entity_id("https://auth.example.com", "master"),
            "https://auth.example.com/realms/master"
        );
    }

    #[test]
    fn the_continue_url_is_ours_so_the_assertion_is_issued_by_us_and_not_by_the_login_page() {
        assert_eq!(
            sso_continue_url("https://auth.example.com/", "master"),
            "https://auth.example.com/realms/master/protocol/saml/continue"
        );
    }

    #[test]
    fn a_generated_element_id_cannot_start_with_a_digit_because_an_xsd_id_is_an_ncname() {
        for _ in 0..64 {
            let id = generate_element_id().expect("a generated id must be a valid ncname");

            assert!(id.as_str().starts_with('_'));
            assert_eq!(id.as_str().len(), 33);
        }
    }

    #[test]
    fn two_generated_element_ids_never_collide_so_the_response_and_its_assertion_stay_distinct() {
        let first = generate_element_id().expect("a generated id must be a valid ncname");
        let second = generate_element_id().expect("a generated id must be a valid ncname");

        assert_ne!(first, second);
    }

    #[test]
    fn an_email_name_id_carries_the_address_the_service_provider_keys_accounts_on() {
        let user = user(RealmId::default());

        assert_eq!(
            derive_name_id(NameIdFormat::EmailAddress, &user, "session"),
            Ok(SubjectNameId {
                value: "alice@example.com".to_string(),
                format: AssertionNameIdFormat::EmailAddress,
            })
        );
    }

    #[test]
    fn an_email_name_id_is_refused_rather_than_emitted_empty_for_a_user_without_an_address() {
        let mut user = user(RealmId::default());
        user.email = None;

        assert_eq!(
            derive_name_id(NameIdFormat::EmailAddress, &user, "session"),
            Err(SamlSsoError::MissingNameIdSource {
                format: "emailAddress",
                requirement: "verified email address",
            })
        );
    }

    #[test]
    fn an_unverified_address_cannot_become_an_email_name_id() {
        let mut user = user(RealmId::default());
        user.email_verified = false;

        assert_eq!(
            derive_name_id(NameIdFormat::EmailAddress, &user, "session"),
            Err(SamlSsoError::MissingNameIdSource {
                format: "emailAddress",
                requirement: "verified email address",
            })
        );
    }

    #[test]
    fn a_persistent_name_id_is_the_user_id_so_it_survives_a_rename() {
        let user = user(RealmId::default());

        assert_eq!(
            derive_name_id(NameIdFormat::Persistent, &user, "session"),
            Ok(SubjectNameId {
                value: user.id.to_string(),
                format: AssertionNameIdFormat::Persistent,
            })
        );
    }

    #[test]
    fn a_transient_name_id_is_scoped_to_the_login_so_it_cannot_correlate_two_sessions() {
        let user = user(RealmId::default());

        assert_eq!(
            derive_name_id(NameIdFormat::Transient, &user, "one-login-only"),
            Ok(SubjectNameId {
                value: "one-login-only".to_string(),
                format: AssertionNameIdFormat::Transient,
            })
        );
    }

    #[test]
    fn an_unspecified_name_id_falls_back_to_the_username() {
        let user = user(RealmId::default());

        assert_eq!(
            derive_name_id(NameIdFormat::Unspecified, &user, "session"),
            Ok(SubjectNameId {
                value: "alice".to_string(),
                format: AssertionNameIdFormat::Unspecified,
            })
        );
    }

    #[test]
    fn every_mapper_becomes_an_attribute_in_the_order_it_was_registered() {
        let user = user(RealmId::default());
        let mappers = vec![
            mapper("email", SamlAttributeSource::Email),
            mapper("first_name", SamlAttributeSource::FirstName),
            mapper("last_name", SamlAttributeSource::LastName),
        ];

        let attributes = build_assertion_attributes(&mappers, &user, &[]);

        assert_eq!(
            attributes
                .iter()
                .map(|attribute| (attribute.name.as_str(), attribute.values.as_slice()))
                .collect::<Vec<_>>(),
            vec![
                ("email", ["alice@example.com".to_string()].as_slice()),
                ("first_name", ["Alice".to_string()].as_slice()),
                ("last_name", ["Liddell".to_string()].as_slice()),
            ]
        );
    }

    #[test]
    fn a_mapper_whose_source_is_empty_is_dropped_rather_than_emitted_valueless() {
        let mut user = user(RealmId::default());
        user.firstname = None;
        user.lastname = Some("   ".to_string());

        let mappers = vec![
            mapper("first_name", SamlAttributeSource::FirstName),
            mapper("last_name", SamlAttributeSource::LastName),
            mapper("username", SamlAttributeSource::Username),
        ];

        let attributes = build_assertion_attributes(&mappers, &user, &[]);

        assert_eq!(
            attributes
                .iter()
                .map(|attribute| attribute.name.as_str())
                .collect::<Vec<_>>(),
            vec!["username"]
        );
    }

    #[test]
    fn a_custom_mapper_carries_every_value_stored_under_its_user_attribute_key() {
        let user = user(RealmId::default());
        let attributes = vec![
            user_attribute(&user, "department", "engineering"),
            user_attribute(&user, "department", "platform"),
            user_attribute(&user, "unrelated", "ignored"),
        ];

        let mappers = vec![mapper(
            "Department",
            SamlAttributeSource::UserAttribute(
                crate::domain::client::entities::saml::UserAttributeKey::from_str("department")
                    .expect("the fixture must be a valid key"),
            ),
        )];

        let built = build_assertion_attributes(&mappers, &user, &attributes);

        assert_eq!(built.len(), 1);
        assert_eq!(
            built[0].values,
            vec!["engineering".to_string(), "platform".to_string()]
        );
    }

    #[test]
    fn user_attributes_are_only_loaded_when_a_mapper_actually_reads_them() {
        let built_in = vec![mapper("email", SamlAttributeSource::Email)];
        let custom = vec![mapper(
            "Department",
            SamlAttributeSource::UserAttribute(
                crate::domain::client::entities::saml::UserAttributeKey::from_str("department")
                    .expect("the fixture must be a valid key"),
            ),
        )];

        assert!(!needs_user_attributes(&built_in));
        assert!(needs_user_attributes(&custom));
    }

    #[test]
    fn the_mapper_name_format_survives_into_the_assertion() {
        let user = user(RealmId::default());
        let mut uri_mapper = mapper(
            "urn:oid:0.9.2342.19200300.100.1.3",
            SamlAttributeSource::Email,
        );
        uri_mapper.name_format = SamlAttributeNameFormat::Uri;

        let built = build_assertion_attributes(&[uri_mapper], &user, &[]);

        assert_eq!(built[0].name_format, AttributeNameFormat::Uri);
    }

    fn blueprint(issued_at: DateTime<Utc>) -> AssertionBlueprint {
        let user = user(RealmId::default());

        AssertionBlueprint {
            response_id: request_id("_response"),
            assertion_id: request_id("_assertion"),
            in_response_to: request_id("_inbound"),
            idp_entity_id: "https://auth.example.com/realms/master".to_string(),
            sp_entity_id: sp_entity_id("https://chat.example.com/saml/sp/1"),
            acs_url: registered_acs(),
            name_id: SubjectNameId {
                value: user.email.clone().unwrap_or_default(),
                format: AssertionNameIdFormat::EmailAddress,
            },
            session_index: "session-index".to_string(),
            attributes: Vec::new(),
            issued_at,
        }
    }

    #[test]
    fn the_descriptor_answers_the_inbound_request_and_targets_the_registered_acs_url() {
        let issued_at = Utc::now();

        let descriptor =
            build_response_descriptor(blueprint(issued_at)).expect("the blueprint is complete");

        assert_eq!(descriptor.in_response_to, request_id("_inbound"));
        assert_eq!(descriptor.destination.as_str(), registered_acs().as_str());
        assert_eq!(
            descriptor.audience.as_str(),
            "https://chat.example.com/saml/sp/1"
        );
        assert_eq!(
            descriptor.issuer.as_str(),
            "https://auth.example.com/realms/master"
        );
    }

    #[test]
    fn the_assertion_window_opens_before_it_is_issued_so_a_skewed_clock_does_not_reject_it() {
        let issued_at = Utc::now();

        let descriptor =
            build_response_descriptor(blueprint(issued_at)).expect("the blueprint is complete");

        assert_eq!(descriptor.issue_instant, issued_at);
        assert_eq!(descriptor.authn_instant, issued_at);
        assert_eq!(
            descriptor.window,
            AssertionWindow::new(
                issued_at - Duration::seconds(ASSERTION_CLOCK_SKEW_SECONDS),
                issued_at + Duration::seconds(ASSERTION_LIFETIME_SECONDS),
                issued_at + Duration::seconds(ASSERTION_LIFETIME_SECONDS),
            )
            .expect("the window is not empty")
        );
    }

    #[test]
    fn a_response_never_shares_its_xsd_id_with_the_assertion_it_carries() {
        let mut blueprint = blueprint(Utc::now());
        blueprint.assertion_id = blueprint.response_id.clone();

        let descriptor =
            build_response_descriptor(blueprint).expect("the descriptor itself is buildable");

        assert_eq!(
            descriptor.response_id, descriptor.assertion_id,
            "the rendering layer is the one that refuses this, so the descriptor keeps it verbatim"
        );
    }

    #[test]
    fn a_session_that_recorded_no_authn_request_id_can_never_answer_one() {
        let session = crate::domain::saml::services::tests::saml_auth_session(None);

        assert_eq!(
            recorded_authn_request_id(&session),
            Err(SamlSsoError::MissingAuthnRequestId)
        );
    }

    #[test]
    fn the_recorded_authn_request_id_round_trips_through_the_auth_session() {
        let id = request_id("_324d8ca747d9c07921d2abe9df447832");
        let session =
            crate::domain::saml::services::tests::saml_auth_session(record_authn_request_id(&id));

        assert_eq!(recorded_authn_request_id(&session), Ok(id));
    }

    #[test]
    fn a_recorded_value_that_is_not_an_ncname_is_refused_rather_than_echoed_into_the_xml() {
        let session = crate::domain::saml::services::tests::saml_auth_session(Some(
            "1-not-an-ncname".to_string(),
        ));

        assert_eq!(
            recorded_authn_request_id(&session),
            Err(SamlSsoError::MissingAuthnRequestId)
        );
    }
}
