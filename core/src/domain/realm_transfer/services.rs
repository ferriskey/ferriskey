use std::fmt::Display;
use std::sync::Arc;

use chrono::Utc;
use futures::Stream;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{instrument, warn};

use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::ports::ClientRepository;
use crate::domain::client::value_objects::CreateClientRequest;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::policies::{FerriskeyPolicy, ensure_policy};
use crate::domain::realm::entities::{RealmId, RealmScope};
use crate::domain::realm::ports::{RealmPolicy, RealmRepository};
use crate::domain::realm_transfer::bundle::{BundleHeader, BundlePassphrase, BundleRealm};
use crate::domain::realm_transfer::codec::{BundleReader, BundleWriter, derive_key};
use crate::domain::realm_transfer::ports::{RealmArchiveRepository, RealmTransferService};
use crate::domain::realm_transfer::value_objects::{
    BundleChunk, ImportMode, ImportReport, RealmBundleStream,
};
use crate::domain::role::entities::permission::Permissions;
use crate::domain::role::ports::RoleRepository;
use crate::domain::role::value_objects::CreateRoleRequest;
use crate::domain::seawatch::{
    EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType,
};
use crate::domain::user::ports::{UserRepository, UserRoleRepository};
use ferriskey_security::crypto::bundle_envelope::BundleKey;

const MASTER_REALM: &str = "master";
const REALM_TARGET: &str = "realm";
const EXPORT_BUFFERED_CHUNKS: usize = 8;
const MIRROR_CLIENT_STEP: &str = "management_client";
const AUDIT_STEP: &str = "security_event";

fn transfer_event(
    realm_id: RealmId,
    identity: &Identity,
    event_type: SecurityEventType,
    status: EventStatus,
    details: serde_json::Value,
) -> SecurityEvent {
    SecurityEvent::new(realm_id, event_type, status, identity.id())
        .with_target(REALM_TARGET.to_string(), realm_id.into(), None)
        .with_details(details)
}

