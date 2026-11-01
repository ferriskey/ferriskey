use futures::Stream;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::domain::realm_transfer::bundle::{BundleHeader, BundlePassphrase, BundleRow};
use crate::domain::realm_transfer::value_objects::{ImportReport, RealmBundleStream};

pub trait RowSink: Send {
    fn accept(&mut self, row: BundleRow) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait RowSource: Send {
    fn next_row(&mut self) -> impl Future<Output = Result<Option<BundleRow>, CoreError>> + Send;
}

pub trait RealmArchiveRepository: Send + Sync + 'static {
    fn schema_version(&self) -> impl Future<Output = Result<i64, CoreError>> + Send;

    fn export_rows<K: RowSink>(
        &self,
        realm_id: RealmId,
        sink: &mut K,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn import_rows<S: RowSource>(
        &self,
        header: &BundleHeader,
        source: &mut S,
    ) -> impl Future<Output = Result<ImportReport, CoreError>> + Send;
}

pub trait RealmTransferService: Send + Sync {
    fn export_realm(
        &self,
        identity: Identity,
        realm_name: String,
        passphrase: BundlePassphrase,
    ) -> impl Future<Output = Result<RealmBundleStream, CoreError>> + Send;

    fn import_realm<S, B, E>(
        &self,
        identity: Identity,
        passphrase: BundlePassphrase,
        body: S,
    ) -> impl Future<Output = Result<ImportReport, CoreError>> + Send
    where
        S: Stream<Item = Result<B, E>> + Send + Unpin,
        B: AsRef<[u8]> + Send,
        E: std::fmt::Display + Send;
}
