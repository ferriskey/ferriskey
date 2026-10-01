use std::time::Duration;

use reqwest::{Client, Url, redirect};

use crate::domain::authentication::backchannel_logout::{LogoutDeliveryError, LogoutTokenSender};
use crate::domain::webhook::endpoint::{PrivateEndpoints, allows_cleartext, is_forbidden_address};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// POSTs logout tokens with the same guards as webhook delivery: the host is
/// resolved once and the request pinned to an address this deployment may
/// reach, cleartext only to an allowed loopback, and no redirects.
#[derive(Debug, Clone, Copy)]
pub struct HttpLogoutTokenSender {
    private_endpoints: PrivateEndpoints,
}

impl HttpLogoutTokenSender {
    pub fn new(private_endpoints: PrivateEndpoints) -> Self {
        Self { private_endpoints }
    }
}

impl LogoutTokenSender for HttpLogoutTokenSender {
    async fn send(
        &self,
        endpoint: String,
        logout_token: String,
    ) -> Result<(), LogoutDeliveryError> {
        let url = Url::parse(&endpoint).map_err(|_| LogoutDeliveryError::InvalidEndpoint)?;
        if url.scheme() != "https" && url.scheme() != "http" {
            return Err(LogoutDeliveryError::InvalidEndpoint);
        }
        let host = url
            .host_str()
            .ok_or(LogoutDeliveryError::InvalidEndpoint)?
            .to_string();
        let port = url
            .port_or_known_default()
            .ok_or(LogoutDeliveryError::InvalidEndpoint)?;

        let mut resolved = tokio::net::lookup_host((host.as_str(), port))
            .await
            .map_err(|err| LogoutDeliveryError::Transport(err.to_string()))?;
        let addr = resolved
            .find(|candidate| !is_forbidden_address(candidate.ip(), self.private_endpoints))
            .ok_or(LogoutDeliveryError::ForbiddenAddress)?;

        if url.scheme() != "https" && !allows_cleartext(addr.ip(), self.private_endpoints) {
            return Err(LogoutDeliveryError::ForbiddenAddress);
        }

        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(redirect::Policy::none())
            .resolve(&host, addr)
            .build()
            .map_err(|err| LogoutDeliveryError::Transport(err.to_string()))?;

        // §2.5: `application/x-www-form-urlencoded`, a single `logout_token`.
        let response = client
            .post(url)
            .form(&[("logout_token", logout_token)])
            .send()
            .await
            .map_err(|err| LogoutDeliveryError::Transport(err.to_string()))?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(LogoutDeliveryError::Rejected(status.as_u16()))
        }
    }
}
