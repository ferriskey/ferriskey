use std::sync::Arc;

use crate::domain::{client::ports::ClientService, realm::ports::RealmService};

#[derive(Debug, Clone)]
pub struct AppState<R, C>
where
    R: RealmService,
    C: ClientService,
{
    pub realm_service: Arc<R>,
    pub client_service: Arc<C>,
}

impl<R, C> AppState<R, C>
where
    R: RealmService,
    C: ClientService,
{
    pub fn new(realm_service: Arc<R>, client_service: Arc<C>) -> Self {
        Self {
            realm_service,
            client_service,
        }
    }
}
