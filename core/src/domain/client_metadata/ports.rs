use crate::domain::client_metadata::entities::{ClientMetadataError, FetchedClientMetadata};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmScope;

#[cfg_attr(test, mockall::automock)]
pub trait ClientMetadataDocumentFetcher: Send + Sync {
    fn fetch(
        &self,
        url: &str,
    ) -> impl Future<Output = Result<FetchedClientMetadata, ClientMetadataError>> + Send;
}

pub trait ClientMetadataResolver: Send + Sync {
    fn resolve(
        &self,
        realm: &RealmScope,
        client_id: &str,
        redirect_uri: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
