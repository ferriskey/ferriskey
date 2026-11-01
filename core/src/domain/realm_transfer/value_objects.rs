use std::collections::BTreeMap;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ImportReport {
    pub realm_id: Uuid,
    pub realm_name: String,
    pub created: bool,
    pub rows: BTreeMap<String, u64>,
    pub ignored_columns: Vec<String>,
}

pub type BundleChunk = Result<Vec<u8>, CoreError>;

#[derive(Debug)]
pub struct RealmBundleStream {
    chunks: mpsc::Receiver<BundleChunk>,
}

impl RealmBundleStream {
    pub fn new(chunks: mpsc::Receiver<BundleChunk>) -> Self {
        Self { chunks }
    }
}

impl Stream for RealmBundleStream {
    type Item = BundleChunk;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.chunks.poll_recv(cx)
    }
}
