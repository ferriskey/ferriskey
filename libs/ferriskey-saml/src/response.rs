use chrono::{DateTime, SecondsFormat, Utc};
use thiserror::Error;

use crate::authn::{
    ASSERTION_NAMESPACE, AbsoluteUri, Issuer, NameIdFormat, PROTOCOL_NAMESPACE, RequestId,
    SamlVersion,
};
use crate::dsig::{SignatureError, sign_enveloped};

const SUCCESS_STATUS: &str = "urn:oasis:names:tc:SAML:2.0:status:Success";
const BEARER_CONFIRMATION: &str = "urn:oasis:names:tc:SAML:2.0:cm:bearer";

const UNSPECIFIED_AUTHN_CONTEXT: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:unspecified";
const PASSWORD_AUTHN_CONTEXT: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:Password";
const PASSWORD_PROTECTED_TRANSPORT_AUTHN_CONTEXT: &str =
    "urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport";
const X509_AUTHN_CONTEXT: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:X509";
const KERBEROS_AUTHN_CONTEXT: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:Kerberos";
const TIME_SYNC_TOKEN_AUTHN_CONTEXT: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:TimeSyncToken";

const UNSPECIFIED_ATTRIBUTE_NAME_FORMAT: &str =
    "urn:oasis:names:tc:SAML:2.0:attrname-format:unspecified";
const URI_ATTRIBUTE_NAME_FORMAT: &str = "urn:oasis:names:tc:SAML:2.0:attrname-format:uri";
const BASIC_ATTRIBUTE_NAME_FORMAT: &str = "urn:oasis:names:tc:SAML:2.0:attrname-format:basic";

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("`{0}` is required but empty")]
    EmptyValue(&'static str),
    #[error("the response and its assertion cannot share the xsd:ID `{0}`")]
    DuplicateElementId(String),
    #[error("the assertion window is empty: NotBefore `{not_before}` is not before `{expiry}`")]
    EmptyWindow { not_before: String, expiry: String },
    #[error(
        "the subject confirmation expires at `{subject}`, after the conditions expire at `{conditions}`"
    )]
    SubjectConfirmationOutlivesConditions { subject: String, conditions: String },
    #[error(transparent)]
    Signature(#[from] SignatureError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthnContextClassRef {
    Unspecified,
    Password,
    PasswordProtectedTransport,
    X509,
    Kerberos,
    TimeSyncToken,
    Unrecognised(String),
}

