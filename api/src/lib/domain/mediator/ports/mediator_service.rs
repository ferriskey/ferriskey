use std::future::Future;

pub trait MediatorService: Clone + Send + Sync + 'static {
    fn initialize_master_realm(&self) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
}
