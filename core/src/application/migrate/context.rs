use std::sync::Arc;

use ferriskey_aegis::ports::{
    ClientScopeMappingRepository, ClientScopeRepository, ProtocolMapperRepository,
};

use crate::domain::{client::ports::ClientRepository, realm::ports::RealmRepository};

/// Shared execution context passed to every data migration.
///
/// All fields are trait objects so migrations depend on ports, not infrastructure.
/// Add fields as new migrations require access to additional repositories.
#[derive(Clone)]
pub struct MigrationContext<R, C, CS, PM, CSM>
where
    R: RealmRepository,
    C: ClientRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
    CSM: ClientScopeMappingRepository,
{
    pub realm_repository: Arc<R>,
    pub client_repository: Arc<C>,
    pub client_scope_repository: Arc<CS>,
    pub protocol_mapper_repository: Arc<PM>,
    pub scope_mapping_repository: Arc<CSM>,
}

impl<R, C, CS, PM, CSM> MigrationContext<R, C, CS, PM, CSM>
where
    R: RealmRepository,
    C: ClientRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
    CSM: ClientScopeMappingRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        client_scope_repository: Arc<CS>,
        protocol_mapper_repository: Arc<PM>,
        scope_mapping_repository: Arc<CSM>,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            client_scope_repository,
            protocol_mapper_repository,
            scope_mapping_repository,
        }
    }
}
