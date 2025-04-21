use std::future::Future;
use uuid::Uuid;

use crate::{
    application::http::client::validators::CreateRedirectUriValidator,
    domain::client::entities::{
        error::ClientError, redirect_uri::RedirectUri, redirect_uri_error::RedirectUriError,
    },
};

pub trait RedirectUriService: Clone + Send + Sync + 'static {
    fn add_redirect_uri(
        &self,
        schema: CreateRedirectUriValidator,
        realm_name: String,
        client_id: Uuid,
    ) -> impl Future<Output = Result<RedirectUri, ClientError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, RedirectUriError>> + Send;

    fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, RedirectUriError>> + Send;

    fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, RedirectUriError>> + Send;

    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), RedirectUriError>> + Send;
}
