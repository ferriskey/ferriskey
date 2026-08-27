use std::borrow::Cow;
use std::io::Write;

use base64::Engine;
use base64::alphabet::STANDARD as STANDARD_ALPHABET;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use flate2::write::DeflateEncoder;
use flate2::{Compression, Decompress, FlushDecompress, Status};
use thiserror::Error;

pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 512 * 1024;

const INFLATE_CHUNK_BYTES: usize = 8 * 1024;

const LENIENT_BASE64: GeneralPurpose = GeneralPurpose::new(
    &STANDARD_ALPHABET,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BindingError {
    #[error("saml message is not valid base64: {0}")]
    NotBase64(String),
    #[error("saml message is not a valid deflate stream")]
    NotDeflate,
    #[error("saml message exceeds the {limit} byte limit")]
    TooLarge { limit: usize },
    #[error("saml message is not valid utf-8")]
    NotUtf8,
    #[error("saml message could not be compressed")]
    NotCompressible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectParameter {
    SamlRequest,
    SamlResponse,
}

impl RedirectParameter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SamlRequest => "SAMLRequest",
            Self::SamlResponse => "SAMLResponse",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectQuery {
    parameter: RedirectParameter,
    message: String,
    relay_state: Option<String>,
}

impl RedirectQuery {
    pub fn new(parameter: RedirectParameter, message: impl Into<String>) -> Self {
        Self {
            parameter,
            message: message.into(),
            relay_state: None,
        }
    }

    pub fn with_relay_state(mut self, relay_state: impl Into<String>) -> Self {
        self.relay_state = Some(relay_state.into());
        self
    }

    pub fn parameter(&self) -> RedirectParameter {
        self.parameter
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn relay_state(&self) -> Option<&str> {
        self.relay_state.as_deref()
    }

    pub fn signing_input(&self, signature_algorithm: &str) -> String {
        let mut input = self.to_query_string();
        input.push_str("&SigAlg=");
        input.push_str(&percent_encode(signature_algorithm));
        input
    }

    pub fn to_query_string(&self) -> String {
        let mut query = String::new();
        query.push_str(self.parameter.as_str());
        query.push('=');
        query.push_str(&percent_encode(&self.message));

        if let Some(relay_state) = &self.relay_state {
            query.push_str("&RelayState=");
            query.push_str(&percent_encode(relay_state));
        }

        query
    }

    pub fn to_signed_query_string(&self, signature_algorithm: &str, signature: &str) -> String {
        let mut query = self.signing_input(signature_algorithm);
        query.push_str("&Signature=");
        query.push_str(&percent_encode(signature));
        query
    }
}

pub fn encode_redirect(xml: &str) -> Result<String, BindingError> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(xml.as_bytes())
        .map_err(|_| BindingError::NotCompressible)?;
    let compressed = encoder
        .finish()
        .map_err(|_| BindingError::NotCompressible)?;

    Ok(BASE64.encode(compressed))
}

pub fn decode_redirect(value: &str) -> Result<String, BindingError> {
    decode_redirect_with_limit(value, DEFAULT_MAX_MESSAGE_BYTES)
}

pub fn decode_redirect_with_limit(value: &str, limit: usize) -> Result<String, BindingError> {
    let compressed = decode_base64(value)?;
    into_utf8(inflate_bounded(&compressed, limit)?)
}

pub fn encode_post(xml: &str) -> String {
    BASE64.encode(xml.as_bytes())
}

pub fn decode_post(value: &str) -> Result<String, BindingError> {
    decode_post_with_limit(value, DEFAULT_MAX_MESSAGE_BYTES)
}

pub fn decode_post_with_limit(value: &str, limit: usize) -> Result<String, BindingError> {
    let normalised = strip_whitespace(value);
    if normalised.len() / 4 * 3 > limit {
        return Err(BindingError::TooLarge { limit });
    }

    let decoded = LENIENT_BASE64
        .decode(normalised.as_bytes())
        .map_err(|error| BindingError::NotBase64(error.to_string()))?;
    if decoded.len() > limit {
        return Err(BindingError::TooLarge { limit });
    }

    into_utf8(decoded)
}

fn inflate_bounded(compressed: &[u8], limit: usize) -> Result<Vec<u8>, BindingError> {
    let mut decompressor = Decompress::new(false);
    let mut inflated = Vec::new();
    let mut chunk = [0_u8; INFLATE_CHUNK_BYTES];

    loop {
        let consumed = decompressor.total_in() as usize;
        let produced = decompressor.total_out() as usize;

        let status = decompressor
            .decompress(&compressed[consumed..], &mut chunk, FlushDecompress::None)
            .map_err(|_| BindingError::NotDeflate)?;

        let written = decompressor.total_out() as usize - produced;
        if inflated.len() + written > limit {
            return Err(BindingError::TooLarge { limit });
        }
        inflated.extend_from_slice(&chunk[..written]);

        match status {
            Status::StreamEnd => return Ok(inflated),
            Status::BufError => return Err(BindingError::NotDeflate),
            Status::Ok => {
                if written == 0 && decompressor.total_in() as usize == consumed {
                    return Err(BindingError::NotDeflate);
                }
            }
        }
    }
}

fn decode_base64(value: &str) -> Result<Vec<u8>, BindingError> {
    LENIENT_BASE64
        .decode(strip_whitespace(value).as_bytes())
        .map_err(|error| BindingError::NotBase64(error.to_string()))
}

fn into_utf8(bytes: Vec<u8>) -> Result<String, BindingError> {
    String::from_utf8(bytes).map_err(|_| BindingError::NotUtf8)
}

fn strip_whitespace(value: &str) -> Cow<'_, str> {
    if value.bytes().any(|byte| byte.is_ascii_whitespace()) {
        Cow::Owned(
            value
                .chars()
                .filter(|character| !character.is_ascii_whitespace())
                .collect(),
        )
    } else {
        Cow::Borrowed(value)
    }
}

fn percent_encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                encoded.push('%');
                encoded.push(char::from(HEX[usize::from(byte >> 4)]));
                encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::io::Write;

    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as BASE64;
    use flate2::Compression;
    use flate2::read::DeflateDecoder;
    use flate2::read::ZlibDecoder;
    use flate2::write::DeflateEncoder;

    use super::{
        BindingError, DEFAULT_MAX_MESSAGE_BYTES, RedirectParameter, RedirectQuery, decode_post,
        decode_post_with_limit, decode_redirect, decode_redirect_with_limit, encode_post,
        encode_redirect,
    };

    const AUTHN_REQUEST: &str = concat!(
        r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" "#,
        r#"xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" "#,
        r#"ID="_4d9f2b1c8e7a4f3b9c1d6e5a8b7c0d2f" Version="2.0" "#,
        r#"IssueInstant="2026-08-27T09:14:32Z" "#,
        r#"Destination="https://idp.ferriskey.rs/realms/master/protocol/saml" "#,
        r#"ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" "#,
        r#"AssertionConsumerServiceURL="https://sp.example.com/saml/acs" "#,
        r#"ForceAuthn="false" IsPassive="false">"#,
        r#"<saml:Issuer>https://sp.example.com/metadata</saml:Issuer>"#,
        r#"<samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:2.0:nameid-format:persistent" "#,
        r#"AllowCreate="true"/>"#,
        r#"</samlp:AuthnRequest>"#,
    );

    fn deflate(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).expect("write to deflate encoder");
        encoder.finish().expect("finish deflate encoder")
    }

    #[test]
    fn redirect_round_trip_returns_the_original_document() {
        let encoded = encode_redirect(AUTHN_REQUEST).expect("encode redirect");
        assert_eq!(
            decode_redirect(&encoded).expect("decode redirect"),
            AUTHN_REQUEST
        );
    }

    #[test]
    fn redirect_encoding_is_raw_deflate_and_not_zlib_wrapped() {
        let encoded = encode_redirect(AUTHN_REQUEST).expect("encode redirect");
        let compressed = BASE64.decode(&encoded).expect("decode base64");

        let mut raw = String::new();
        DeflateDecoder::new(compressed.as_slice())
            .read_to_string(&mut raw)
            .expect("raw deflate stream");
        assert_eq!(raw, AUTHN_REQUEST);

        let mut wrapped = Vec::new();
        assert!(
            ZlibDecoder::new(compressed.as_slice())
                .read_to_end(&mut wrapped)
                .is_err(),
            "a zlib header must not be emitted on the redirect binding"
        );
    }

    #[test]
    fn redirect_encoding_compresses_a_realistic_authn_request() {
        let raw = AUTHN_REQUEST.len();
        assert!(
            (560..=700).contains(&raw),
            "fixture drifted away from the reference size: {raw}"
        );

        let encoded = encode_redirect(AUTHN_REQUEST).expect("encode redirect");
        let compressed = BASE64.decode(&encoded).expect("decode base64").len();
        let ratio = compressed * 100 / raw;

        assert!(
            (45..=70).contains(&ratio),
            "deflate ratio {ratio}% is outside the 45-70% band expected of raw deflate"
        );
        assert!(
            encoded.len() < raw * 4 / 3,
            "the redirect binding must stay smaller than plain base64"
        );
    }

    #[test]
    fn post_round_trip_returns_the_original_document() {
        let encoded = encode_post(AUTHN_REQUEST);
        assert_eq!(decode_post(&encoded).expect("decode post"), AUTHN_REQUEST);
    }

    #[test]
    fn post_encoding_is_plain_base64_without_compression() {
        let encoded = encode_post(AUTHN_REQUEST);
        assert_eq!(
            BASE64.decode(&encoded).expect("decode base64"),
            AUTHN_REQUEST.as_bytes()
        );
        assert!(
            encoded.len() > AUTHN_REQUEST.len(),
            "plain base64 always grows the document"
        );
    }

    #[test]
    fn decode_accepts_line_wrapped_base64() {
        let encoded = encode_post(AUTHN_REQUEST);
        let wrapped = encoded
            .as_bytes()
            .chunks(64)
            .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(decode_post(&wrapped).expect("decode post"), AUTHN_REQUEST);
    }

    #[test]
    fn decode_rejects_input_that_is_not_base64() {
        let error = decode_redirect("not base64 !!").expect_err("must reject");
        assert!(matches!(error, BindingError::NotBase64(_)));
        assert!(matches!(
            decode_post("not base64 !!").expect_err("must reject"),
            BindingError::NotBase64(_)
        ));
    }

    #[test]
    fn decode_redirect_rejects_a_malformed_deflate_stream() {
        let garbage = BASE64.encode([0xff_u8; 64]);
        assert_eq!(
            decode_redirect(&garbage).expect_err("must reject"),
            BindingError::NotDeflate
        );
    }

    #[test]
    fn decode_redirect_rejects_a_truncated_deflate_stream() {
        let mut compressed = deflate(AUTHN_REQUEST.as_bytes());
        compressed.truncate(compressed.len() / 2);

        assert_eq!(
            decode_redirect(&BASE64.encode(&compressed)).expect_err("must reject"),
            BindingError::NotDeflate
        );
    }

    #[test]
    fn decode_redirect_rejects_a_decompression_bomb() {
        let bomb = BASE64.encode(deflate(&vec![b'A'; 4 * 1024 * 1024]));
        let limit = 64 * 1024;

        assert!(
            bomb.len() < 16 * 1024,
            "the bomb must stay small on the wire to be a bomb at all"
        );
        assert_eq!(
            decode_redirect_with_limit(&bomb, limit).expect_err("must reject"),
            BindingError::TooLarge { limit }
        );
    }

    #[test]
    fn decode_redirect_accepts_a_payload_that_sits_on_the_limit() {
        let payload = "x".repeat(1024);
        let encoded = encode_redirect(&payload).expect("encode redirect");

        assert_eq!(
            decode_redirect_with_limit(&encoded, 1024).expect("decode redirect"),
            payload
        );
        assert_eq!(
            decode_redirect_with_limit(&encoded, 1023).expect_err("must reject"),
            BindingError::TooLarge { limit: 1023 }
        );
    }

    #[test]
    fn decode_post_rejects_an_oversized_payload() {
        let encoded = encode_post(&"x".repeat(8 * 1024));
        assert_eq!(
            decode_post_with_limit(&encoded, 1024).expect_err("must reject"),
            BindingError::TooLarge { limit: 1024 }
        );
    }

    #[test]
    fn decode_rejects_a_payload_that_is_not_utf8() {
        let encoded = BASE64.encode([0xff_u8, 0xfe, 0xfd]);
        assert_eq!(
            decode_post(&encoded).expect_err("must reject"),
            BindingError::NotUtf8
        );

        let deflated = BASE64.encode(deflate(&[0xff_u8, 0xfe, 0xfd]));
        assert_eq!(
            decode_redirect(&deflated).expect_err("must reject"),
            BindingError::NotUtf8
        );
    }

    #[test]
    fn the_limitless_entry_points_still_apply_the_default_limit() {
        let oversized = "x".repeat(DEFAULT_MAX_MESSAGE_BYTES + 1);

        assert_eq!(
            decode_redirect(&BASE64.encode(deflate(oversized.as_bytes())))
                .expect_err("must reject"),
            BindingError::TooLarge {
                limit: DEFAULT_MAX_MESSAGE_BYTES
            }
        );
        assert_eq!(
            decode_post(&encode_post(&oversized)).expect_err("must reject"),
            BindingError::TooLarge {
                limit: DEFAULT_MAX_MESSAGE_BYTES
            }
        );
    }

    #[test]
    fn signing_input_orders_the_parameters_and_percent_encodes_the_values() {
        let query = RedirectQuery::new(RedirectParameter::SamlRequest, "fVJJT+MwFP4r")
            .with_relay_state("/app/accounts/1/dashboard");

        assert_eq!(
            query.signing_input("http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"),
            concat!(
                "SAMLRequest=fVJJT%2BMwFP4r",
                "&RelayState=%2Fapp%2Faccounts%2F1%2Fdashboard",
                "&SigAlg=http%3A%2F%2Fwww.w3.org%2F2001%2F04%2Fxmldsig-more%23rsa-sha256",
            )
        );
    }

    #[test]
    fn signing_input_omits_the_relay_state_when_it_is_absent() {
        let query = RedirectQuery::new(RedirectParameter::SamlResponse, "fVJJT+MwFP4r");

        assert_eq!(
            query.signing_input("http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"),
            concat!(
                "SAMLResponse=fVJJT%2BMwFP4r",
                "&SigAlg=http%3A%2F%2Fwww.w3.org%2F2001%2F04%2Fxmldsig-more%23rsa-sha256",
            )
        );
    }

    #[test]
    fn the_unsigned_query_string_carries_only_the_message_and_the_relay_state() {
        let query = RedirectQuery::new(RedirectParameter::SamlRequest, "fVJJT+MwFP4r")
            .with_relay_state("/app/accounts/1/dashboard");

        assert_eq!(
            query.to_query_string(),
            "SAMLRequest=fVJJT%2BMwFP4r&RelayState=%2Fapp%2Faccounts%2F1%2Fdashboard"
        );
        assert_eq!(
            RedirectQuery::new(RedirectParameter::SamlRequest, "abc").to_query_string(),
            "SAMLRequest=abc"
        );
    }

    #[test]
    fn the_signed_query_string_is_the_signing_input_plus_the_signature() {
        let query = RedirectQuery::new(RedirectParameter::SamlRequest, "fVJJT+MwFP4r")
            .with_relay_state("/app/accounts/1/dashboard");
        let algorithm = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";

        assert_eq!(
            query.to_signed_query_string(algorithm, "c2ln+bmF0dXJl=="),
            format!(
                "{}&Signature=c2ln%2BbmF0dXJl%3D%3D",
                query.signing_input(algorithm)
            )
        );
    }

    #[test]
    fn a_query_string_round_trips_through_the_redirect_codec() {
        let query = RedirectQuery::new(
            RedirectParameter::SamlRequest,
            encode_redirect(AUTHN_REQUEST).expect("encode redirect"),
        )
        .with_relay_state("/app/accounts/1/dashboard");

        assert_eq!(query.parameter(), RedirectParameter::SamlRequest);
        assert_eq!(query.relay_state(), Some("/app/accounts/1/dashboard"));
        assert_eq!(
            decode_redirect(query.message()).expect("decode redirect"),
            AUTHN_REQUEST
        );
        assert!(query.to_query_string().starts_with("SAMLRequest="));
    }

    #[test]
    fn the_parameter_name_matches_the_saml_binding_specification() {
        assert_eq!(RedirectParameter::SamlRequest.as_str(), "SAMLRequest");
        assert_eq!(RedirectParameter::SamlResponse.as_str(), "SAMLResponse");
    }
}
