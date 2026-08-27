use std::fmt::{self, Display, Formatter};
use std::str::from_utf8;

use chrono::{DateTime, Utc};
use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::name::{Namespace, ResolveResult};
use quick_xml::{NsReader, XmlVersion};
use thiserror::Error;

pub const PROTOCOL_NAMESPACE: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
pub const ASSERTION_NAMESPACE: &str = "urn:oasis:names:tc:SAML:2.0:assertion";

const HTTP_POST_BINDING: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST";
const HTTP_REDIRECT_BINDING: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect";

const UNSPECIFIED_FORMAT: &str = "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified";
const EMAIL_ADDRESS_FORMAT: &str = "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress";
const X509_SUBJECT_NAME_FORMAT: &str = "urn:oasis:names:tc:SAML:1.1:nameid-format:X509SubjectName";
const WINDOWS_DOMAIN_QUALIFIED_NAME_FORMAT: &str =
    "urn:oasis:names:tc:SAML:1.1:nameid-format:WindowsDomainQualifiedName";
const KERBEROS_FORMAT: &str = "urn:oasis:names:tc:SAML:2.0:nameid-format:kerberos";
const ENTITY_FORMAT: &str = "urn:oasis:names:tc:SAML:2.0:nameid-format:entity";
const PERSISTENT_FORMAT: &str = "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent";
const TRANSIENT_FORMAT: &str = "urn:oasis:names:tc:SAML:2.0:nameid-format:transient";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthnError {
    #[error("malformed xml: {0}")]
    MalformedXml(String),
    #[error("expected an <AuthnRequest> in the saml 2.0 protocol namespace, found {found}")]
    NotAnAuthnRequest { found: String },
    #[error("missing attribute `{0}` on <AuthnRequest>")]
    MissingAttribute(&'static str),
    #[error("missing <Issuer> in the saml 2.0 assertion namespace")]
    MissingIssuer,
    #[error("`{0}` is present but empty")]
    EmptyValue(&'static str),
    #[error("<{0}> appears more than once in <AuthnRequest>")]
    DuplicateElement(&'static str),
    #[error("unsupported saml version `{0}`, expected `2.0`")]
    UnsupportedVersion(String),
    #[error(
        "`{0}` is not a valid xml NCName: an ID must be non-empty, must not start with a digit and must not contain `:` or whitespace"
    )]
    InvalidRequestId(String),
    #[error("`{value}` is not a utc rfc 3339 timestamp for `{attribute}`")]
    InvalidTimestamp {
        attribute: &'static str,
        value: String,
    },
    #[error("`{value}` is not an absolute uri for `{attribute}`")]
    RelativeUri {
        attribute: &'static str,
        value: String,
    },
    #[error("`{value}` is not an xsd:boolean for `{attribute}`")]
    InvalidBoolean {
        attribute: &'static str,
        value: String,
    },
    #[error("unsupported protocol binding `{0}`")]
    UnsupportedProtocolBinding(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(String);

impl RequestId {
    pub fn parse(raw: &str) -> Result<Self, AuthnError> {
        if !is_ncname(raw) {
            return Err(AuthnError::InvalidRequestId(raw.to_owned()));
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RequestId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Issuer(String);

impl Issuer {
    pub fn parse(raw: &str) -> Result<Self, AuthnError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AuthnError::EmptyValue("Issuer"));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Issuer {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsoluteUri(String);

impl AbsoluteUri {
    pub fn parse(attribute: &'static str, raw: &str) -> Result<Self, AuthnError> {
        if has_uri_scheme(raw) {
            Ok(Self(raw.to_owned()))
        } else {
            Err(AuthnError::RelativeUri {
                attribute,
                value: raw.to_owned(),
            })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for AbsoluteUri {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SamlVersion {
    #[default]
    TwoPointZero,
}

impl SamlVersion {
    pub fn parse(raw: &str) -> Result<Self, AuthnError> {
        match raw {
            "2.0" => Ok(Self::TwoPointZero),
            other => Err(AuthnError::UnsupportedVersion(other.to_owned())),
        }
    }

    pub const fn as_str(self) -> &'static str {
        "2.0"
    }
}

impl Display for SamlVersion {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolBinding {
    HttpPost,
    HttpRedirect,
}

impl ProtocolBinding {
    pub fn parse(raw: &str) -> Result<Self, AuthnError> {
        match raw {
            HTTP_POST_BINDING => Ok(Self::HttpPost),
            HTTP_REDIRECT_BINDING => Ok(Self::HttpRedirect),
            other => Err(AuthnError::UnsupportedProtocolBinding(other.to_owned())),
        }
    }

    pub const fn as_uri(self) -> &'static str {
        match self {
            Self::HttpPost => HTTP_POST_BINDING,
            Self::HttpRedirect => HTTP_REDIRECT_BINDING,
        }
    }
}

impl Display for ProtocolBinding {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_uri())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NameIdFormat {
    Unspecified,
    EmailAddress,
    X509SubjectName,
    WindowsDomainQualifiedName,
    Kerberos,
    Entity,
    Persistent,
    Transient,
    Unrecognised(String),
}

impl NameIdFormat {
    pub fn parse(raw: &str) -> Result<Self, AuthnError> {
        match raw {
            "" => Err(AuthnError::EmptyValue("Format")),
            UNSPECIFIED_FORMAT => Ok(Self::Unspecified),
            EMAIL_ADDRESS_FORMAT => Ok(Self::EmailAddress),
            X509_SUBJECT_NAME_FORMAT => Ok(Self::X509SubjectName),
            WINDOWS_DOMAIN_QUALIFIED_NAME_FORMAT => Ok(Self::WindowsDomainQualifiedName),
            KERBEROS_FORMAT => Ok(Self::Kerberos),
            ENTITY_FORMAT => Ok(Self::Entity),
            PERSISTENT_FORMAT => Ok(Self::Persistent),
            TRANSIENT_FORMAT => Ok(Self::Transient),
            other => Ok(Self::Unrecognised(other.to_owned())),
        }
    }

    pub fn as_uri(&self) -> &str {
        match self {
            Self::Unspecified => UNSPECIFIED_FORMAT,
            Self::EmailAddress => EMAIL_ADDRESS_FORMAT,
            Self::X509SubjectName => X509_SUBJECT_NAME_FORMAT,
            Self::WindowsDomainQualifiedName => WINDOWS_DOMAIN_QUALIFIED_NAME_FORMAT,
            Self::Kerberos => KERBEROS_FORMAT,
            Self::Entity => ENTITY_FORMAT,
            Self::Persistent => PERSISTENT_FORMAT,
            Self::Transient => TRANSIENT_FORMAT,
            Self::Unrecognised(uri) => uri,
        }
    }
}

impl Display for NameIdFormat {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_uri())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthnRequest {
    pub id: RequestId,
    pub version: SamlVersion,
    pub issue_instant: DateTime<Utc>,
    pub issuer: Issuer,
    pub destination: Option<AbsoluteUri>,
    pub assertion_consumer_service_url: Option<AbsoluteUri>,
    pub protocol_binding: Option<ProtocolBinding>,
    pub name_id_policy_format: Option<NameIdFormat>,
    pub force_authn: bool,
    pub is_passive: bool,
}

impl TryFrom<&str> for AuthnRequest {
    type Error = AuthnError;

    fn try_from(xml: &str) -> Result<Self, Self::Error> {
        Self::parse(xml)
    }
}

impl AuthnRequest {
    pub fn parse(xml: &str) -> Result<Self, AuthnError> {
        let mut reader = NsReader::from_str(xml);
        reader.config_mut().expand_empty_elements = true;
        reader.config_mut().check_comments = true;

        let root = read_root(&mut reader)?;
        let attributes = RootAttributes::read(&root)?;
        let children = Children::read(&mut reader)?;

        Ok(Self {
            id: attributes.id,
            version: attributes.version,
            issue_instant: attributes.issue_instant,
            issuer: children.issuer.ok_or(AuthnError::MissingIssuer)?,
            destination: attributes.destination,
            assertion_consumer_service_url: attributes.assertion_consumer_service_url,
            protocol_binding: attributes.protocol_binding,
            name_id_policy_format: children.name_id_policy_format,
            force_authn: attributes.force_authn,
            is_passive: attributes.is_passive,
        })
    }
}

struct RootAttributes {
    id: RequestId,
    version: SamlVersion,
    issue_instant: DateTime<Utc>,
    destination: Option<AbsoluteUri>,
    assertion_consumer_service_url: Option<AbsoluteUri>,
    protocol_binding: Option<ProtocolBinding>,
    force_authn: bool,
    is_passive: bool,
}

impl RootAttributes {
    fn read(root: &BytesStart<'_>) -> Result<Self, AuthnError> {
        let mut id = None;
        let mut version = None;
        let mut issue_instant = None;
        let mut destination = None;
        let mut assertion_consumer_service_url = None;
        let mut protocol_binding = None;
        let mut force_authn = false;
        let mut is_passive = false;

        let attributes = root.attributes();
        let decoder = attributes.decoder();

        for attribute in attributes {
            let attribute = attribute.map_err(malformed)?;
            if attribute.key.prefix().is_some() {
                continue;
            }
            let value = attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, decoder)
                .map_err(malformed)?;

            match attribute.key.local_name().as_ref() {
                b"ID" => id = Some(RequestId::parse(&value)?),
                b"Version" => version = Some(SamlVersion::parse(&value)?),
                b"IssueInstant" => issue_instant = Some(utc_timestamp("IssueInstant", &value)?),
                b"Destination" => destination = Some(AbsoluteUri::parse("Destination", &value)?),
                b"AssertionConsumerServiceURL" => {
                    assertion_consumer_service_url =
                        Some(AbsoluteUri::parse("AssertionConsumerServiceURL", &value)?);
                }
                b"ProtocolBinding" => protocol_binding = Some(ProtocolBinding::parse(&value)?),
                b"ForceAuthn" => force_authn = xsd_boolean("ForceAuthn", &value)?,
                b"IsPassive" => is_passive = xsd_boolean("IsPassive", &value)?,
                _ => {}
            }
        }

        Ok(Self {
            id: id.ok_or(AuthnError::MissingAttribute("ID"))?,
            version: version.ok_or(AuthnError::MissingAttribute("Version"))?,
            issue_instant: issue_instant.ok_or(AuthnError::MissingAttribute("IssueInstant"))?,
            destination,
            assertion_consumer_service_url,
            protocol_binding,
            force_authn,
            is_passive,
        })
    }
}

struct Children {
    issuer: Option<Issuer>,
    name_id_policy_format: Option<NameIdFormat>,
}

impl Children {
    fn read(reader: &mut NsReader<&[u8]>) -> Result<Self, AuthnError> {
        let mut issuer = None;
        let mut name_id_policy_format = None;
        let mut seen_name_id_policy = false;
        let mut depth = 0usize;

        loop {
            let (namespace, event) = read_event(reader)?;
            match event {
                Event::Start(child) if depth == 0 => {
                    let local = local_name(&child)?;
                    match (namespace.as_deref(), local.as_str()) {
                        (Some(ASSERTION_NAMESPACE), "Issuer") => {
                            if issuer.is_some() {
                                return Err(AuthnError::DuplicateElement("Issuer"));
                            }
                            issuer = Some(Issuer::parse(&read_text(reader, "Issuer")?)?);
                        }
                        (Some(PROTOCOL_NAMESPACE), "NameIDPolicy") => {
                            if seen_name_id_policy {
                                return Err(AuthnError::DuplicateElement("NameIDPolicy"));
                            }
                            seen_name_id_policy = true;
                            name_id_policy_format = read_name_id_policy_format(&child)?;
                            depth += 1;
                        }
                        _ => depth += 1,
                    }
                }
                Event::Start(_) => depth += 1,
                Event::End(_) => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                Event::Eof => {
                    return Err(AuthnError::MalformedXml(
                        "<AuthnRequest> is not closed".to_owned(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Self {
            issuer,
            name_id_policy_format,
        })
    }
}

fn read_root<'i>(reader: &mut NsReader<&'i [u8]>) -> Result<BytesStart<'i>, AuthnError> {
    loop {
        let (namespace, event) = read_event(reader)?;
        match event {
            Event::Start(tag) => {
                let local = local_name(&tag)?;
                if namespace.as_deref() == Some(PROTOCOL_NAMESPACE) && local == "AuthnRequest" {
                    return Ok(tag);
                }
                return Err(AuthnError::NotAnAuthnRequest {
                    found: qualified_name(namespace.as_deref(), &local),
                });
            }
            Event::Eof => {
                return Err(AuthnError::MalformedXml(
                    "document holds no element".to_owned(),
                ));
            }
            _ => {}
        }
    }
}

fn read_name_id_policy_format(tag: &BytesStart<'_>) -> Result<Option<NameIdFormat>, AuthnError> {
    let attributes = tag.attributes();
    let decoder = attributes.decoder();

    for attribute in attributes {
        let attribute = attribute.map_err(malformed)?;
        if attribute.key.prefix().is_some() || attribute.key.local_name().as_ref() != b"Format" {
            continue;
        }
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, decoder)
            .map_err(malformed)?;
        return NameIdFormat::parse(&value).map(Some);
    }

    Ok(None)
}

fn read_text(reader: &mut NsReader<&[u8]>, element: &'static str) -> Result<String, AuthnError> {
    let mut text = String::new();

    loop {
        let (_, event) = read_event(reader)?;
        match event {
            Event::Text(chunk) => text.push_str(&chunk.xml10_content().map_err(malformed)?),
            Event::CData(chunk) => text.push_str(&chunk.xml10_content().map_err(malformed)?),
            Event::GeneralRef(reference) => text.push(resolve_reference(&reference)?),
            Event::End(_) => break,
            Event::Start(_) => {
                return Err(AuthnError::MalformedXml(format!(
                    "<{element}> holds markup where character data was expected"
                )));
            }
            Event::Eof => {
                return Err(AuthnError::MalformedXml(format!(
                    "<{element}> is not closed"
                )));
            }
            _ => {}
        }
    }

    Ok(text)
}

fn read_event<'i>(
    reader: &mut NsReader<&'i [u8]>,
) -> Result<(Option<String>, Event<'i>), AuthnError> {
    let (resolved, event) = reader.read_resolved_event().map_err(malformed)?;
    let namespace = match resolved {
        ResolveResult::Bound(Namespace(uri)) => Some(utf8(uri)?),
        ResolveResult::Unbound => None,
        ResolveResult::Unknown(prefix) => {
            return Err(AuthnError::MalformedXml(format!(
                "undeclared namespace prefix `{}`",
                utf8(&prefix)?
            )));
        }
    };
    Ok((namespace, event))
}

fn resolve_reference(reference: &BytesRef<'_>) -> Result<char, AuthnError> {
    if let Some(character) = reference.resolve_char_ref().map_err(malformed)? {
        return Ok(character);
    }

    let name = reference.decode().map_err(malformed)?;
    match name.as_ref() {
        "amp" => Ok('&'),
        "lt" => Ok('<'),
        "gt" => Ok('>'),
        "quot" => Ok('"'),
        "apos" => Ok('\''),
        other => Err(AuthnError::MalformedXml(format!(
            "unknown entity reference `&{other};`"
        ))),
    }
}

fn utc_timestamp(attribute: &'static str, raw: &str) -> Result<DateTime<Utc>, AuthnError> {
    let invalid = || AuthnError::InvalidTimestamp {
        attribute,
        value: raw.to_owned(),
    };

    if !raw.ends_with('Z') {
        return Err(invalid());
    }

    DateTime::parse_from_rfc3339(raw)
        .map(|instant| instant.with_timezone(&Utc))
        .map_err(|_| invalid())
}

fn xsd_boolean(attribute: &'static str, raw: &str) -> Result<bool, AuthnError> {
    match raw {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        other => Err(AuthnError::InvalidBoolean {
            attribute,
            value: other.to_owned(),
        }),
    }
}

fn is_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) if is_ncname_start(first) => characters.all(is_ncname_char),
        _ => false,
    }
}

fn is_ncname_start(character: char) -> bool {
    matches!(character,
        'A'..='Z'
        | '_'
        | 'a'..='z'
        | '\u{c0}'..='\u{d6}'
        | '\u{d8}'..='\u{f6}'
        | '\u{f8}'..='\u{2ff}'
        | '\u{370}'..='\u{37d}'
        | '\u{37f}'..='\u{1fff}'
        | '\u{200c}'..='\u{200d}'
        | '\u{2070}'..='\u{218f}'
        | '\u{2c00}'..='\u{2fef}'
        | '\u{3001}'..='\u{d7ff}'
        | '\u{f900}'..='\u{fdcf}'
        | '\u{fdf0}'..='\u{fffd}'
        | '\u{10000}'..='\u{effff}')
}

fn is_ncname_char(character: char) -> bool {
    is_ncname_start(character)
        || matches!(character,
            '-' | '.'
            | '0'..='9'
            | '\u{b7}'
            | '\u{300}'..='\u{36f}'
            | '\u{203f}'..='\u{2040}')
}

fn has_uri_scheme(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    let mut characters = scheme.chars();
    match characters.next() {
        Some(first) if first.is_ascii_alphabetic() => characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        }),
        _ => false,
    }
}

fn qualified_name(namespace: Option<&str>, local: &str) -> String {
    match namespace {
        Some(uri) => format!("{{{uri}}}{local}"),
        None => local.to_owned(),
    }
}

fn local_name(tag: &BytesStart<'_>) -> Result<String, AuthnError> {
    utf8(tag.local_name().as_ref())
}

fn utf8(bytes: &[u8]) -> Result<String, AuthnError> {
    from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|error| AuthnError::MalformedXml(error.to_string()))
}

fn malformed<E: Display>(error: E) -> AuthnError {
    AuthnError::MalformedXml(error.to_string())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::{
        AbsoluteUri, AuthnError, AuthnRequest, Issuer, NameIdFormat, ProtocolBinding, RequestId,
        SamlVersion,
    };

    const CHATWOOT_REQUEST: &str = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_324d8ca747d9c07921d2abe9df447832" Version="2.0" IssueInstant="2026-08-25T18:51:18.334Z" Destination="https://auth.example.com/realms/master/protocol/saml" AssertionConsumerServiceURL="https://chat.example.com/omniauth/saml/callback?account_id=1" ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
  <saml:Issuer>https://chat.example.com/saml/sp/1</saml:Issuer>
  <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" AllowCreate="true"/>
</samlp:AuthnRequest>"#;

    fn minimal_request(id: &str) -> String {
        format!(
            r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="{id}" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#
        )
    }

    fn instant(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .expect("test timestamp is rfc 3339")
            .with_timezone(&Utc)
    }

    #[test]
    fn a_chatwoot_authn_request_is_parsed_field_by_field() {
        let request = AuthnRequest::parse(CHATWOOT_REQUEST).expect("parse chatwoot request");

        assert_eq!(request.id.as_str(), "_324d8ca747d9c07921d2abe9df447832");
        assert_eq!(request.version, SamlVersion::TwoPointZero);
        assert_eq!(request.issue_instant, instant("2026-08-25T18:51:18.334Z"));
        assert_eq!(
            request.issuer.as_str(),
            "https://chat.example.com/saml/sp/1"
        );
        assert_eq!(
            request.destination.expect("destination").as_str(),
            "https://auth.example.com/realms/master/protocol/saml"
        );
        assert_eq!(
            request
                .assertion_consumer_service_url
                .expect("assertion consumer service url")
                .as_str(),
            "https://chat.example.com/omniauth/saml/callback?account_id=1"
        );
        assert_eq!(request.protocol_binding, Some(ProtocolBinding::HttpPost));
        assert_eq!(
            request.name_id_policy_format,
            Some(NameIdFormat::EmailAddress)
        );
        assert!(!request.force_authn);
        assert!(!request.is_passive);
    }

    #[test]
    fn an_id_that_starts_with_a_digit_is_refused() {
        assert_eq!(
            AuthnRequest::parse(&minimal_request("4d8ca747d9c07921d2abe9df447832ab")),
            Err(AuthnError::InvalidRequestId(
                "4d8ca747d9c07921d2abe9df447832ab".to_owned()
            ))
        );
    }

    #[test]
    fn an_id_containing_a_colon_is_refused() {
        assert_eq!(
            AuthnRequest::parse(&minimal_request("ns:request")),
            Err(AuthnError::InvalidRequestId("ns:request".to_owned()))
        );
    }

    #[test]
    fn an_id_containing_whitespace_is_refused() {
        assert_eq!(
            AuthnRequest::parse(&minimal_request("two words")),
            Err(AuthnError::InvalidRequestId("two words".to_owned()))
        );
    }

    #[test]
    fn an_id_starting_with_a_hyphen_is_refused() {
        assert_eq!(
            AuthnRequest::parse(&minimal_request("-request")),
            Err(AuthnError::InvalidRequestId("-request".to_owned()))
        );
    }

    #[test]
    fn an_underscore_prefixed_uuid_is_accepted() {
        let request =
            AuthnRequest::parse(&minimal_request("_4d8ca747-d9c0-7921-d2ab-e9df447832ab"))
                .expect("parse underscore prefixed id");

        assert_eq!(request.id.as_str(), "_4d8ca747-d9c0-7921-d2ab-e9df447832ab");
    }

    #[test]
    fn a_raw_uuid_is_refused_exactly_when_it_opens_on_a_digit() {
        for first in "0123456789abcdef".chars() {
            let id = format!("{first}d8ca747-d9c0-7921-d2ab-e9df447832ab");
            let parsed = AuthnRequest::parse(&minimal_request(&id));

            assert_eq!(parsed.is_err(), first.is_ascii_digit(), "id `{id}`");
        }
    }

    #[test]
    fn an_id_starting_with_a_letter_is_accepted() {
        let request =
            AuthnRequest::parse(&minimal_request("id-42.7")).expect("parse letter prefixed id");

        assert_eq!(request.id.as_str(), "id-42.7");
    }

    #[test]
    fn an_empty_id_is_refused() {
        assert_eq!(
            AuthnRequest::parse(&minimal_request("")),
            Err(AuthnError::InvalidRequestId(String::new()))
        );
    }

    #[test]
    fn a_missing_id_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::MissingAttribute("ID"))
        );
    }

    #[test]
    fn a_missing_version_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::MissingAttribute("Version"))
        );
    }

    #[test]
    fn an_empty_version_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::UnsupportedVersion(String::new()))
        );
    }

    #[test]
    fn a_version_other_than_two_zero_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="1.1" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::UnsupportedVersion("1.1".to_owned()))
        );
    }

    #[test]
    fn a_missing_issue_instant_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::MissingAttribute("IssueInstant"))
        );
    }

    #[test]
    fn an_issue_instant_that_is_not_a_timestamp_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="yesterday"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::InvalidTimestamp {
                attribute: "IssueInstant",
                value: "yesterday".to_owned()
            })
        );
    }

    #[test]
    fn an_issue_instant_carrying_a_timezone_offset_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T20:51:18+02:00"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::InvalidTimestamp {
                attribute: "IssueInstant",
                value: "2026-08-25T20:51:18+02:00".to_owned()
            })
        );
    }

    #[test]
    fn a_missing_issuer_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"/>"#;

        assert_eq!(AuthnRequest::parse(xml), Err(AuthnError::MissingIssuer));
    }

    #[test]
    fn an_empty_issuer_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>   </saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::EmptyValue("Issuer"))
        );
    }

    #[test]
    fn an_issuer_in_the_protocol_namespace_is_not_the_saml_issuer() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><samlp:Issuer>https://sp.example.com</samlp:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(AuthnRequest::parse(xml), Err(AuthnError::MissingIssuer));
    }

    #[test]
    fn a_duplicate_issuer_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer><saml:Issuer>https://other.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::DuplicateElement("Issuer"))
        );
    }

    #[test]
    fn an_issuer_wrapped_in_whitespace_is_trimmed() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>
        https://sp.example.com
      </saml:Issuer></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse padded issuer");

        assert_eq!(request.issuer.as_str(), "https://sp.example.com");
    }

    #[test]
    fn an_entity_reference_inside_the_issuer_is_expanded() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com/?a=1&amp;b=2</saml:Issuer></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse escaped issuer");

        assert_eq!(request.issuer.as_str(), "https://sp.example.com/?a=1&b=2");
    }

    #[test]
    fn a_default_namespace_bound_to_the_protocol_namespace_is_accepted() {
        let xml = r#"<AuthnRequest xmlns="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer></AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse default namespaced request");

        assert_eq!(request.id.as_str(), "_abc");
    }

    #[test]
    fn an_arbitrary_prefix_bound_to_the_protocol_namespace_is_accepted() {
        let xml = r#"<p:AuthnRequest xmlns:p="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:a="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><a:Issuer>https://sp.example.com</a:Issuer></p:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse arbitrary prefix");

        assert_eq!(request.issuer.as_str(), "https://sp.example.com");
    }

    #[test]
    fn the_samlp_prefix_bound_to_another_namespace_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:example:not-saml" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"/>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::NotAnAuthnRequest {
                found: "{urn:example:not-saml}AuthnRequest".to_owned()
            })
        );
    }

    #[test]
    fn an_authn_request_without_any_namespace_is_refused() {
        let xml = r#"<AuthnRequest ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"/>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::NotAnAuthnRequest {
                found: "AuthnRequest".to_owned()
            })
        );
    }

    #[test]
    fn a_saml_response_is_refused() {
        let xml = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"/>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::NotAnAuthnRequest {
                found: "{urn:oasis:names:tc:SAML:2.0:protocol}Response".to_owned()
            })
        );
    }

    #[test]
    fn an_xml_declaration_before_the_root_element_is_skipped() {
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>{}"#,
            minimal_request("_abc")
        );

        let request = AuthnRequest::parse(&xml).expect("parse declared document");

        assert_eq!(request.id.as_str(), "_abc");
    }

    #[test]
    fn force_authn_and_is_passive_default_to_false() {
        let request = AuthnRequest::parse(&minimal_request("_abc")).expect("parse minimal request");

        assert!(!request.force_authn);
        assert!(!request.is_passive);
    }

    #[test]
    fn force_authn_and_is_passive_are_read_from_the_xsd_boolean_words() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z" ForceAuthn="true" IsPassive="false"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse boolean words");

        assert!(request.force_authn);
        assert!(!request.is_passive);
    }

    #[test]
    fn force_authn_and_is_passive_are_read_from_the_xsd_boolean_digits() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z" ForceAuthn="0" IsPassive="1"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse boolean digits");

        assert!(!request.force_authn);
        assert!(request.is_passive);
    }

    #[test]
    fn a_value_that_is_not_an_xsd_boolean_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z" ForceAuthn="yes"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::InvalidBoolean {
                attribute: "ForceAuthn",
                value: "yes".to_owned()
            })
        );
    }

    #[test]
    fn a_relative_destination_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z" Destination="/realms/master/protocol/saml"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::RelativeUri {
                attribute: "Destination",
                value: "/realms/master/protocol/saml".to_owned()
            })
        );
    }

    #[test]
    fn an_escaped_query_string_on_the_assertion_consumer_service_url_survives() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z" AssertionConsumerServiceURL="https://chat.example.com/omniauth/saml/callback?account_id=1&amp;tenant=acme"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse escaped acs url");

        assert_eq!(
            request
                .assertion_consumer_service_url
                .expect("assertion consumer service url")
                .as_str(),
            "https://chat.example.com/omniauth/saml/callback?account_id=1&tenant=acme"
        );
    }

    #[test]
    fn an_unsupported_protocol_binding_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z" ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Artifact"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::UnsupportedProtocolBinding(
                "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Artifact".to_owned()
            ))
        );
    }

    #[test]
    fn the_redirect_binding_is_accepted() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z" ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect"><saml:Issuer>https://sp.example.com</saml:Issuer></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse redirect binding");

        assert_eq!(
            request.protocol_binding,
            Some(ProtocolBinding::HttpRedirect)
        );
    }

    #[test]
    fn a_missing_name_id_policy_leaves_the_format_absent() {
        let request = AuthnRequest::parse(&minimal_request("_abc")).expect("parse minimal request");

        assert_eq!(request.name_id_policy_format, None);
    }

    #[test]
    fn a_name_id_policy_without_a_format_leaves_the_format_absent() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer><samlp:NameIDPolicy AllowCreate="true"/></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse policy without format");

        assert_eq!(request.name_id_policy_format, None);
    }

    #[test]
    fn an_unrecognised_name_id_format_is_preserved_verbatim() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer><samlp:NameIDPolicy Format="urn:example:custom-format"/></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse custom format");

        assert_eq!(
            request.name_id_policy_format,
            Some(NameIdFormat::Unrecognised(
                "urn:example:custom-format".to_owned()
            ))
        );
    }

    #[test]
    fn an_empty_name_id_format_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer><samlp:NameIDPolicy Format=""/></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::EmptyValue("Format"))
        );
    }

    #[test]
    fn a_duplicate_name_id_policy_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer><samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:2.0:nameid-format:transient"/><samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:2.0:nameid-format:persistent"/></samlp:AuthnRequest>"#;

        assert_eq!(
            AuthnRequest::parse(xml),
            Err(AuthnError::DuplicateElement("NameIDPolicy"))
        );
    }

    #[test]
    fn unhandled_child_elements_and_their_descendants_are_skipped() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><samlp:Extensions><vendor:Flag xmlns:vendor="urn:example:vendor"><vendor:Nested>x</vendor:Nested></vendor:Flag></samlp:Extensions><saml:Issuer>https://sp.example.com</saml:Issuer><samlp:RequestedAuthnContext Comparison="exact"><saml:AuthnContextClassRef>urn:oasis:names:tc:SAML:2.0:ac:classes:Password</saml:AuthnContextClassRef></samlp:RequestedAuthnContext><samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:2.0:nameid-format:persistent"/></samlp:AuthnRequest>"#;

        let request = AuthnRequest::parse(xml).expect("parse request with extra children");

        assert_eq!(request.issuer.as_str(), "https://sp.example.com");
        assert_eq!(
            request.name_id_policy_format,
            Some(NameIdFormat::Persistent)
        );
    }

    #[test]
    fn a_nested_issuer_deeper_in_the_tree_is_not_mistaken_for_the_request_issuer() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><samlp:Scoping><samlp:IDPList><samlp:IDPEntry ProviderID="https://idp.example.com"/></samlp:IDPList><saml:Issuer>https://nested.example.com</saml:Issuer></samlp:Scoping></samlp:AuthnRequest>"#;

        assert_eq!(AuthnRequest::parse(xml), Err(AuthnError::MissingIssuer));
    }

    #[test]
    fn malformed_xml_is_refused() {
        assert!(matches!(
            AuthnRequest::parse("<samlp:AuthnRequest"),
            Err(AuthnError::MalformedXml(_))
        ));
    }

    #[test]
    fn an_unclosed_root_element_is_refused() {
        let xml = r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"><saml:Issuer>https://sp.example.com</saml:Issuer>"#;

        assert!(matches!(
            AuthnRequest::parse(xml),
            Err(AuthnError::MalformedXml(_))
        ));
    }

    #[test]
    fn an_undeclared_namespace_prefix_is_refused() {
        let xml =
            r#"<samlp:AuthnRequest ID="_abc" Version="2.0" IssueInstant="2026-08-25T18:51:18Z"/>"#;

        assert!(matches!(
            AuthnRequest::parse(xml),
            Err(AuthnError::MalformedXml(_))
        ));
    }

    #[test]
    fn an_empty_document_is_refused() {
        assert!(matches!(
            AuthnRequest::parse(""),
            Err(AuthnError::MalformedXml(_))
        ));
    }

    #[test]
    fn try_from_is_the_same_parse() {
        let request: AuthnRequest = CHATWOOT_REQUEST.try_into().expect("parse via try_into");

        assert_eq!(request.id.as_str(), "_324d8ca747d9c07921d2abe9df447832");
    }

    #[test]
    fn newtypes_expose_their_source_text() {
        assert_eq!(
            RequestId::parse("_abc").expect("valid ncname").to_string(),
            "_abc"
        );
        assert_eq!(
            Issuer::parse(" https://sp.example.com ")
                .expect("valid issuer")
                .to_string(),
            "https://sp.example.com"
        );
        assert_eq!(
            AbsoluteUri::parse("Destination", "https://idp.example.com/saml")
                .expect("valid uri")
                .to_string(),
            "https://idp.example.com/saml"
        );
        assert_eq!(SamlVersion::TwoPointZero.to_string(), "2.0");
        assert_eq!(
            ProtocolBinding::HttpPost.to_string(),
            "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
        );
        assert_eq!(
            NameIdFormat::EmailAddress.to_string(),
            "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"
        );
    }
}
