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

/// Whether a webhook endpoint may resolve to a loopback or private address.
///
/// `Forbidden` is the [`Default`] so that a code path which forgets to thread the setting through
/// fails closed. Link-local addresses stay refused under both values: `169.254.169.254` and its
/// IPv6 equivalent serve cloud instance metadata, which is the SSRF payoff this guard exists for
/// and which no local development setup needs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PrivateEndpoints {
    #[default]
    Forbidden,
    Allowed,
}

impl PrivateEndpoints {
    pub fn from_allowed(allowed: bool) -> Self {
        if allowed {
            Self::Allowed
        } else {
            Self::Forbidden
        }
    }

    fn allows_private(self) -> bool {
        matches!(self, Self::Allowed)
    }
}

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
    /// `UnresolvableHost` and `ForbiddenAddress` deliberately share one message. Telling them
    /// apart would answer "does this name resolve, and is it internal?" for any caller who can
    /// create a webhook, which is the probe an SSRF attempt starts with. Every other variant
    /// describes the submitted string itself and discloses nothing about the network.
    fn from(error: EndpointError) -> Self {
        let message = match error {
            EndpointError::Malformed => "the endpoint is not a valid URL",
            EndpointError::SchemeNotHttps => "the endpoint must use https",
            EndpointError::EmbeddedCredentials => {
                "the endpoint must not embed credentials in the URL"
            }
            EndpointError::MissingHost => "the endpoint has no host",
            EndpointError::UnresolvableHost | EndpointError::ForbiddenAddress => {
                "the endpoint must be a publicly reachable address"
            }
            EndpointError::ReservedHeader(_) => {
                return CoreError::InvalidWebhookEndpoint(error.to_string());
            }
        };

        CoreError::InvalidWebhookEndpoint(message.to_string())
    }
}

/// Parses `raw`, rejecting it unless it is an `https` URL, carries no userinfo credentials, and
/// has a host whose every resolved address passes [`is_forbidden_address`]. An IP-literal host
/// is checked directly; a domain name is resolved via the system resolver and every returned
/// address is checked, so a name that resolves to both a public and a private address is
/// rejected rather than allowed on the strength of one good answer.
pub fn validate_endpoint(raw: &str, policy: PrivateEndpoints) -> Result<Url, EndpointError> {
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

    if addresses
        .into_iter()
        .any(|address| is_forbidden_address(address, policy))
    {
        return Err(EndpointError::ForbiddenAddress);
    }

    Ok(url)
}

/// True for any address a webhook endpoint must never reach: loopback, link-local (including the
/// `169.254.169.254` cloud metadata address), private-use ranges, IPv6 unique-local, unspecified,
/// and multicast — plus the IPv4-mapped IPv6 form of every one of those, checked by unwrapping
/// the mapping and recursing into the IPv4 rules, since `::ffff:127.0.0.1` must be rejected
/// exactly like `127.0.0.1` is rather than slipping past a check that only looks at IPv6 ranges.
pub fn is_forbidden_address(ip: IpAddr, policy: PrivateEndpoints) -> bool {
    match ip {
        IpAddr::V4(v4) => is_forbidden_ipv4(v4, policy),
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(mapped) => is_forbidden_ipv4(mapped, policy),
            None => {
                v6.is_unspecified()
                    || v6.is_multicast()
                    || is_ipv6_link_local(v6)
                    || (!policy.allows_private() && (v6.is_loopback() || is_ipv6_unique_local(v6)))
            }
        },
    }
}

