//! Client domain port traits live in the shared `ferriskey-domain` crate. `ClientRepository` and
//! `RedirectUriRepository` use `#[cfg_attr(any(test, feature = "mock"), mockall::automock)]`;
//! `core` enables ferriskey-domain's `mock` feature, so `MockClientRepository` /
//! `MockRedirectUriRepository` come through this glob re-export (same convention as user/realm).
pub use ferriskey_domain::client::ports::*;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use ferriskey_domain::client::entities::redirect_uri::RedirectUri;

#[cfg_attr(test, mockall::automock)]
pub trait PostLogoutRedirectUriRepository: Send + Sync {
    fn create_redirect_uri(
        &self,
        client_id: Uuid,
        value: String,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, CoreError>> + Send;

    fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, CoreError>> + Send;

    fn update_enabled(
        &self,
        client_id: Uuid,
        id: Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;

    fn delete(
        &self,
        client_id: Uuid,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
