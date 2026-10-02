use std::net::SocketAddr;

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::HeaderMap;

use crate::app_state::AppState;

const MAX_USER_AGENT_BYTES: usize = 512;

pub fn client_ip(
    headers: &HeaderMap,
    peer: Option<SocketAddr>,
    trust_forwarded_for: bool,
) -> Option<String> {
    if trust_forwarded_for {
        let forwarded = headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if let Some(forwarded) = forwarded {
            return Some(forwarded.to_string());
        }
    }

    peer.map(|addr| addr.ip().to_string())
}

pub fn user_agent(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("user-agent")?;
    let raw = std::str::from_utf8(raw.as_bytes()).ok()?;

    if raw.len() <= MAX_USER_AGENT_BYTES {
        return Some(raw.to_string());
    }

    let mut cut = MAX_USER_AGENT_BYTES;
    while cut > 0 && !raw.is_char_boundary(cut) {
        cut -= 1;
    }

    Some(raw[..cut].to_string())
}

pub struct RequestContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl FromRequestParts<AppState> for RequestContext {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| *addr);

        Ok(RequestContext {
            ip_address: client_ip(&parts.headers, peer, state.args.server.trust_forwarded_for),
            user_agent: user_agent(&parts.headers),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{client_ip, user_agent};
    use axum::http::{HeaderMap, HeaderValue};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn peer(ip: [u8; 4], port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)
    }

    fn headers_with(name: &'static str, value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            name,
            HeaderValue::from_str(value).expect("a valid header value"),
        );
        headers
    }

    #[test]
    fn the_forwarded_header_is_ignored_when_the_flag_is_off_and_the_peer_is_used_instead() {
        let headers = headers_with("x-forwarded-for", "203.0.113.5");

        let result = client_ip(&headers, Some(peer([198, 51, 100, 7], 51820)), false);

        assert_eq!(result, Some("198.51.100.7".to_string()));
    }

    #[test]
    fn the_leftmost_entry_of_a_multi_hop_forwarded_header_wins_when_the_flag_is_on() {
        let headers = headers_with(
            "x-forwarded-for",
            "203.0.113.5, 70.41.3.18, 150.172.238.178",
        );

        let result = client_ip(&headers, Some(peer([10, 0, 0, 1], 8080)), true);

        assert_eq!(result, Some("203.0.113.5".to_string()));
    }

    #[test]
    fn the_peer_is_used_when_the_flag_is_on_but_the_header_is_absent() {
        let headers = HeaderMap::new();

        let result = client_ip(&headers, Some(peer([198, 51, 100, 9], 443)), true);

        assert_eq!(result, Some("198.51.100.9".to_string()));
    }

    #[test]
    fn no_peer_and_no_header_yields_none() {
        let headers = HeaderMap::new();

        let result = client_ip(&headers, None, true);

        assert_eq!(result, None);
    }

    #[test]
    fn the_peer_address_never_carries_its_port() {
        let headers = HeaderMap::new();

        let result = client_ip(&headers, Some(peer([93, 184, 216, 34], 51820)), false);

        assert_eq!(result, Some("93.184.216.34".to_string()));
        assert!(!result.expect("an ip address").contains(':'));
    }

    #[test]
    fn a_user_agent_longer_than_the_cap_is_truncated() {
        let long_agent = "a".repeat(600);
        let headers = headers_with("user-agent", &long_agent);

        let result = user_agent(&headers).expect("a truncated user agent");

        assert_eq!(result.len(), 512);
        assert_eq!(result, "a".repeat(512));
    }

    #[test]
    fn truncation_lands_on_a_char_boundary_instead_of_splitting_a_multi_byte_character() {
        let mut long_agent = "a".repeat(511);
        long_agent.push('é');
        long_agent.push_str(&"b".repeat(100));
        let headers = headers_with("user-agent", &long_agent);

        let result = user_agent(&headers).expect("a truncated user agent");

        assert_eq!(result, "a".repeat(511));
        assert!(result.len() < 512);
    }

    #[test]
    fn an_absent_user_agent_yields_none() {
        let headers = HeaderMap::new();

        assert_eq!(user_agent(&headers), None);
    }
}
