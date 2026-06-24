use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use utoipa::ToSchema;

use super::entities::SecurityEvent;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditPiiMode {
    #[default]
    Off,
    Mask,
    Pseudonymise,
}

impl AuditPiiMode {
    pub fn as_str(&self) -> &str {
        match self {
            AuditPiiMode::Off => "off",
            AuditPiiMode::Mask => "mask",
            AuditPiiMode::Pseudonymise => "pseudonymise",
        }
    }
}

impl std::str::FromStr for AuditPiiMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "mask" => AuditPiiMode::Mask,
            "pseudonymise" => AuditPiiMode::Pseudonymise,
            _ => AuditPiiMode::Off,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PiiConfig {
    pub mode: AuditPiiMode,
    pub pseudo_key: Option<String>,
}

impl Default for PiiConfig {
    fn default() -> Self {
        Self {
            mode: AuditPiiMode::Off,
            pseudo_key: None,
        }
    }
}

pub fn apply_to_event(mut event: SecurityEvent, cfg: &PiiConfig) -> SecurityEvent {
    match cfg.mode {
        AuditPiiMode::Off => event,
        AuditPiiMode::Mask => {
            event.ip_address = event.ip_address.map(|ip| mask_ip(&ip));
            event.user_agent = event.user_agent.map(|ua| mask_user_agent(&ua));
            event
        }
        AuditPiiMode::Pseudonymise => {
            let key = cfg.pseudo_key.as_deref().unwrap_or_default();
            event.ip_address = event.ip_address.map(|ip| pseudonymise(&ip, key));
            event.user_agent = event.user_agent.map(|ua| pseudonymise(&ua, key));
            event
        }
    }
}

pub fn mask_ip(ip: &str) -> String {
    if ip.contains(':') {
        mask_ipv6(ip)
    } else {
        mask_ipv4(ip)
    }
}

fn mask_ipv4(ip: &str) -> String {
    let parts: Vec<&str> = ip.splitn(4, '.').collect();
    if parts.len() == 4 {
        format!("{}.{}.0.0", parts[0], parts[1])
    } else {
        "0.0.0.0".to_string()
    }
}

fn mask_ipv6(ip: &str) -> String {
    let groups: Vec<&str> = ip.splitn(9, ':').collect();
    let kept = groups.iter().take(3).cloned().collect::<Vec<_>>().join(":");
    if groups.len() >= 3 {
        format!("{}::", kept)
    } else {
        "::".to_string()
    }
}

pub fn mask_email(email: &str) -> String {
    let mut parts = email.splitn(2, '@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");

    let masked_local = if local.is_empty() {
        "***".to_string()
    } else {
        let first = &local[..local.len().min(1)];
        format!("{}***", first)
    };

    let masked_domain = if domain.is_empty() {
        "***".to_string()
    } else {
        let first = &domain[..domain.len().min(1)];
        format!("{}***", first)
    };

    format!("{}@{}", masked_local, masked_domain)
}

pub fn mask_user_agent(ua: &str) -> String {
    ua.split('/').next().unwrap_or(ua).trim().to_string()
}

pub fn pseudonymise(value: &str, key: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .unwrap_or_else(|_| HmacSha256::new_from_slice(b"fallback").expect("valid key"));
    mac.update(value.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(&result[..8])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realm::entities::RealmId;
    use crate::domain::seawatch::entities::{EventStatus, SecurityEvent, SecurityEventType};
    use uuid::Uuid;

    fn make_event() -> SecurityEvent {
        SecurityEvent::new(
            RealmId::from(Uuid::new_v4()),
            SecurityEventType::LoginSuccess,
            EventStatus::Success,
            Uuid::new_v4(),
        )
        .with_context(
            Some("192.168.1.42".to_string()),
            Some("Mozilla/5.0 (X11; Linux x86_64) Gecko/20100101 Firefox/128.0".to_string()),
            None,
        )
    }

    #[test]
    fn test_off_mode_unchanged() {
        let event = make_event();
        let orig_ip = event.ip_address.clone();
        let orig_ua = event.user_agent.clone();
        let cfg = PiiConfig::default();
        let out = apply_to_event(event, &cfg);
        assert_eq!(out.ip_address, orig_ip);
        assert_eq!(out.user_agent, orig_ua);
    }

    #[test]
    fn test_mask_ipv4() {
        assert_eq!(mask_ip("192.168.1.42"), "192.168.0.0");
        assert_eq!(mask_ip("10.0.0.1"), "10.0.0.0");
    }

    #[test]
    fn test_mask_ipv6() {
        let masked = mask_ip("2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert!(masked.starts_with("2001:0db8:85a3::"), "got: {masked}");
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("alice@example.com"), "a***@e***");
        assert_eq!(mask_email("bob@foo.org"), "b***@f***");
    }

    #[test]
    fn test_mask_user_agent_family_only() {
        assert_eq!(mask_user_agent("Mozilla/5.0 ..."), "Mozilla");
        assert_eq!(mask_user_agent("curl/7.88.1"), "curl");
        assert_eq!(mask_user_agent("Go-http-client/2.0"), "Go-http-client");
    }

    #[test]
    fn test_mask_mode_applies_to_event() {
        let event = make_event();
        let cfg = PiiConfig {
            mode: AuditPiiMode::Mask,
            pseudo_key: None,
        };
        let out = apply_to_event(event, &cfg);
        assert_eq!(out.ip_address.as_deref(), Some("192.168.0.0"));
        assert_eq!(out.user_agent.as_deref(), Some("Mozilla"));
    }

    #[test]
    fn test_pseudonymise_deterministic() {
        let token1 = pseudonymise("192.168.1.42", "secret-key");
        let token2 = pseudonymise("192.168.1.42", "secret-key");
        assert_eq!(token1, token2);
        assert_eq!(token1.len(), 16);
    }

    #[test]
    fn test_pseudonymise_different_inputs_differ() {
        let t1 = pseudonymise("192.168.1.1", "key");
        let t2 = pseudonymise("192.168.1.2", "key");
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_pseudonymise_key_isolation() {
        let t1 = pseudonymise("192.168.1.1", "realm-a-key");
        let t2 = pseudonymise("192.168.1.1", "realm-b-key");
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_pseudonymise_mode_applies_to_event() {
        let event = make_event();
        let key = "test-key";
        let cfg = PiiConfig {
            mode: AuditPiiMode::Pseudonymise,
            pseudo_key: Some(key.to_string()),
        };
        let out = apply_to_event(event.clone(), &cfg);

        let expected_ip = pseudonymise("192.168.1.42", key);
        assert_eq!(out.ip_address.as_deref(), Some(expected_ip.as_str()));

        let expected_ua = pseudonymise(
            "Mozilla/5.0 (X11; Linux x86_64) Gecko/20100101 Firefox/128.0",
            key,
        );
        assert_eq!(out.user_agent.as_deref(), Some(expected_ua.as_str()));
    }

    #[test]
    fn test_pseudonymise_not_raw() {
        let value = "192.168.1.42";
        let token = pseudonymise(value, "some-key");
        assert_ne!(token, value);
        assert!(!token.contains('.'));
    }
}
