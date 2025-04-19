use std::future::Future;
use uuid::Uuid;

use crate::domain::client::entities::{error::ClientError, redirect_uri::RedirectUri};

pub trait RedirectUriService: Clone + Send + Sync + 'static {
    fn add_redirect_uri(
        &self,
        client_id: Uuid,
        uri: String,
    ) -> impl Future<Output = Result<RedirectUri, ClientError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, ClientError>> + Send;

    fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RedirectUri>, ClientError>> + Send;

    fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<RedirectUri, ClientError>> + Send;

    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), ClientError>> + Send;
}