async fn export_bundle<A: RealmArchiveRepository>(
    archive: &A,
    realm_id: RealmId,
    key: BundleKey,
    header: BundleHeader,
    out: mpsc::Sender<BundleChunk>,
) -> bool {
    let mut writer = match BundleWriter::start(key, header, out.clone()).await {
        Ok(writer) => writer,
        Err(error) => {
            let _ = out.send(Err(error)).await;
            return false;
        }
    };
    if let Err(error) = archive.export_rows(realm_id, &mut writer).await {
        warn!(%error, "the realm export stopped before its end");
        writer.fail(error).await;
        return false;
    }
    match writer.finish().await {
        Ok(()) => true,
        Err(error) => {
            warn!(%error, "the realm export could not be completed");
            let _ = out.send(Err(error)).await;
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct RealmTransferServiceImpl<R, C, RO, U, UR, SE, A>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository,
    A: RealmArchiveRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) archive: Arc<A>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

pub struct RealmTransferRepositories<R, C, RO, U, UR, SE, A> {
    pub realm: Arc<R>,
    pub client: Arc<C>,
    pub role: Arc<RO>,
    pub user: Arc<U>,
    pub user_role: Arc<UR>,
    pub security_event: Arc<SE>,
    pub archive: Arc<A>,
}

impl<R, C, RO, U, UR, SE, A> RealmTransferServiceImpl<R, C, RO, U, UR, SE, A>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository,
    A: RealmArchiveRepository,
{
    pub fn new(
        repositories: RealmTransferRepositories<R, C, RO, U, UR, SE, A>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository: repositories.realm,
            client_repository: repositories.client,
            role_repository: repositories.role,
            user_repository: repositories.user,
            user_role_repository: repositories.user_role,
            security_event_repository: repositories.security_event,
            archive: repositories.archive,
            policy,
        }
    }

    async fn record(
        &self,
        realm_id: RealmId,
        identity: &Identity,
        event_type: SecurityEventType,
        details: serde_json::Value,
    ) -> Result<(), CoreError> {
        self.security_event_repository
            .store_event(transfer_event(
                realm_id,
                identity,
                event_type,
                EventStatus::Success,
                details,
            ))
            .await
    }

    async fn ensure_mirror_client(
        &self,
        identity: &Identity,
        master: &RealmScope,
        realm_name: &str,
        mode: ImportMode,
    ) -> Result<(), CoreError> {
        let name = format!("{realm_name}-realm");
        let existing = match self
            .client_repository
            .get_by_client_id(name.clone(), master.id())
            .await
        {
            Ok(client) => Some(client),
            Err(CoreError::NotFound) => None,
            Err(error) => return Err(error),
        };
        // Before the realm exists, a client holding its name is an orphan: its
        // role would hand ManageRealm to whoever held it, as in `create_realm`.
        let existing = match (existing, mode) {
            (Some(orphan), ImportMode::Create) => {
                let orphan = orphan.in_realm(master)?;
                self.client_repository.delete_by_id(&orphan).await?;
                None
            }
            (existing, _) => existing,
        };
        let client = match existing {
            Some(client) => client.across_realms(),
            None => {
                self.client_repository
                    .create_client(CreateClientRequest::create_realm_system_client(
                        master.id(),
                        name.clone(),
                    ))
                    .await?
            }
        };

        let existing_role = self
            .role_repository
            .get_by_client_id(client.id)
            .await?
            .into_iter()
            .find(|role| role.name == name);
        let role = match existing_role {
            Some(role) => role,
            None => {
                self.role_repository
                    .create(CreateRoleRequest {
                        client_id: Some(client.id),
                        description: None,
                        name,
                        permissions: vec![Permissions::ManageRealm.name()],
                        realm_id: master.id(),
                    })
                    .await?
            }
        };

        let user_id = match identity {
            Identity::User(user) => user.id,
            Identity::Client(client) => {
                self.user_repository
                    .get_by_client_id(client.id)
                    .await?
                    .across_realms()
                    .id
            }
        };
        let assigned = self.user_role_repository.get_user_roles(user_id).await?;
        if assigned.iter().any(|held| held.id == role.id) {
            return Ok(());
        }
        self.user_role_repository
            .assign_role(user_id, role.id)
            .await
    }

    async fn check_import_target(
        &self,
        identity: &Identity,
        header: &BundleHeader,
    ) -> Result<ImportMode, CoreError> {
        let realm = &header.realm;
        let Some(existing) = self.realm_repository.get_by_id(realm.id.into()).await? else {
            return Ok(ImportMode::Create);
        };
        if existing.name != realm.name {
            return Err(CoreError::RealmBundleConflict(format!(
                "the realm id {} belongs to the realm {:?} here",
                realm.id, existing.name
            )));
        }
        ensure_policy(
            self.policy.can_update_realm(identity, &existing).await,
            "insufficient permissions",
        )?;
        Ok(ImportMode::Update)
    }
}

impl<R, C, RO, U, UR, SE, A> RealmTransferService
    for RealmTransferServiceImpl<R, C, RO, U, UR, SE, A>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository + 'static,
    A: RealmArchiveRepository,
{
    #[instrument(skip(self, identity, passphrase))]
    async fn export_realm(
        &self,
        identity: Identity,
        realm_name: String,
        passphrase: BundlePassphrase,
    ) -> Result<RealmBundleStream, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        if !scope.realm().can_delete() {
            return Err(CoreError::MasterRealmNotTransferable);
        }
        let master = RealmScope::resolve(self.realm_repository.as_ref(), MASTER_REALM).await?;
        ensure_policy(
            self.policy
                .can_create_realm(&identity, master.realm())
                .await,
            "insufficient permissions",
        )?;
        ensure_policy(
            self.policy.can_update_realm(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;
        self.archive.ensure_exportable(scope.id()).await?;

        let header = BundleHeader::new(
            BundleRealm {
                id: scope.id().into(),
                name: scope.name().to_owned(),
            },
            self.archive.schema_version().await?,
            Utc::now(),
        );
        let key = derive_key(passphrase, None).await?;

        let details = json!({ "schema": header.schema, "version": header.version });
        let succeeded = transfer_event(
            scope.id(),
            &identity,
            SecurityEventType::RealmExported,
            EventStatus::Success,
            details.clone(),
        );
        let failed = transfer_event(
            scope.id(),
            &identity,
            SecurityEventType::RealmExported,
            EventStatus::Failure,
            details,
        );

        let (out, chunks) = mpsc::channel(EXPORT_BUFFERED_CHUNKS);
        let archive = Arc::clone(&self.archive);
        let events = Arc::clone(&self.security_event_repository);
        let realm_id = scope.id();
        tokio::spawn(async move {
            let outcome = export_bundle(archive.as_ref(), realm_id, key, header, out).await;
            let event = if outcome { succeeded } else { failed };
            if let Err(error) = events.store_event(event).await {
                warn!(%error, "the realm export could not be recorded");
            }
        });

        Ok(RealmBundleStream::new(chunks))
    }

    #[instrument(skip(self, identity, passphrase, body))]
    async fn import_realm<S, B, E>(
        &self,
        identity: Identity,
        passphrase: BundlePassphrase,
        body: S,
    ) -> Result<ImportReport, CoreError>
    where
        S: Stream<Item = Result<B, E>> + Send + Unpin,
        B: AsRef<[u8]> + Send,
        E: Display + Send,
    {
        let master = RealmScope::resolve(self.realm_repository.as_ref(), MASTER_REALM).await?;
        ensure_policy(
            self.policy
                .can_create_realm(&identity, master.realm())
                .await,
            "insufficient permissions",
        )?;

        let mut reader = BundleReader::open(passphrase, body).await?;
        let header = reader.header().clone();
        header.check(self.archive.schema_version().await?)?;
        let mode = self.check_import_target(&identity, &header).await?;

        // A create-only caller cannot re-import to repair a missing management
        // client, so a new realm gets it before any row is written.
        if mode == ImportMode::Create {
            self.ensure_mirror_client(&identity, &master, &header.realm.name, mode)
                .await?;
        }

        let mut report = self.archive.import_rows(&header, mode, &mut reader).await?;

        if mode == ImportMode::Update
            && let Err(error) = self
                .ensure_mirror_client(&identity, &master, &header.realm.name, mode)
                .await
        {
            warn!(%error, "the imported realm has no management client in master yet");
            report.unfinished_steps.push(MIRROR_CLIENT_STEP.to_owned());
        }
        if let Err(error) = self
            .record(
                header.realm.id.into(),
                &identity,
                SecurityEventType::RealmImported,
                json!({
                    "schema": header.schema,
                    "version": header.version,
                    "exported_at": header.exported_at,
                    "created": report.created,
                }),
            )
            .await
        {
            warn!(%error, "the realm import could not be recorded");
            report.unfinished_steps.push(AUDIT_STEP.to_owned());
        }

        Ok(report)
    }
}
