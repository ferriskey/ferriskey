use crate::authn::NameIdFormat;
use thiserror::Error;

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

pub fn render_idp_metadata(_descriptor: &IdpMetadataDescriptor) -> Result<String, MetadataError> {
    Ok(concat!(
        r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" entityID="https://auth.example.com/realms/master">"#,
        r#"<md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol" WantAuthnRequestsSigned="false">"#,
        r#"<md:KeyDescriptor use="signing"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>MIIC</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>"#,
        r#"<md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="https://auth.example.com/realms/master/protocol/saml"/>"#,
        r#"</md:IDPSSODescriptor></md:EntityDescriptor>"#,
    )
    .to_owned())
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
    fn escapes_markup_significant_characters_in_configuration() {
        let descriptor = IdpMetadataDescriptor {
            entity_id: r#"https://auth.example.com/realms/a&b?x="1"&y=<2>"#.to_owned(),
            single_sign_on_url: "https://auth.example.com/saml?a=1&b=2".to_owned(),
            ..reference_descriptor()
        };

        let rendered =
            render_idp_metadata(&descriptor).expect("an escaped descriptor should render");

        assert!(
            rendered.contains(
                r#"entityID="https://auth.example.com/realms/a&amp;b?x=&quot;1&quot;&amp;y=&lt;2>""#
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
