use ferriskey_core::domain::saml::entities::idp_entity_id;
use ferriskey_saml::authn::NameIdFormat;
use ferriskey_saml::metadata::{IdpMetadataDescriptor, MetadataError, render_idp_metadata};

pub const SAML_METADATA_CONTENT_TYPE: &str = "application/samlmetadata+xml";

pub fn single_sign_on_url(public_base_url: &str, realm_name: &str) -> String {
    format!(
        "{}/realms/{realm_name}/protocol/saml",
        public_base_url.trim_end_matches('/')
    )
}

pub fn idp_metadata_document(
    public_base_url: &str,
    realm_name: &str,
    signing_certificate_base64_der: String,
) -> Result<String, MetadataError> {
    render_idp_metadata(&IdpMetadataDescriptor {
        entity_id: idp_entity_id(public_base_url, realm_name),
        single_sign_on_url: single_sign_on_url(public_base_url, realm_name),
        name_id_formats: vec![
            NameIdFormat::EmailAddress,
            NameIdFormat::Persistent,
            NameIdFormat::Transient,
            NameIdFormat::Unspecified,
        ],
        want_authn_requests_signed: false,
        signing_certificate_base64_der,
    })
}

#[cfg(test)]
mod tests {
    use super::{idp_metadata_document, single_sign_on_url};

    use ferriskey_saml::metadata::MetadataError;

    const CERTIFICATE: &str = "MIICzzCCAbegAwIBAgIUAKZ2QpVn3xY0Rq9tLd8sFbEwT1UwDQYJKoZIhvcNAQEL";

    #[test]
    fn the_single_sign_on_url_is_the_route_both_bindings_are_served_on() {
        assert_eq!(
            single_sign_on_url("https://auth.example.com/", "master"),
            "https://auth.example.com/realms/master/protocol/saml"
        );
    }

    #[test]
    fn the_advertised_entity_id_is_the_one_the_assertion_is_issued_under() {
        let document = idp_metadata_document(
            "https://auth.example.com",
            "master",
            CERTIFICATE.to_string(),
        )
        .expect("the descriptor is complete");

        assert!(document.contains(r#"entityID="https://auth.example.com/realms/master""#));
    }

    #[test]
    fn both_bindings_are_advertised_at_the_single_sign_on_url() {
        let document = idp_metadata_document(
            "https://auth.example.com",
            "master",
            CERTIFICATE.to_string(),
        )
        .expect("the descriptor is complete");

        assert!(document.contains(
            r#"<md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="https://auth.example.com/realms/master/protocol/saml">"#
        ));
        assert!(document.contains(
            r#"<md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="https://auth.example.com/realms/master/protocol/saml">"#
        ));
    }

    #[test]
    fn only_the_name_id_formats_the_assertion_builder_can_produce_are_advertised() {
        let document = idp_metadata_document(
            "https://auth.example.com",
            "master",
            CERTIFICATE.to_string(),
        )
        .expect("the descriptor is complete");

        for format in [
            "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
            "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent",
            "urn:oasis:names:tc:SAML:2.0:nameid-format:transient",
            "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified",
        ] {
            assert!(
                document.contains(&format!("<md:NameIDFormat>{format}</md:NameIDFormat>")),
                "{format} is not advertised: {document}"
            );
        }

        assert!(
            !document.contains("nameid-format:kerberos"),
            "a format the assertion builder cannot produce must not be advertised: {document}"
        );
    }

    #[test]
    fn signed_authn_requests_are_not_advertised_while_no_signature_is_verified() {
        let document = idp_metadata_document(
            "https://auth.example.com",
            "master",
            CERTIFICATE.to_string(),
        )
        .expect("the descriptor is complete");

        assert!(document.contains(r#"WantAuthnRequestsSigned="false""#));
    }

    #[test]
    fn a_realm_without_a_signing_certificate_yields_no_document_at_all() {
        assert_eq!(
            idp_metadata_document("https://auth.example.com", "master", String::new()),
            Err(MetadataError::EmptyValue("X509Certificate"))
        );
    }
}
