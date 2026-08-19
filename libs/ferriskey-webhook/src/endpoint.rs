//! SSRF-resistant validation for webhook endpoints and headers, shared between webhook writes
//! (`services.rs` calls this at `create_webhook`/`update_webhook` time) and webhook delivery
//! (`core` calls [`is_forbidden_address`] again immediately after resolving the endpoint's host,
//! right before every send). Validating only once, at write time, is not enough: a hostname that
//! resolves to a public address today can resolve to a private one on the next lookup — DNS
//! rebinding — so the address that is actually about to receive the request must be checked too.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};

use thiserror::Error;
use url::{Host, Url};

use ferriskey_domain::common::app_errors::CoreError;

use crate::signing::{DELIVERY_HEADER, SIGNATURE_HEADER, TIMESTAMP_HEADER};

const RESERVED_HEADERS: [&str; 7] = [
    "host",
    "content-type",
    "content-length",
    "transfer-encoding",
    SIGNATURE_HEADER,
    TIMESTAMP_HEADER,
    DELIVERY_HEADER,
];

/// Every reason [`validate_endpoint`] or [`reject_reserved_headers`] can refuse a caller's input.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EndpointError {
    #[error("webhook endpoint could not be parsed as a URL")]
    Malformed,

    #[error("webhook endpoint must use the https scheme")]
    SchemeNotHttps,

    #[error("webhook endpoint must not carry embedded userinfo credentials")]
    EmbeddedCredentials,

    #[error("webhook endpoint has no host")]
    MissingHost,

    #[error("webhook endpoint host could not be resolved")]
    UnresolvableHost,

    #[error("webhook endpoint resolves to an address that must not be reachable")]
    ForbiddenAddress,

    #[error("header '{0}' is reserved and cannot be set on a webhook")]
    ReservedHeader(String),
}

impl From<EndpointError> for CoreError {
    /// `CoreError` has no webhook-endpoint-specific variant to add without touching
    /// `ferriskey-domain`, which is out of scope here, so every rejection collapses to the
    /// existing generic `CoreError::Invalid` ("Invalid resource", HTTP 400). The precise reason
    /// is still available to anyone matching on the `EndpointError` before this conversion runs.
    fn from(_error: EndpointError) -> Self {
        CoreError::Invalid
    }
}

/// Parses `raw`, rejecting it unless it is an `https` URL, carries no userinfo credentials, and
/// has a host whose every resolved address passes [`is_forbidden_address`]. An IP-literal host
/// is checked directly; a domain name is resolved via the system resolver and every returned
/// address is checked, so a name that resolves to both a public and a private address is
/// rejected rather than allowed on the strength of one good answer.
pub fn validate_endpoint(raw: &str) -> Result<Url, EndpointError> {
    let url = Url::parse(raw).map_err(|_| EndpointError::Malformed)?;

    if url.scheme() != "https" {
        return Err(EndpointError::SchemeNotHttps);
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(EndpointError::EmbeddedCredentials);
    }

    let host = url.host().ok_or(EndpointError::MissingHost)?;

    let addresses: Vec<IpAddr> = match host {
        Host::Ipv4(ip) => vec![IpAddr::V4(ip)],
        Host::Ipv6(ip) => vec![IpAddr::V6(ip)],
        Host::Domain(domain) => {
            let port = url
                .port_or_known_default()
                .ok_or(EndpointError::MissingHost)?;

            (domain, port)
                .to_socket_addrs()
                .map_err(|_| EndpointError::UnresolvableHost)?
                .map(|socket_addr| socket_addr.ip())
                .collect()
        }
    };

    if addresses.is_empty() {
        return Err(EndpointError::UnresolvableHost);
    }

    if addresses.into_iter().any(is_forbidden_address) {
        return Err(EndpointError::ForbiddenAddress);
    }

    Ok(url)
}