fn is_forbidden_ipv4(v4: Ipv4Addr, policy: PrivateEndpoints) -> bool {
    v4.is_unspecified()
        || v4.is_multicast()
        || v4.is_link_local()
        || (!policy.allows_private() && (v4.is_loopback() || v4.is_private()))
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
    fn forbidden(ip: std::net::IpAddr) -> bool {
        super::is_forbidden_address(ip, super::PrivateEndpoints::Forbidden)
    }

    fn validate(raw: &str) -> Result<super::Url, super::EndpointError> {
        super::validate_endpoint(raw, super::PrivateEndpoints::Forbidden)
    }

    use super::*;

    #[test]
    fn rejects_loopback_v4() {
        assert!(forbidden(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    }

    #[test]
    fn rejects_unspecified_v4() {
        assert!(forbidden(IpAddr::V4(Ipv4Addr::UNSPECIFIED)));
    }

    #[test]
    fn rejects_link_local_metadata_address() {
        assert!(forbidden(IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))));
    }

    #[test]
    fn rejects_private_ranges_v4() {
        assert!(forbidden(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(forbidden(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
        assert!(forbidden(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
    }

    #[test]
    fn rejects_multicast_v4() {
        assert!(forbidden(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1))));
    }

    #[test]
    fn rejects_loopback_v6() {
        assert!(forbidden(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn rejects_unspecified_v6() {
        assert!(forbidden(IpAddr::V6(Ipv6Addr::UNSPECIFIED)));
    }

    #[test]
    fn rejects_multicast_v6() {
        assert!(forbidden(IpAddr::V6(Ipv6Addr::new(
            0xff02, 0, 0, 0, 0, 0, 0, 1
        ))));
    }

    #[test]
    fn rejects_unique_local_v6() {
        assert!(forbidden(IpAddr::V6(Ipv6Addr::new(
            0xfd00, 0, 0, 0, 0, 0, 0, 1
        ))));
    }

    #[test]
    fn rejects_link_local_v6() {
        assert!(forbidden(IpAddr::V6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        ))));
    }

    #[test]
    fn rejects_ipv4_mapped_loopback() {
        assert!(forbidden(IpAddr::V6(
            Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped()
        )));
    }

    #[test]
    fn rejects_ipv4_mapped_metadata_address() {
        assert!(forbidden(IpAddr::V6(
            Ipv4Addr::new(169, 254, 169, 254).to_ipv6_mapped()
        )));
    }

    #[test]
    fn rejects_ipv4_mapped_private_range() {
        assert!(forbidden(IpAddr::V6(
            Ipv4Addr::new(10, 0, 0, 1).to_ipv6_mapped()
        )));
    }

    #[test]
    fn rejects_ipv4_mapped_unspecified() {
        assert!(forbidden(IpAddr::V6(
            Ipv4Addr::UNSPECIFIED.to_ipv6_mapped()
        )));
    }

    #[test]
    fn allows_public_v4() {
        assert!(!forbidden(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))));
    }

    #[test]
    fn allows_public_v6() {
        assert!(!forbidden(IpAddr::V6(Ipv6Addr::new(
            0x2606, 0x2800, 0x220, 1, 0x248, 0x1893, 0x25c8, 0x1946
        ))));
    }

    #[test]
    fn allows_ipv4_mapped_public_address() {
        assert!(!forbidden(IpAddr::V6(
            Ipv4Addr::new(93, 184, 216, 34).to_ipv6_mapped()
        )));
    }

    #[test]
    fn rejects_http_scheme() {
        assert_eq!(
            validate("http://93.184.216.34/hook"),
            Err(EndpointError::SchemeNotHttps)
        );
    }

    #[test]
    fn rejects_malformed_url() {
        assert_eq!(validate("not a url"), Err(EndpointError::Malformed));
    }

    #[test]
    fn rejects_embedded_credentials() {
        assert_eq!(
            validate("https://user:pass@93.184.216.34/hook"),
            Err(EndpointError::EmbeddedCredentials)
        );
    }

    #[test]
    fn rejects_loopback_ip_literal_endpoint() {
        assert_eq!(
            validate("https://127.0.0.1/hook"),
            Err(EndpointError::ForbiddenAddress)
        );
    }

    #[test]
    fn rejects_ipv4_mapped_ip_literal_endpoint() {
        assert_eq!(
            validate("https://[::ffff:127.0.0.1]/hook"),
            Err(EndpointError::ForbiddenAddress)
        );
    }

    #[test]
    fn rejects_metadata_ip_literal_endpoint() {
        assert_eq!(
            validate("https://169.254.169.254/hook"),
            Err(EndpointError::ForbiddenAddress)
        );
    }

    #[test]
    fn accepts_public_ip_literal_endpoint_with_non_default_port() {
        let url = validate("https://93.184.216.34:8443/hook")
            .expect("a public IP-literal endpoint on a non-default port must be accepted");

        assert_eq!(url.port(), Some(8443));
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn accepts_public_ip_literal_endpoint_on_default_port() {
        assert!(validate("https://93.184.216.34/hook").is_ok());
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

#[cfg(test)]
mod conversion_tests {
    use super::*;

    fn message(error: EndpointError) -> String {
        match CoreError::from(error) {
            CoreError::InvalidWebhookEndpoint(message) => message,
            other => panic!("expected InvalidWebhookEndpoint, got {other:?}"),
        }
    }

    #[test]
    fn a_rejection_says_what_to_change() {
        assert!(message(EndpointError::SchemeNotHttps).contains("https"));
        assert!(message(EndpointError::EmbeddedCredentials).contains("credentials"));
        assert!(message(EndpointError::Malformed).contains("valid URL"));
        assert!(message(EndpointError::MissingHost).contains("host"));
    }

    #[test]
    fn an_unresolvable_host_is_indistinguishable_from_a_forbidden_one() {
        assert_eq!(
            message(EndpointError::UnresolvableHost),
            message(EndpointError::ForbiddenAddress),
            "telling these apart would answer whether a name resolves to an internal address"
        );
    }

    #[test]
    fn no_message_echoes_the_submitted_host() {
        for error in [
            EndpointError::UnresolvableHost,
            EndpointError::ForbiddenAddress,
        ] {
            let message = message(error);
            assert!(!message.contains("localhost"));
            assert!(!message.contains("127.0.0.1"));
        }
    }

    #[test]
    fn a_reserved_header_names_the_offending_header() {
        let message = message(EndpointError::ReservedHeader(
            "x-ferriskey-signature".to_string(),
        ));

        assert!(message.contains("x-ferriskey-signature"));
    }
}

#[cfg(test)]
mod private_endpoint_tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn allowed(ip: IpAddr) -> bool {
        !is_forbidden_address(ip, PrivateEndpoints::Allowed)
    }

    fn refused_by_default(ip: IpAddr) -> bool {
        is_forbidden_address(ip, PrivateEndpoints::Forbidden)
    }

    #[test]
    fn the_default_is_to_refuse() {
        assert_eq!(PrivateEndpoints::default(), PrivateEndpoints::Forbidden);
        assert_eq!(
            PrivateEndpoints::from_allowed(false),
            PrivateEndpoints::Forbidden
        );
    }

    #[test]
    fn opting_in_admits_loopback_and_private_ranges() {
        for ip in [
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
        ] {
            assert!(refused_by_default(ip), "{ip} must be refused by default");
            assert!(allowed(ip), "{ip} must be admitted once opted in");
        }
    }

    #[test]
    fn opting_in_still_refuses_link_local_metadata_addresses() {
        for ip in [
            IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)),
            IpAddr::V4(Ipv4Addr::new(169, 254, 0, 1)),
            IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
        ] {
            assert!(
                !allowed(ip),
                "{ip} serves cloud instance metadata and must stay refused even when opted in"
            );
        }
    }

    #[test]
    fn opting_in_still_refuses_unspecified_and_multicast() {
        for ip in [
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1)),
            IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        ] {
            assert!(!allowed(ip), "{ip} is never a valid endpoint");
        }
    }

    #[test]
    fn opting_in_does_not_relax_the_https_requirement() {
        assert_eq!(
            validate_endpoint("http://127.0.0.1/hook", PrivateEndpoints::Allowed),
            Err(EndpointError::SchemeNotHttps)
        );
    }

    #[test]
    fn a_loopback_endpoint_is_accepted_once_opted_in() {
        assert!(validate_endpoint("https://127.0.0.1/hook", PrivateEndpoints::Allowed).is_ok());
        assert_eq!(
            validate_endpoint("https://127.0.0.1/hook", PrivateEndpoints::Forbidden),
            Err(EndpointError::ForbiddenAddress)
        );
    }

    #[test]
    fn opting_in_does_not_relax_the_credentials_rule() {
        assert_eq!(
            validate_endpoint(
                "https://user:pass@127.0.0.1/hook",
                PrivateEndpoints::Allowed
            ),
            Err(EndpointError::EmbeddedCredentials)
        );
    }
}
