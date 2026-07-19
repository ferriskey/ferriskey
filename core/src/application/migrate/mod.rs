pub mod context;
pub mod m0001_seed_default_client_scopes;
pub mod m0002_seed_organization_scope;

use ferriskey_aegis::ports::{
    ClientScopeMappingRepository, ClientScopeRepository, ProtocolMapperRepository,
};
use ferriskey_migrate::runner::MigrationRunner;

use crate::{
    domain::{client::ports::ClientRepository, realm::ports::RealmRepository},
    infrastructure::migrate::repository::PostgresMigrationRepository,
};

use self::{
    context::MigrationContext, m0001_seed_default_client_scopes::SeedDefaultClientScopes,
    m0002_seed_organization_scope::SeedOrganizationScope,
};

/// Builds the migration runner with every registered data migration.
///
/// Call this at application startup, before serving traffic:
///
/// ```rust,ignore
/// let ctx = MigrationContext::new(realm.clone(), client.clone(), ...);
/// build_runner(db.clone()).run(&ctx).await?;
/// ```
pub fn build_runner<R, C, CS, PM, CSM>(
    repository: PostgresMigrationRepository,
) -> MigrationRunner<MigrationContext<R, C, CS, PM, CSM>, PostgresMigrationRepository>
where
    R: RealmRepository + 'static,
    C: ClientRepository + 'static,
    CS: ClientScopeRepository + 'static,
    PM: ProtocolMapperRepository + 'static,
    CSM: ClientScopeMappingRepository + 'static,
{
    MigrationRunner::new(repository)
        .register(SeedDefaultClientScopes)
        .register(SeedOrganizationScope)
}