/// True for any address a webhook endpoint must never reach: loopback, link-local (including the
/// `169.254.169.254` cloud metadata address), private-use ranges, IPv6 unique-local, unspecified,
/// and multicast — plus the IPv4-mapped IPv6 form of every one of those, checked by unwrapping
/// the mapping and recursing into the IPv4 rules, since `::ffff:127.0.0.1` must be rejected
/// exactly like `127.0.0.1` is rather than slipping past a check that only looks at IPv6 ranges.
pub fn is_forbidden_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_forbidden_ipv4(v4),
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(mapped) => is_forbidden_ipv4(mapped),
            None => {
                v6.is_loopback()
                    || v6.is_unspecified()
                    || v6.is_multicast()
                    || is_ipv6_unique_local(v6)
                    || is_ipv6_link_local(v6)
            }
        },
    }
}

fn is_forbidden_ipv4(v4: Ipv4Addr) -> bool {
    v4.is_loopback()
        || v4.is_private()
        || v4.is_link_local()
        || v4.is_unspecified()
        || v4.is_multicast()
}

/// `fc00::/7`, checked on the raw segment rather than via a standard-library helper so this
/// crate's minimum Rust version does not depend on when that helper was stabilized.
fn is_ipv6_unique_local(v6: Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xfe00) == 0xfc00
}

/// `fe80::/10`, checked on the raw segment for the same reason as [`is_ipv6_unique_local`].
fn is_ipv6_link_local(v6: Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xffc0) == 0xfe80
}

