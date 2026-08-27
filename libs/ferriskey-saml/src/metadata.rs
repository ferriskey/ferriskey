use crate::authn::{NameIdFormat, ProtocolBinding};
use thiserror::Error;

const METADATA_NAMESPACE: &str = "urn:oasis:names:tc:SAML:2.0:metadata";
const XMLDSIG_NAMESPACE: &str = "http://www.w3.org/2000/09/xmldsig#";
const PROTOCOL_NAMESPACE: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
const SIGNING_KEY_USE: &str = "signing";
const SINGLE_SIGN_ON_BINDINGS: [ProtocolBinding; 2] =
    [ProtocolBinding::HttpRedirect, ProtocolBinding::HttpPost];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MetadataError {
    #[error("`{0}` is required but empty")]
    EmptyValue(&'static str),
    #[error("at least one NameID format must be advertised")]
    NoNameIdFormats,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdpMetadataDescriptor {
    pub entity_id: String,
    pub single_sign_on_url: String,
    pub name_id_formats: Vec<NameIdFormat>,
    pub want_authn_requests_signed: bool,
    pub signing_certificate_base64_der: String,
}

pub fn render_idp_metadata(descriptor: &IdpMetadataDescriptor) -> Result<String, MetadataError> {
    let entity_id = non_empty(&descriptor.entity_id, "entityID")?;
    let single_sign_on_url = non_empty(
        &descriptor.single_sign_on_url,
        "SingleSignOnService/@Location",
    )?;
    let certificate = non_empty(
        &descriptor.signing_certificate_base64_der,
        "X509Certificate",
    )?;

    if descriptor.name_id_formats.is_empty() {
        return Err(MetadataError::NoNameIdFormats);
    }
    let name_id_formats = descriptor
        .name_id_formats
        .iter()
        .map(|format| non_empty(format.as_uri(), "NameIDFormat"))
        .collect::<Result<Vec<_>, _>>()?;

    let want_authn_requests_signed = if descriptor.want_authn_requests_signed {
        "true"
    } else {
        "false"
    };

    let mut xml = String::new();

    open_element(
        &mut xml,
        "md:EntityDescriptor",
        &[("xmlns:md", METADATA_NAMESPACE), ("entityID", entity_id)],
    );
    open_element(
        &mut xml,
        "md:IDPSSODescriptor",
        &[
            ("WantAuthnRequestsSigned", want_authn_requests_signed),
            ("protocolSupportEnumeration", PROTOCOL_NAMESPACE),
        ],
    );

    open_element(&mut xml, "md:KeyDescriptor", &[("use", SIGNING_KEY_USE)]);
    open_element(&mut xml, "ds:KeyInfo", &[("xmlns:ds", XMLDSIG_NAMESPACE)]);
    open_element(&mut xml, "ds:X509Data", &[]);
    text_element(&mut xml, "ds:X509Certificate", certificate);
    close_element(&mut xml, "ds:X509Data");
    close_element(&mut xml, "ds:KeyInfo");
    close_element(&mut xml, "md:KeyDescriptor");

    for format in name_id_formats {
        text_element(&mut xml, "md:NameIDFormat", format);
    }

    for binding in SINGLE_SIGN_ON_BINDINGS {
        open_element(
            &mut xml,
            "md:SingleSignOnService",
            &[
                ("Binding", binding.as_uri()),
                ("Location", single_sign_on_url),
            ],
        );
        close_element(&mut xml, "md:SingleSignOnService");
    }

    close_element(&mut xml, "md:IDPSSODescriptor");
    close_element(&mut xml, "md:EntityDescriptor");

    Ok(xml)
}

fn non_empty<'a>(value: &'a str, label: &'static str) -> Result<&'a str, MetadataError> {
    if value.is_empty() {
        return Err(MetadataError::EmptyValue(label));
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
    use super::{IdpMetadataDescriptor, MetadataError, render_idp_metadata};
    use crate::authn::NameIdFormat;
    use crate::c14n::canonicalize_exclusive;

    const CERTIFICATE: &str = "MIICzzCCAbegAwIBAgIUAKZ2QpVn3xY0Rq9tLd8sFbEwT1UwDQYJKoZIhvcNAQEL";

    fn reference_descriptor() -> IdpMetadataDescriptor {
        IdpMetadataDescriptor {
            entity_id: "https://auth.example.com/realms/master".to_owned(),
            single_sign_on_url: "https://auth.example.com/realms/master/protocol/saml".to_owned(),
            name_id_formats: vec![NameIdFormat::EmailAddress, NameIdFormat::Persistent],
            want_authn_requests_signed: false,
            signing_certificate_base64_der: CERTIFICATE.to_owned(),
        }
    }

    #[test]
    fn renders_the_reference_descriptor() {
        let expected = concat!(
            r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" entityID="https://auth.example.com/realms/master">"#,
            r#"<md:IDPSSODescriptor WantAuthnRequestsSigned="false" protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">"#,
            r#"<md:KeyDescriptor use="signing">"#,
            r#"<ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#"><ds:X509Data><ds:X509Certificate>"#,
            "MIICzzCCAbegAwIBAgIUAKZ2QpVn3xY0Rq9tLd8sFbEwT1UwDQYJKoZIhvcNAQEL",
            r#"</ds:X509Certificate></ds:X509Data></ds:KeyInfo>"#,
            r#"</md:KeyDescriptor>"#,
            r#"<md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>"#,
            r#"<md:NameIDFormat>urn:oasis:names:tc:SAML:2.0:nameid-format:persistent</md:NameIDFormat>"#,
            r#"<md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="https://auth.example.com/realms/master/protocol/saml"></md:SingleSignOnService>"#,
            r#"<md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="https://auth.example.com/realms/master/protocol/saml"></md:SingleSignOnService>"#,
            r#"</md:IDPSSODescriptor></md:EntityDescriptor>"#,
        );

        let rendered = render_idp_metadata(&reference_descriptor())
            .expect("the reference descriptor should render");

        assert_eq!(rendered, expected);
    }

    #[test]
    fn emitted_document_is_already_canonical() {
        let rendered = render_idp_metadata(&reference_descriptor())
            .expect("the reference descriptor should render");

        let canonical =
            canonicalize_exclusive(&rendered).expect("the emitted document should be well formed");

        assert_eq!(canonical, rendered);
    }

    #[test]
    fn escapes_markup_and_whitespace_in_configuration() {
        let descriptor = IdpMetadataDescriptor {
            entity_id: "https://auth.example.com/realms/a&b?x=\"1\"&y=<2>\tz".to_owned(),
            single_sign_on_url: "https://auth.example.com/saml?a=1&b=2".to_owned(),
            ..reference_descriptor()
        };

        let rendered =
            render_idp_metadata(&descriptor).expect("an escaped descriptor should render");

        assert!(
            rendered.contains(
                r#"entityID="https://auth.example.com/realms/a&amp;b?x=&quot;1&quot;&amp;y=&lt;2>&#x9;z""#
            ),
            "entity id was not escaped: {rendered}"
        );
        assert!(
            rendered.contains(r#"Location="https://auth.example.com/saml?a=1&amp;b=2""#),
            "single sign on url was not escaped: {rendered}"
        );

        let canonical =
            canonicalize_exclusive(&rendered).expect("the emitted document should be well formed");

        assert_eq!(canonical, rendered);
    }

    #[test]
    fn escapes_markup_significant_characters_in_name_id_formats() {
        let descriptor = IdpMetadataDescriptor {
            name_id_formats: vec![NameIdFormat::Unrecognised(
                "urn:example:format?a=1&b=<2>".to_owned(),
            )],
            ..reference_descriptor()
        };

        let rendered =
            render_idp_metadata(&descriptor).expect("an escaped descriptor should render");

        assert!(
            rendered.contains(
                "<md:NameIDFormat>urn:example:format?a=1&amp;b=&lt;2&gt;</md:NameIDFormat>"
            ),
            "name id format was not escaped: {rendered}"
        );

        let canonical =
            canonicalize_exclusive(&rendered).expect("the emitted document should be well formed");

        assert_eq!(canonical, rendered);
    }

    #[test]
    fn advertises_signed_authn_requests_when_requested() {
        let descriptor = IdpMetadataDescriptor {
            want_authn_requests_signed: true,
            ..reference_descriptor()
        };

        let rendered =
            render_idp_metadata(&descriptor).expect("the reference descriptor should render");

        assert!(
            rendered.contains(r#"WantAuthnRequestsSigned="true""#),
            "want authn requests signed was not advertised: {rendered}"
        );
    }

    #[test]
    fn rejects_an_empty_entity_id() {
        let descriptor = IdpMetadataDescriptor {
            entity_id: String::new(),
            ..reference_descriptor()
        };

        assert_eq!(
            render_idp_metadata(&descriptor),
            Err(MetadataError::EmptyValue("entityID"))
        );
    }

    #[test]
    fn rejects_an_empty_single_sign_on_url() {
        let descriptor = IdpMetadataDescriptor {
            single_sign_on_url: String::new(),
            ..reference_descriptor()
        };

        assert_eq!(
            render_idp_metadata(&descriptor),
            Err(MetadataError::EmptyValue("SingleSignOnService/@Location"))
        );
    }

    #[test]
    fn rejects_an_empty_certificate() {
        let descriptor = IdpMetadataDescriptor {
            signing_certificate_base64_der: String::new(),
            ..reference_descriptor()
        };

        assert_eq!(
            render_idp_metadata(&descriptor),
            Err(MetadataError::EmptyValue("X509Certificate"))
        );
    }

    #[test]
    fn rejects_an_empty_name_id_format_list() {
        let descriptor = IdpMetadataDescriptor {
            name_id_formats: Vec::new(),
            ..reference_descriptor()
        };

        assert_eq!(
            render_idp_metadata(&descriptor),
            Err(MetadataError::NoNameIdFormats)
        );
    }

    #[test]
    fn rejects_an_empty_name_id_format_uri() {
        let descriptor = IdpMetadataDescriptor {
            name_id_formats: vec![NameIdFormat::Unrecognised(String::new())],
            ..reference_descriptor()
        };

        assert_eq!(
            render_idp_metadata(&descriptor),
            Err(MetadataError::EmptyValue("NameIDFormat"))
        );
    }
}