impl AuthnContextClassRef {
    pub fn as_uri(&self) -> &str {
        match self {
            Self::Unspecified => UNSPECIFIED_AUTHN_CONTEXT,
            Self::Password => PASSWORD_AUTHN_CONTEXT,
            Self::PasswordProtectedTransport => PASSWORD_PROTECTED_TRANSPORT_AUTHN_CONTEXT,
            Self::X509 => X509_AUTHN_CONTEXT,
            Self::Kerberos => KERBEROS_AUTHN_CONTEXT,
            Self::TimeSyncToken => TIME_SYNC_TOKEN_AUTHN_CONTEXT,
            Self::Unrecognised(uri) => uri,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeNameFormat {
    Unspecified,
    Uri,
    Basic,
}

impl AttributeNameFormat {
    pub const fn as_uri(self) -> &'static str {
        match self {
            Self::Unspecified => UNSPECIFIED_ATTRIBUTE_NAME_FORMAT,
            Self::Uri => URI_ATTRIBUTE_NAME_FORMAT,
            Self::Basic => BASIC_ATTRIBUTE_NAME_FORMAT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionAttribute {
    pub name: String,
    pub name_format: AttributeNameFormat,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssertionWindow {
    not_before: DateTime<Utc>,
    subject_expires_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl AssertionWindow {
    pub fn new(
        not_before: DateTime<Utc>,
        subject_expires_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, ResponseError> {
        if not_before >= expires_at {
            return Err(ResponseError::EmptyWindow {
                not_before: format_instant(not_before),
                expiry: format_instant(expires_at),
            });
        }
        if not_before >= subject_expires_at {
            return Err(ResponseError::EmptyWindow {
                not_before: format_instant(not_before),
                expiry: format_instant(subject_expires_at),
            });
        }
        if subject_expires_at > expires_at {
            return Err(ResponseError::SubjectConfirmationOutlivesConditions {
                subject: format_instant(subject_expires_at),
                conditions: format_instant(expires_at),
            });
        }
        Ok(Self {
            not_before,
            subject_expires_at,
            expires_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseDescriptor {
    pub response_id: RequestId,
    pub assertion_id: RequestId,
    pub in_response_to: RequestId,
    pub issuer: Issuer,
    pub destination: AbsoluteUri,
    pub audience: AbsoluteUri,
    pub issue_instant: DateTime<Utc>,
    pub authn_instant: DateTime<Utc>,
    pub window: AssertionWindow,
    pub name_id: String,
    pub name_id_format: NameIdFormat,
    pub session_index: String,
    pub authn_context: AuthnContextClassRef,
    pub attributes: Vec<AssertionAttribute>,
}

pub fn render_response(descriptor: &ResponseDescriptor) -> Result<String, ResponseError> {
    if descriptor.response_id == descriptor.assertion_id {
        return Err(ResponseError::DuplicateElementId(
            descriptor.response_id.as_str().to_owned(),
        ));
    }

    let name_id = non_empty(&descriptor.name_id, "NameID")?;
    let session_index = non_empty(&descriptor.session_index, "SessionIndex")?;
    let authn_context = non_empty(descriptor.authn_context.as_uri(), "AuthnContextClassRef")?;

    let issue_instant = format_instant(descriptor.issue_instant);
    let authn_instant = format_instant(descriptor.authn_instant);
    let not_before = format_instant(descriptor.window.not_before);
    let subject_expires_at = format_instant(descriptor.window.subject_expires_at);
    let expires_at = format_instant(descriptor.window.expires_at);
    let version = SamlVersion::TwoPointZero.as_str();

    let mut xml = String::new();

    open_element(
        &mut xml,
        "samlp:Response",
        &[
            ("xmlns:samlp", PROTOCOL_NAMESPACE),
            ("Destination", descriptor.destination.as_str()),
            ("ID", descriptor.response_id.as_str()),
            ("InResponseTo", descriptor.in_response_to.as_str()),
            ("IssueInstant", &issue_instant),
            ("Version", version),
        ],
    );

    open_element(
        &mut xml,
        "saml:Issuer",
        &[("xmlns:saml", ASSERTION_NAMESPACE)],
    );
    push_escaped_text(&mut xml, descriptor.issuer.as_str());
    close_element(&mut xml, "saml:Issuer");

    open_element(&mut xml, "samlp:Status", &[]);
    open_element(&mut xml, "samlp:StatusCode", &[("Value", SUCCESS_STATUS)]);
    close_element(&mut xml, "samlp:StatusCode");
    close_element(&mut xml, "samlp:Status");

    open_element(
        &mut xml,
        "saml:Assertion",
        &[
            ("xmlns:saml", ASSERTION_NAMESPACE),
            ("ID", descriptor.assertion_id.as_str()),
            ("IssueInstant", &issue_instant),
            ("Version", version),
        ],
    );

    text_element(&mut xml, "saml:Issuer", descriptor.issuer.as_str());

    open_element(&mut xml, "saml:Subject", &[]);
    open_element(
        &mut xml,
        "saml:NameID",
        &[("Format", descriptor.name_id_format.as_uri())],
    );
    push_escaped_text(&mut xml, name_id);
    close_element(&mut xml, "saml:NameID");
    open_element(
        &mut xml,
        "saml:SubjectConfirmation",
        &[("Method", BEARER_CONFIRMATION)],
    );
    open_element(
        &mut xml,
        "saml:SubjectConfirmationData",
        &[
            ("InResponseTo", descriptor.in_response_to.as_str()),
            ("NotOnOrAfter", &subject_expires_at),
            ("Recipient", descriptor.destination.as_str()),
        ],
    );
    close_element(&mut xml, "saml:SubjectConfirmationData");
    close_element(&mut xml, "saml:SubjectConfirmation");
    close_element(&mut xml, "saml:Subject");

    open_element(
        &mut xml,
        "saml:Conditions",
        &[("NotBefore", &not_before), ("NotOnOrAfter", &expires_at)],
    );
    open_element(&mut xml, "saml:AudienceRestriction", &[]);
    text_element(&mut xml, "saml:Audience", descriptor.audience.as_str());
    close_element(&mut xml, "saml:AudienceRestriction");
    close_element(&mut xml, "saml:Conditions");

    open_element(
        &mut xml,
        "saml:AuthnStatement",
        &[
            ("AuthnInstant", &authn_instant),
            ("SessionIndex", session_index),
        ],
    );
    open_element(&mut xml, "saml:AuthnContext", &[]);
    text_element(&mut xml, "saml:AuthnContextClassRef", authn_context);
    close_element(&mut xml, "saml:AuthnContext");
    close_element(&mut xml, "saml:AuthnStatement");

    if !descriptor.attributes.is_empty() {
        open_element(&mut xml, "saml:AttributeStatement", &[]);
        for attribute in &descriptor.attributes {
            let name = non_empty(&attribute.name, "Attribute/@Name")?;
            open_element(
                &mut xml,
                "saml:Attribute",
                &[
                    ("Name", name),
                    ("NameFormat", attribute.name_format.as_uri()),
                ],
            );
            for value in &attribute.values {
                text_element(&mut xml, "saml:AttributeValue", value);
            }
            close_element(&mut xml, "saml:Attribute");
        }
        close_element(&mut xml, "saml:AttributeStatement");
    }

    close_element(&mut xml, "saml:Assertion");
    close_element(&mut xml, "samlp:Response");

    Ok(xml)
}

pub fn render_signed_response(
    descriptor: &ResponseDescriptor,
    private_key_pem: &str,
    certificate_base64_der: &str,
) -> Result<String, ResponseError> {
    let document = render_response(descriptor)?;
    let signed = sign_enveloped(
        &document,
        descriptor.assertion_id.as_str(),
        private_key_pem,
        certificate_base64_der,
    )?;
    Ok(signed)
}

fn format_instant(instant: DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn non_empty<'a>(value: &'a str, label: &'static str) -> Result<&'a str, ResponseError> {
    if value.is_empty() {
        return Err(ResponseError::EmptyValue(label));
    }
    Ok(value)
}

fn open_element(xml: &mut String, name: &str, attributes: &[(&str, &str)]) {
    xml.push('<');
    xml.push_str(name);
    for (key, value) in attributes {
        xml.push(' ');
        xml.push_str(key);
        xml.push_str("=\"");
        push_escaped_attribute_value(xml, value);
        xml.push('"');
    }
    xml.push('>');
}

fn close_element(xml: &mut String, name: &str) {
    xml.push_str("</");
    xml.push_str(name);
    xml.push('>');
}

fn text_element(xml: &mut String, name: &str, text: &str) {
    open_element(xml, name, &[]);
    push_escaped_text(xml, text);
    close_element(xml, name);
}

fn push_escaped_attribute_value(xml: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '&' => xml.push_str("&amp;"),
            '<' => xml.push_str("&lt;"),
            '"' => xml.push_str("&quot;"),
            '\t' => xml.push_str("&#x9;"),
            '\n' => xml.push_str("&#xA;"),
            '\r' => xml.push_str("&#xD;"),
            other => xml.push(other),
        }
    }
}

fn push_escaped_text(xml: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '&' => xml.push_str("&amp;"),
            '<' => xml.push_str("&lt;"),
            '>' => xml.push_str("&gt;"),
            '\r' => xml.push_str("&#xD;"),
            other => xml.push(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AssertionAttribute, AssertionWindow, AttributeNameFormat, AuthnContextClassRef,
        ResponseDescriptor, ResponseError, render_response, render_signed_response,
    };
    use crate::authn::{AbsoluteUri, Issuer, NameIdFormat, RequestId};
    use crate::c14n::canonicalize_exclusive;
    use chrono::{DateTime, Utc};

    fn instant(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .expect("the fixture instant should parse")
            .with_timezone(&Utc)
    }

    fn reference_descriptor() -> ResponseDescriptor {
        ResponseDescriptor {
            response_id: RequestId::parse("_0d749ed5ea0e6f4800630da48d1f8006")
                .expect("the response id should be an NCName"),
            assertion_id: RequestId::parse("_018d3ab762d8fd3f7feb90b5164e821f")
                .expect("the assertion id should be an NCName"),
            in_response_to: RequestId::parse("_324d8ca747d9c07921d2abe9df447832")
                .expect("the request id should be an NCName"),
            issuer: Issuer::parse("https://auth.example.com/realms/master")
                .expect("the issuer should be non empty"),
            destination: AbsoluteUri::parse(
                "Destination",
                "https://chat.example.com/omniauth/saml/callback?account_id=1",
            )
            .expect("the destination should be absolute"),
            audience: AbsoluteUri::parse("Audience", "https://chat.example.com/saml/sp/1")
                .expect("the audience should be absolute"),
            issue_instant: instant("2026-08-25T18:51:18.335Z"),
            authn_instant: instant("2026-08-25T18:51:18.335Z"),
            window: AssertionWindow::new(
                instant("2026-08-25T18:50:18.335Z"),
                instant("2026-08-25T18:56:18.335Z"),
                instant("2026-08-25T19:01:18.335Z"),
            )
            .expect("the reference window should be valid"),
            name_id: "alice@example.com".to_owned(),
            name_id_format: NameIdFormat::EmailAddress,
            session_index: "b1f7c2e4-3d5a-4c8b-9e1f-7a2d6c0b3e59".to_owned(),
            authn_context: AuthnContextClassRef::PasswordProtectedTransport,
            attributes: vec![AssertionAttribute {
                name: "email".to_owned(),
                name_format: AttributeNameFormat::Basic,
                values: vec!["alice@example.com".to_owned()],
            }],
        }
    }

    #[test]
    fn renders_the_reference_response() {
        let expected = concat!(
            r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" Destination="https://chat.example.com/omniauth/saml/callback?account_id=1" ID="_0d749ed5ea0e6f4800630da48d1f8006" InResponseTo="_324d8ca747d9c07921d2abe9df447832" IssueInstant="2026-08-25T18:51:18.335Z" Version="2.0">"#,
            r#"<saml:Issuer xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">https://auth.example.com/realms/master</saml:Issuer>"#,
            r#"<samlp:Status><samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"></samlp:StatusCode></samlp:Status>"#,
            r#"<saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_018d3ab762d8fd3f7feb90b5164e821f" IssueInstant="2026-08-25T18:51:18.335Z" Version="2.0">"#,
            r#"<saml:Issuer>https://auth.example.com/realms/master</saml:Issuer>"#,
            r#"<saml:Subject>"#,
            r#"<saml:NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">alice@example.com</saml:NameID>"#,
            r#"<saml:SubjectConfirmation Method="urn:oasis:names:tc:SAML:2.0:cm:bearer">"#,
            r#"<saml:SubjectConfirmationData InResponseTo="_324d8ca747d9c07921d2abe9df447832" NotOnOrAfter="2026-08-25T18:56:18.335Z" Recipient="https://chat.example.com/omniauth/saml/callback?account_id=1"></saml:SubjectConfirmationData>"#,
            r#"</saml:SubjectConfirmation>"#,
            r#"</saml:Subject>"#,
            r#"<saml:Conditions NotBefore="2026-08-25T18:50:18.335Z" NotOnOrAfter="2026-08-25T19:01:18.335Z">"#,
            r#"<saml:AudienceRestriction><saml:Audience>https://chat.example.com/saml/sp/1</saml:Audience></saml:AudienceRestriction>"#,
            r#"</saml:Conditions>"#,
            r#"<saml:AuthnStatement AuthnInstant="2026-08-25T18:51:18.335Z" SessionIndex="b1f7c2e4-3d5a-4c8b-9e1f-7a2d6c0b3e59">"#,
            r#"<saml:AuthnContext><saml:AuthnContextClassRef>urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport</saml:AuthnContextClassRef></saml:AuthnContext>"#,
            r#"</saml:AuthnStatement>"#,
            r#"<saml:AttributeStatement>"#,
            r#"<saml:Attribute Name="email" NameFormat="urn:oasis:names:tc:SAML:2.0:attrname-format:basic">"#,
            r#"<saml:AttributeValue>alice@example.com</saml:AttributeValue>"#,
            r#"</saml:Attribute>"#,
            r#"</saml:AttributeStatement>"#,
            r#"</saml:Assertion>"#,
            r#"</samlp:Response>"#,
        );

        let rendered = render_response(&reference_descriptor())
            .expect("the reference descriptor should render");

        assert_eq!(rendered, expected);
    }

    #[test]
    fn the_assertion_redeclares_the_namespace_so_it_verifies_outside_the_envelope() {
        let rendered = render_response(&reference_descriptor())
            .expect("the reference descriptor should render");

        assert!(
            rendered.contains(
                r#"<saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="#
            ),
            "the assertion must carry its own namespace declaration: {rendered}"
        );
    }

    #[test]
    fn the_request_id_is_echoed_on_the_envelope_and_in_the_subject_confirmation() {
        let rendered = render_response(&reference_descriptor())
            .expect("the reference descriptor should render");

        let echoes = rendered
            .matches(r#"InResponseTo="_324d8ca747d9c07921d2abe9df447832""#)
            .count();

        assert_eq!(
            echoes, 2,
            "InResponseTo must appear on the response and on the subject confirmation: {rendered}"
        );
    }

    #[test]
    fn the_subject_confirmation_expires_before_the_conditions_do() {
        let rendered = render_response(&reference_descriptor())
            .expect("the reference descriptor should render");

        assert!(rendered.contains(r#"NotOnOrAfter="2026-08-25T18:56:18.335Z""#));
        assert!(rendered.contains(r#"NotOnOrAfter="2026-08-25T19:01:18.335Z""#));
    }

    #[test]
    fn emitted_document_is_already_canonical() {
        let rendered = render_response(&reference_descriptor())
            .expect("the reference descriptor should render");

        let canonical =
            canonicalize_exclusive(&rendered).expect("the emitted document should be well formed");

        assert_eq!(canonical, rendered);
    }

    #[test]
    fn a_raw_uuid_starting_with_a_digit_is_not_a_usable_element_id() {
        assert!(
            RequestId::parse("0d749ed5ea0e6f4800630da48d1f8006").is_err(),
            "an xsd:ID must not start with a digit"
        );
    }

    #[test]
    fn the_response_and_its_assertion_may_not_share_an_id() {
        let shared = RequestId::parse("_shared").expect("the id should be an NCName");
        let descriptor = ResponseDescriptor {
            response_id: shared.clone(),
            assertion_id: shared,
            ..reference_descriptor()
        };

        assert!(
            matches!(
                render_response(&descriptor),
                Err(ResponseError::DuplicateElementId(_))
            ),
            "an xsd:ID must be unique within a document"
        );
    }

    #[test]
    fn an_acs_url_carrying_a_query_string_is_escaped() {
        let descriptor = ResponseDescriptor {
            destination: AbsoluteUri::parse(
                "Destination",
                "https://chat.example.com/callback?a=1&b=2",
            )
            .expect("the destination should be absolute"),
            ..reference_descriptor()
        };

        let rendered = render_response(&descriptor).expect("an escaped descriptor should render");

        assert!(
            rendered.contains(r#"Destination="https://chat.example.com/callback?a=1&amp;b=2""#),
            "the destination was not escaped: {rendered}"
        );
        assert!(
            rendered.contains(r#"Recipient="https://chat.example.com/callback?a=1&amp;b=2""#),
            "the recipient was not escaped: {rendered}"
        );

        let canonical =
            canonicalize_exclusive(&rendered).expect("the emitted document should be well formed");

        assert_eq!(canonical, rendered);
    }

    #[test]
    fn rejects_an_empty_name_id() {
        let descriptor = ResponseDescriptor {
            name_id: String::new(),
            ..reference_descriptor()
        };

        assert!(matches!(
            render_response(&descriptor),
            Err(ResponseError::EmptyValue("NameID"))
        ));
    }

    #[test]
    fn rejects_an_empty_session_index() {
        let descriptor = ResponseDescriptor {
            session_index: String::new(),
            ..reference_descriptor()
        };

        assert!(matches!(
            render_response(&descriptor),
            Err(ResponseError::EmptyValue("SessionIndex"))
        ));
    }

    #[test]
    fn rejects_an_empty_authn_context_class_ref() {
        let descriptor = ResponseDescriptor {
            authn_context: AuthnContextClassRef::Unrecognised(String::new()),
            ..reference_descriptor()
        };

        assert!(matches!(
            render_response(&descriptor),
            Err(ResponseError::EmptyValue("AuthnContextClassRef"))
        ));
    }

    #[test]
    fn rejects_an_empty_attribute_name() {
        let descriptor = ResponseDescriptor {
            attributes: vec![AssertionAttribute {
                name: String::new(),
                name_format: AttributeNameFormat::Basic,
                values: vec!["x".to_owned()],
            }],
            ..reference_descriptor()
        };

        assert!(matches!(
            render_response(&descriptor),
            Err(ResponseError::EmptyValue("Attribute/@Name"))
        ));
    }

    #[test]
    fn markup_in_a_name_id_and_an_attribute_value_is_escaped() {
        let descriptor = ResponseDescriptor {
            name_id: "a<b>&c".to_owned(),
            attributes: vec![AssertionAttribute {
                name: "display name".to_owned(),
                name_format: AttributeNameFormat::Basic,
                values: vec!["Alice & <Bob>".to_owned()],
            }],
            ..reference_descriptor()
        };

        let rendered = render_response(&descriptor).expect("an escaped descriptor should render");

        assert!(
            rendered.contains(">a&lt;b&gt;&amp;c</saml:NameID>"),
            "{rendered}"
        );
        assert!(
            rendered.contains("<saml:AttributeValue>Alice &amp; &lt;Bob&gt;</saml:AttributeValue>"),
            "{rendered}"
        );

        let canonical =
            canonicalize_exclusive(&rendered).expect("the emitted document should be well formed");

        assert_eq!(canonical, rendered);
    }

    #[test]
    fn an_attribute_statement_is_omitted_when_there_are_no_attributes() {
        let descriptor = ResponseDescriptor {
            attributes: Vec::new(),
            ..reference_descriptor()
        };

        let rendered = render_response(&descriptor).expect("the descriptor should render");

        assert!(
            !rendered.contains("AttributeStatement"),
            "an empty <AttributeStatement> is schema invalid: {rendered}"
        );

        let canonical =
            canonicalize_exclusive(&rendered).expect("the emitted document should be well formed");

        assert_eq!(canonical, rendered);
    }

    #[test]
    fn an_attribute_may_carry_several_values() {
        let descriptor = ResponseDescriptor {
            attributes: vec![AssertionAttribute {
                name: "groups".to_owned(),
                name_format: AttributeNameFormat::Uri,
                values: vec!["admins".to_owned(), "users".to_owned()],
            }],
            ..reference_descriptor()
        };

        let rendered = render_response(&descriptor).expect("the descriptor should render");

        assert!(
            rendered.contains(concat!(
                r#"<saml:Attribute Name="groups" NameFormat="urn:oasis:names:tc:SAML:2.0:attrname-format:uri">"#,
                "<saml:AttributeValue>admins</saml:AttributeValue>",
                "<saml:AttributeValue>users</saml:AttributeValue>",
                "</saml:Attribute>",
            )),
            "{rendered}"
        );
    }

    #[test]
    fn a_window_that_expires_before_it_opens_is_refused() {
        assert!(matches!(
            AssertionWindow::new(
                instant("2026-08-25T19:00:00Z"),
                instant("2026-08-25T19:05:00Z"),
                instant("2026-08-25T18:00:00Z"),
            ),
            Err(ResponseError::EmptyWindow { .. })
        ));
    }

    #[test]
    fn a_subject_confirmation_outliving_the_conditions_is_refused() {
        assert!(matches!(
            AssertionWindow::new(
                instant("2026-08-25T18:50:00Z"),
                instant("2026-08-25T19:30:00Z"),
                instant("2026-08-25T19:01:00Z"),
            ),
            Err(ResponseError::SubjectConfirmationOutlivesConditions { .. })
        ));
    }

    const SIGNING_KEY: &str = include_str!("../tests/fixtures/signing-key.pem");
    const CERTIFICATE: &str = "MIICzzCCAbegAwIBAgIU";

    #[test]
    fn the_signature_lands_inside_the_assertion_not_on_the_envelope() {
        let descriptor = reference_descriptor();
        let signed = render_signed_response(&descriptor, SIGNING_KEY, CERTIFICATE)
            .expect("the reference descriptor should sign");

        let assertion_start = signed
            .find("<saml:Assertion")
            .expect("the assertion should be present");
        let signature_start = signed
            .find("<ds:Signature")
            .expect("the signature should be present");

        assert!(
            signature_start > assertion_start,
            "only the assertion is signed, never the envelope: {signed}"
        );
        assert_eq!(
            signed.matches("</ds:Signature>").count(),
            1,
            "exactly one signature is expected: {signed}"
        );
    }

    #[test]
    fn the_signature_immediately_follows_the_assertion_issuer() {
        let signed = render_signed_response(&reference_descriptor(), SIGNING_KEY, CERTIFICATE)
            .expect("the reference descriptor should sign");

        let assertion_start = signed
            .find("<saml:Assertion")
            .expect("the assertion should be present");
        let issuer_end = signed[assertion_start..]
            .find("</saml:Issuer>")
            .expect("the assertion issuer should be present")
            + assertion_start
            + "</saml:Issuer>".len();

        assert!(
            signed[issuer_end..].starts_with("<ds:Signature"),
            "the signature must be the second child of the assertion: {signed}"
        );
    }

    #[test]
    fn the_signed_document_is_still_canonical() {
        let signed = render_signed_response(&reference_descriptor(), SIGNING_KEY, CERTIFICATE)
            .expect("the reference descriptor should sign");

        let canonical =
            canonicalize_exclusive(&signed).expect("the signed document should be well formed");

        assert_eq!(canonical, signed);
    }

    #[test]
    fn the_signature_covers_the_assertion_not_the_response() {
        let signed = render_signed_response(&reference_descriptor(), SIGNING_KEY, CERTIFICATE)
            .expect("the reference descriptor should sign");

        assert!(
            signed.contains(r##"<ds:Reference URI="#_018d3ab762d8fd3f7feb90b5164e821f">"##),
            "the reference must point at the assertion id: {signed}"
        );
    }
}