/// Refuses any header a caller could use to override transport framing (`host`, `content-type`,
/// `content-length`, `transfer-encoding`) or forge the delivery signature (the
/// [`SIGNATURE_HEADER`], [`TIMESTAMP_HEADER`] and [`DELIVERY_HEADER`] `signing` sets on every
/// outbound request). Compared case-insensitively because HTTP header names are case-insensitive,
/// so `Content-Type` is the same override attempt as `content-type` and a case-sensitive check
/// would let a caller walk straight past it.
pub fn reject_reserved_headers(headers: &HashMap<String, String>) -> Result<(), EndpointError> {
    for key in headers.keys() {
        let normalized = key.to_ascii_lowercase();
        if RESERVED_HEADERS.contains(&normalized.as_str()) {
            return Err(EndpointError::ReservedHeader(key.clone()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_loopback_v4() {
        assert!(is_forbidden_address(IpAddr::V4(Ipv4Addr::new(
            127, 0, 0, 1
        ))));
    }

    #[test]
    fn rejects_unspecified_v4() {
        assert!(is_forbidden_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED)));
    }

    #[test]
    fn rejects_link_local_metadata_address() {
        assert!(is_forbidden_address(IpAddr::V4(Ipv4Addr::new(
            169, 254, 169, 254
        ))));
    }

    #[test]
    fn rejects_private_ranges_v4() {
        assert!(is_forbidden_address(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(is_forbidden_address(IpAddr::V4(Ipv4Addr::new(
            172, 16, 0, 1
        ))));
        assert!(is_forbidden_address(IpAddr::V4(Ipv4Addr::new(
            192, 168, 1, 1
        ))));
    }

    #[test]
    fn rejects_multicast_v4() {
        assert!(is_forbidden_address(IpAddr::V4(Ipv4Addr::new(
            224, 0, 0, 1
        ))));
    }

    #[test]
    fn rejects_loopback_v6() {
        assert!(is_forbidden_address(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn rejects_unspecified_v6() {
        assert!(is_forbidden_address(IpAddr::V6(Ipv6Addr::UNSPECIFIED)));
    }

    #[test]
    fn rejects_multicast_v6() {
        assert!(is_forbidden_address(IpAddr::V6(Ipv6Addr::new(
            0xff02, 0, 0, 0, 0, 0, 0, 1
        ))));
    }

    #[test]
    fn rejects_unique_local_v6() {
        assert!(is_forbidden_address(IpAddr::V6(Ipv6Addr::new(
            0xfd00, 0, 0, 0, 0, 0, 0, 1
        ))));
    }

    #[test]
    fn rejects_link_local_v6() {
        assert!(is_forbidden_address(IpAddr::V6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        ))));
    }

    #[test]
    fn rejects_ipv4_mapped_loopback() {
        assert!(is_forbidden_address(IpAddr::V6(
            Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped()
        )));
    }

    #[test]
    fn rejects_ipv4_mapped_metadata_address() {
        assert!(is_forbidden_address(IpAddr::V6(
            Ipv4Addr::new(169, 254, 169, 254).to_ipv6_mapped()
        )));
    }

    #[test]
    fn rejects_ipv4_mapped_private_range() {
        assert!(is_forbidden_address(IpAddr::V6(
            Ipv4Addr::new(10, 0, 0, 1).to_ipv6_mapped()
        )));
    }

    #[test]
    fn rejects_ipv4_mapped_unspecified() {
        assert!(is_forbidden_address(IpAddr::V6(
            Ipv4Addr::UNSPECIFIED.to_ipv6_mapped()
        )));
    }

    #[test]
    fn allows_public_v4() {
        assert!(!is_forbidden_address(IpAddr::V4(Ipv4Addr::new(
            93, 184, 216, 34
        ))));
    }

    #[test]
    fn allows_public_v6() {
        assert!(!is_forbidden_address(IpAddr::V6(Ipv6Addr::new(
            0x2606, 0x2800, 0x220, 1, 0x248, 0x1893, 0x25c8, 0x1946
        ))));
    }

    #[test]
    fn allows_ipv4_mapped_public_address() {
        assert!(!is_forbidden_address(IpAddr::V6(
            Ipv4Addr::new(93, 184, 216, 34).to_ipv6_mapped()
        )));
    }

    #[test]
    fn rejects_http_scheme() {
        assert_eq!(
            validate_endpoint("http://93.184.216.34/hook"),
            Err(EndpointError::SchemeNotHttps)
        );
    }

    #[test]
    fn rejects_malformed_url() {
        assert_eq!(
            validate_endpoint("not a url"),
            Err(EndpointError::Malformed)
        );
    }

    #[test]
    fn rejects_embedded_credentials() {
        assert_eq!(
            validate_endpoint("https://user:pass@93.184.216.34/hook"),
            Err(EndpointError::EmbeddedCredentials)
        );
    }

    #[test]
    fn rejects_loopback_ip_literal_endpoint() {
        assert_eq!(
            validate_endpoint("https://127.0.0.1/hook"),
            Err(EndpointError::ForbiddenAddress)
        );
    }

    #[test]
    fn rejects_ipv4_mapped_ip_literal_endpoint() {
        assert_eq!(
            validate_endpoint("https://[::ffff:127.0.0.1]/hook"),
            Err(EndpointError::ForbiddenAddress)
        );
    }

    #[test]
    fn rejects_metadata_ip_literal_endpoint() {
        assert_eq!(
            validate_endpoint("https://169.254.169.254/hook"),
            Err(EndpointError::ForbiddenAddress)
        );
    }

    #[test]
    fn accepts_public_ip_literal_endpoint_with_non_default_port() {
        let url = validate_endpoint("https://93.184.216.34:8443/hook")
            .expect("a public IP-literal endpoint on a non-default port must be accepted");

        assert_eq!(url.port(), Some(8443));
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn accepts_public_ip_literal_endpoint_on_default_port() {
        assert!(validate_endpoint("https://93.184.216.34/hook").is_ok());
    }

    #[test]
    fn reject_reserved_headers_rejects_mixed_case_reserved_names() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/xml".to_string());

        assert_eq!(
            reject_reserved_headers(&headers),
            Err(EndpointError::ReservedHeader("Content-Type".to_string()))
        );
    }

    #[test]
    fn reject_reserved_headers_rejects_signature_header_override() {
        let mut headers = HashMap::new();
        headers.insert("X-Ferriskey-Signature".to_string(), "forged".to_string());

        assert_eq!(
            reject_reserved_headers(&headers),
            Err(EndpointError::ReservedHeader(
                "X-Ferriskey-Signature".to_string()
            ))
        );
    }

    #[test]
    fn reject_reserved_headers_allows_custom_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Trace".to_string(), "abc123".to_string());

        assert!(reject_reserved_headers(&headers).is_ok());
    }
}
