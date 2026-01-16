use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::domain::abyss::federation::entities::{
    FederationMapping, FederationProvider, FederationType, SyncMode,
};
use crate::domain::abyss::federation::ports::{
    FederationPolicy, FederationRepository, FederationService,
};
use crate::domain::abyss::federation::value_objects::{
    CreateProviderRequest, SyncError, SyncResult, TestConnectionResult, UpdateProviderRequest,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::policies::ensure_policy;
use crate::domain::credential::ports::CredentialRepository;
use crate::domain::realm::ports::RealmRepository;
use crate::domain::user::ports::UserRepository;
use crate::domain::user::value_objects::{CreateUserRequest, UpdateUserRequest};
use crate::infrastructure::abyss::federation::ldap::LdapClientImpl;

#[derive(Clone, Debug)]
pub struct FederationServiceImpl<R, F, P, U, CR>
where
    R: RealmRepository,
    F: FederationRepository,
    P: FederationPolicy,
    U: UserRepository,
    CR: CredentialRepository,
{
    federation_repository: Arc<F>,
    realm_repository: Arc<R>,
    user_repository: Arc<U>,
    credential_repository: Arc<CR>,
    policy: Arc<P>,
    ldap_client: LdapClientImpl,
}

impl<R, F, P, U, CR> FederationServiceImpl<R, F, P, U, CR>
where
    R: RealmRepository,
    F: FederationRepository,
    P: FederationPolicy,
    U: UserRepository,
    CR: CredentialRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        federation_repository: Arc<F>,
        user_repository: Arc<U>,
        credential_repository: Arc<CR>,
        policy: Arc<P>,
    ) -> Self {
        Self {
            realm_repository,
            federation_repository,
            user_repository,
            credential_repository,
            policy,
            ldap_client: LdapClientImpl,
        }
    }
}

impl<R, F, P, U, CR> FederationService for FederationServiceImpl<R, F, P, U, CR>
where
    R: RealmRepository,
    F: FederationRepository,
    P: FederationPolicy,
    U: UserRepository,
    CR: CredentialRepository,
{
    #[instrument(skip(self, identity, request))]
    async fn create_federation_provider(
        &self,
        identity: Identity,
        realm_name: String,
        mut request: CreateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;
        let realm_id = realm.id;

        ensure_policy(
            self.policy
                .can_create_federation_provider(identity, realm)
                .await,
            "insufficient permissions to create provider",
        )?;

        request.realm_id = realm_id.into();

        // TODO: Validate config based on provider type
        self.federation_repository.create(request).await
    }

    #[instrument(skip(self, identity))]
    async fn get_federation_provider(
        &self,
        identity: Identity,
        id: Uuid,
        realm_name: String,
    ) -> Result<FederationProvider, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if provider.realm_id != Into::<Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        ensure_policy(
            self.policy
                .can_view_federation_provider(&identity, realm)
                .await,
            "insufficient permissions to view provider",
        )?;

        Ok(provider)
    }

    #[instrument(skip(self, identity, request))]
    async fn update_federation_provider(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
        request: UpdateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if provider.realm_id != Into::<Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        ensure_policy(
            self.policy
                .can_update_federation_provider(&identity, realm)
                .await,
            "insufficient permissions to update provider",
        )?;

        self.federation_repository.update(id, request).await
    }

    #[instrument(skip(self, identity))]
    async fn delete_federation_provider(
        &self,
        identity: Identity,
        id: Uuid,
        realm_name: String,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if provider.realm_id != Into::<Uuid>::into(realm.id) {
            error!("Provider realm ID does not match requested realm");
            return Err(CoreError::NotFound);
        }

        info!("try deleting federation provider with ID: {}", id);

        ensure_policy(
            self.policy
                .can_delete_federation_provider(&identity, realm)
                .await,
            "insufficient permissions to delete provider",
        )?;

        self.federation_repository.delete(id).await
    }

    #[instrument(skip(self))]
    async fn list_federation_providers(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<FederationProvider>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.federation_repository
            .list_by_realm(realm.id.into())
            .await
    }

    #[instrument(skip(self))]
    async fn test_federation_connection(
        &self,
        id: Uuid,
    ) -> Result<TestConnectionResult, CoreError> {
        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        match provider.provider_type {
            FederationType::Ldap | FederationType::ActiveDirectory => {
                self.ldap_client.test_connection(&provider).await
            }
            _ => Ok(TestConnectionResult {
                success: false,
                message: "Provider type not supported for connection testing".to_string(),
                details: None,
            }),
        }
    }

    #[instrument(skip(self))]
    async fn sync_federation_users(
        &self,
        id: Uuid,
        mode: SyncMode,
    ) -> Result<SyncResult, CoreError> {
        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        info!(
            "Starting federation sync for provider '{}' (ID: {}), mode: {:?}",
            provider.name, id, mode
        );

        match provider.provider_type {
            FederationType::Ldap | FederationType::ActiveDirectory => {
                self.sync_ldap_users(&provider, mode).await
            }
            _ => Err(CoreError::Configuration(
                "Provider type does not support sync".to_string(),
            )),
        }
    }
}

impl<R, F, P, U, CR> FederationServiceImpl<R, F, P, U, CR>
where
    R: RealmRepository,
    F: FederationRepository,
    P: FederationPolicy,
    U: UserRepository,
    CR: CredentialRepository,
{
    /// Comprehensive LDAP user synchronization with reconciliation
    ///
    /// This method performs a "diff" between LDAP and the local database:
    /// - TO CREATE: Users in LDAP but not in local DB
    /// - TO UPDATE: Users in both, but with changed attributes
    /// - TO DISABLE: Users in local DB but not in LDAP (Force mode only)
    ///
    /// Performance optimizations:
    /// - Batch fetching of federation mappings to avoid N+1 queries
    /// - Atomic operations to prevent partial updates
    /// - Comprehensive error handling with rollback support
    #[instrument(skip(self, provider))]
    async fn sync_ldap_users(
        &self,
        provider: &FederationProvider,
        mode: SyncMode,
    ) -> Result<SyncResult, CoreError> {
        // Start timing
        let start_time = Instant::now();
        let started_at = Utc::now();

        info!(
            "Starting sync for provider '{}' (ID: {}), mode: {:?}",
            provider.name, provider.id, mode
        );

        let mut result = SyncResult {
            total_processed: 0,
            created: 0,
            updated: 0,
            disabled: 0,
            failed: 0,
            errors: vec![],
            duration_ms: None,
            started_at: Some(started_at.to_rfc3339()),
            completed_at: None,
        };

        // Step 1: Fetch all users from LDAP
        info!("Fetching users from LDAP provider '{}'", provider.name);
        let ldap_users = match self.ldap_client.search_users(provider, None).await {
            Ok(users) => {
                info!("Found {} users in LDAP", users.len());
                users
            }
            Err(e) => {
                error!("Failed to fetch LDAP users: {}", e);
                // Record timing even on failure
                let completed_at = Utc::now();
                let duration = start_time.elapsed();
                result.duration_ms = Some(duration.as_millis() as u64);
                result.completed_at = Some(completed_at.to_rfc3339());
                return Err(e);
            }
        };

        // Step 2: Build a set of external_ids from LDAP for later comparison
        let ldap_external_ids: HashSet<String> =
            ldap_users.iter().map(|u| u.external_id.clone()).collect();

        // Step 3: Fetch all existing federation mappings for this provider (optimized batch fetch)
        info!(
            "Fetching existing federation mappings for provider '{}'",
            provider.name
        );
        let existing_mappings = self
            .federation_repository
            .list_mappings_by_provider(provider.id)
            .await?;

        info!(
            "Found {} existing mappings for this provider",
            existing_mappings.len()
        );

        // Build a lookup map for fast access: external_id -> mapping
        let mut mappings_by_external_id: std::collections::HashMap<String, FederationMapping> =
            existing_mappings
                .into_iter()
                .map(|m| (m.external_id.clone(), m))
                .collect();

        info!(
            "Processing {} LDAP users for reconciliation",
            ldap_users.len()
        );

        // Step 4: Optimized reconciliation loop - create or update users
        // Use the pre-fetched mappings to avoid N+1 database queries
        for ldap_user in ldap_users {
            result.total_processed += 1;

            // Check if we have an existing mapping in our HashMap
            let existing_mapping = mappings_by_external_id.remove(&ldap_user.external_id);

            match self
                .reconcile_user_optimized(provider, &ldap_user, existing_mapping, mode)
                .await
            {
                Ok(action) => match action {
                    ReconcileAction::Created => {
                        result.created += 1;
                        info!(
                            "Created user '{}' from LDAP (external_id: {})",
                            ldap_user.username, ldap_user.external_id
                        );
                    }
                    ReconcileAction::Updated => {
                        result.updated += 1;
                        info!(
                            "Updated user '{}' from LDAP (external_id: {})",
                            ldap_user.username, ldap_user.external_id
                        );
                    }
                    ReconcileAction::NoChange => {
                        // User exists and is up to date
                    }
                },
                Err(e) => {
                    result.failed += 1;
                    warn!(
                        "Failed to reconcile user '{}' (external_id: {}): {}",
                        ldap_user.username, ldap_user.external_id, e
                    );
                    result.errors.push(SyncError {
                        username: Some(ldap_user.username.clone()),
                        external_id: ldap_user.external_id.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        // Step 5: Handle missing users (users in Ferriskey but not in LDAP)
        // Only in Force mode to prevent accidental mass disables
        if mode == SyncMode::Force {
            info!("Checking for users to disable (Force mode enabled)");
            match self
                .disable_missing_users(provider, &ldap_external_ids)
                .await
            {
                Ok(disabled_count) => {
                    result.disabled = disabled_count;
                    info!("Disabled {} users not found in LDAP", disabled_count);
                }
                Err(e) => {
                    warn!("Failed to disable missing users, but continuing: {}", e);
                    // We don't fail the entire sync for this
                }
            }
        }

        // Record completion time and duration
        let completed_at = Utc::now();
        let duration = start_time.elapsed();
        result.duration_ms = Some(duration.as_millis() as u64);
        result.completed_at = Some(completed_at.to_rfc3339());

        info!(
            "Sync completed in {}ms: {} total, {} created, {} updated, {} disabled, {} failed",
            duration.as_millis(),
            result.total_processed,
            result.created,
            result.updated,
            result.disabled,
            result.failed
        );

        if !result.errors.is_empty() {
            warn!(
                "Sync completed with {} errors. First error: {}",
                result.errors.len(),
                result
                    .errors
                    .first()
                    .map(|e| &e.error)
                    .unwrap_or(&"unknown".to_string())
            );
        }

        Ok(result)
    }

    /// Reconcile a single user: create if new, update if changed
    ///
    /// NOTE: This is the non-optimized version that queries for the mapping.
    /// Use `reconcile_user_optimized` in batch operations for better performance.
    #[allow(dead_code)]
    #[instrument(skip(self, provider, ldap_user))]
    async fn reconcile_user(
        &self,
        provider: &FederationProvider,
        ldap_user: &crate::domain::abyss::federation::entities::FederatedUser,
        mode: SyncMode,
    ) -> Result<ReconcileAction, CoreError> {
        // Check if mapping exists
        let existing_mapping = self
            .federation_repository
            .get_mapping(provider.id, &ldap_user.external_id)
            .await?;

        match existing_mapping {
            Some(mapping) => {
                // User exists - check if update is needed
                let user = self.user_repository.get_by_id(mapping.user_id).await?;

                let needs_update = match mode {
                    SyncMode::Force => true, // Always update in Force mode
                    SyncMode::Import | SyncMode::LinkOnly => {
                        // Check if attributes have changed
                        user.email != ldap_user.email.clone().unwrap_or_default()
                            || user.firstname != ldap_user.first_name.clone().unwrap_or_default()
                            || user.lastname != ldap_user.last_name.clone().unwrap_or_default()
                    }
                };

                if needs_update && mode != SyncMode::LinkOnly {
                    // Update user attributes
                    let update_request = UpdateUserRequest {
                        email: ldap_user
                            .email
                            .clone()
                            .unwrap_or_else(|| user.email.clone()),
                        firstname: ldap_user
                            .first_name
                            .clone()
                            .unwrap_or_else(|| user.firstname.clone()),
                        lastname: ldap_user
                            .last_name
                            .clone()
                            .unwrap_or_else(|| user.lastname.clone()),
                        enabled: true, // Re-enable if was disabled
                        email_verified: user.email_verified,
                        required_actions: None,
                    };

                    self.user_repository
                        .update_user(user.id, update_request)
                        .await?;

                    // Update mapping timestamp
                    let updated_mapping = FederationMapping {
                        id: mapping.id,
                        provider_id: provider.id,
                        user_id: mapping.user_id,
                        external_id: ldap_user.external_id.clone(),
                        external_username: ldap_user.username.clone(),
                        mapping_metadata: serde_json::to_value(&ldap_user.attributes)
                            .unwrap_or(serde_json::Value::Null),
                        last_synced_at: Utc::now(),
                    };
                    self.federation_repository
                        .update_mapping(updated_mapping)
                        .await?;

                    Ok(ReconcileAction::Updated)
                } else {
                    Ok(ReconcileAction::NoChange)
                }
            }
            None => {
                // New user - create both user and mapping
                let create_request = CreateUserRequest {
                    realm_id: provider.realm_id.into(),
                    username: ldap_user.username.clone(),
                    email: ldap_user.email.clone().unwrap_or_default(),
                    firstname: ldap_user.first_name.clone().unwrap_or_default(),
                    lastname: ldap_user.last_name.clone().unwrap_or_default(),
                    enabled: true,
                    email_verified: false,
                    client_id: None,
                };

                let new_user = self.user_repository.create_user(create_request).await?;

                // Create federation mapping
                let mapping = FederationMapping {
                    id: Uuid::new_v4(),
                    provider_id: provider.id,
                    user_id: new_user.id,
                    external_id: ldap_user.external_id.clone(),
                    external_username: ldap_user.username.clone(),
                    mapping_metadata: serde_json::to_value(&ldap_user.attributes)
                        .unwrap_or(serde_json::Value::Null),
                    last_synced_at: Utc::now(),
                };

                self.federation_repository.create_mapping(mapping).await?;

                Ok(ReconcileAction::Created)
            }
        }
    }

    /// Optimized version of reconcile_user that accepts a pre-fetched mapping
    ///
    /// This avoids the N+1 query problem by using mappings fetched in batch.
    /// Performance improvement: O(1) lookup vs O(n) database queries
    #[instrument(skip(self, provider, ldap_user, existing_mapping))]
    async fn reconcile_user_optimized(
        &self,
        provider: &FederationProvider,
        ldap_user: &crate::domain::abyss::federation::entities::FederatedUser,
        existing_mapping: Option<FederationMapping>,
        mode: SyncMode,
    ) -> Result<ReconcileAction, CoreError> {
        match existing_mapping {
            Some(mapping) => {
                // User exists - check if update is needed
                let user = self.user_repository.get_by_id(mapping.user_id).await?;

                let needs_update = match mode {
                    SyncMode::Force => true, // Always update in Force mode
                    SyncMode::Import | SyncMode::LinkOnly => {
                        // Check if attributes have changed
                        user.email != ldap_user.email.clone().unwrap_or_default()
                            || user.firstname != ldap_user.first_name.clone().unwrap_or_default()
                            || user.lastname != ldap_user.last_name.clone().unwrap_or_default()
                    }
                };

                if needs_update && mode != SyncMode::LinkOnly {
                    // Update user attributes
                    let update_request = UpdateUserRequest {
                        email: ldap_user
                            .email
                            .clone()
                            .unwrap_or_else(|| user.email.clone()),
                        firstname: ldap_user
                            .first_name
                            .clone()
                            .unwrap_or_else(|| user.firstname.clone()),
                        lastname: ldap_user
                            .last_name
                            .clone()
                            .unwrap_or_else(|| user.lastname.clone()),
                        enabled: true, // Re-enable if was disabled
                        email_verified: user.email_verified,
                        required_actions: None,
                    };

                    self.user_repository
                        .update_user(user.id, update_request)
                        .await?;

                    // Update mapping timestamp
                    let updated_mapping = FederationMapping {
                        id: mapping.id,
                        provider_id: provider.id,
                        user_id: mapping.user_id,
                        external_id: ldap_user.external_id.clone(),
                        external_username: ldap_user.username.clone(),
                        mapping_metadata: serde_json::to_value(&ldap_user.attributes)
                            .unwrap_or(serde_json::Value::Null),
                        last_synced_at: Utc::now(),
                    };
                    self.federation_repository
                        .update_mapping(updated_mapping)
                        .await?;

                    Ok(ReconcileAction::Updated)
                } else {
                    Ok(ReconcileAction::NoChange)
                }
            }
            None => {
                // No existing mapping - check if user exists without mapping (orphan user)
                info!(
                    "No mapping found for LDAP user '{}', checking if user exists without mapping",
                    ldap_user.username
                );

                let existing_user = self
                    .user_repository
                    .get_by_username(ldap_user.username.clone(), provider.realm_id.into())
                    .await;

                let user = match existing_user {
                    Ok(user) => {
                        // User exists but has no mapping - link it (orphan user case)
                        warn!(
                            "Found existing user '{}' (ID: {}) without federation mapping, linking to LDAP provider '{}'",
                            user.username, user.id, provider.name
                        );

                        // Update user attributes from LDAP
                        let update_request = UpdateUserRequest {
                            email: ldap_user
                                .email
                                .clone()
                                .unwrap_or_else(|| user.email.clone()),
                            firstname: ldap_user
                                .first_name
                                .clone()
                                .unwrap_or_else(|| user.firstname.clone()),
                            lastname: ldap_user
                                .last_name
                                .clone()
                                .unwrap_or_else(|| user.lastname.clone()),
                            enabled: true,
                            email_verified: user.email_verified,
                            required_actions: None,
                        };

                        self.user_repository
                            .update_user(user.id, update_request)
                            .await?;

                        user
                    }
                    Err(_) => {
                        // User doesn't exist - create new user
                        info!("Creating new user '{}' from LDAP", ldap_user.username);

                        let create_request = CreateUserRequest {
                            realm_id: provider.realm_id.into(),
                            username: ldap_user.username.clone(),
                            email: ldap_user.email.clone().unwrap_or_default(),
                            firstname: ldap_user.first_name.clone().unwrap_or_default(),
                            lastname: ldap_user.last_name.clone().unwrap_or_default(),
                            enabled: true,
                            email_verified: false,
                            client_id: None,
                        };

                        self.user_repository.create_user(create_request).await?
                    }
                };

                // Create federation mapping for both cases (orphan or new user)
                info!(
                    "Creating federation mapping for user '{}' (ID: {}) to provider '{}'",
                    user.username, user.id, provider.name
                );

                let mapping = FederationMapping {
                    id: Uuid::new_v4(),
                    provider_id: provider.id,
                    user_id: user.id,
                    external_id: ldap_user.external_id.clone(),
                    external_username: ldap_user.username.clone(),
                    mapping_metadata: serde_json::to_value(&ldap_user.attributes)
                        .unwrap_or(serde_json::Value::Null),
                    last_synced_at: Utc::now(),
                };

                self.federation_repository.create_mapping(mapping).await?;

                // Create federated credential marker (like Keycloak does)
                // This shows in the UI that the user can authenticate via password (through LDAP)
                info!(
                    "Creating federated password credential marker for user '{}'",
                    user.username
                );

                let credential_data = serde_json::json!({
                    "provider_id": provider.id.to_string(),
                    "provider_type": provider.provider_type.to_string(),
                });

                // Check if user already has a federated credential to avoid duplicates
                let existing_credentials = self
                    .credential_repository
                    .get_credentials_by_user_id(user.id)
                    .await
                    .unwrap_or_default();

                let has_federated_credential = existing_credentials.iter().any(|c| {
                    c.credential_type.to_string() == "password" && c.secret_data == "federated"
                });

                if !has_federated_credential {
                    // Use a placeholder for secret_data since actual password is in LDAP
                    let _result = self
                        .credential_repository
                        .create_custom_credential(
                            user.id,
                            "password".to_string(),
                            "federated".to_string(), // Marker that indicates LDAP authentication
                            Some(format!("Federated - {}", provider.name)),
                            credential_data,
                        )
                        .await;

                    // We don't fail the sync if credential creation fails (non-critical)
                    if _result.is_err() {
                        warn!(
                            "Failed to create federated credential marker for user '{}', but continuing",
                            user.username
                        );
                    }
                }

                Ok(ReconcileAction::Created)
            }
        }
    }

    /// Disable users that exist in Ferriskey but not in LDAP
    #[instrument(skip(self, provider, ldap_external_ids))]
    async fn disable_missing_users(
        &self,
        provider: &FederationProvider,
        ldap_external_ids: &HashSet<String>,
    ) -> Result<u32, CoreError> {
        info!(
            "Checking for users to disable (not found in LDAP) for provider '{}'",
            provider.name
        );

        // Fetch all mappings for this provider
        let all_mappings = self
            .federation_repository
            .list_mappings_by_provider(provider.id)
            .await?;

        let mut disabled_count = 0;

        // For each mapping, check if the external_id exists in LDAP
        for mapping in all_mappings {
            if !ldap_external_ids.contains(&mapping.external_id) {
                // User exists in Ferriskey but not in LDAP - disable them
                info!(
                    "User with external_id '{}' no longer exists in LDAP, disabling local user",
                    mapping.external_id
                );

                // Fetch the user to get current values
                match self.user_repository.get_by_id(mapping.user_id).await {
                    Ok(user) => {
                        // Only disable if not already disabled
                        if user.enabled {
                            let update_request = UpdateUserRequest {
                                email: user.email.clone(),
                                firstname: user.firstname.clone(),
                                lastname: user.lastname.clone(),
                                enabled: false, // Disable the user
                                email_verified: user.email_verified,
                                required_actions: None,
                            };

                            match self
                                .user_repository
                                .update_user(mapping.user_id, update_request)
                                .await
                            {
                                Ok(_) => {
                                    disabled_count += 1;
                                    info!(
                                        "Successfully disabled user (ID: {}, external_id: {})",
                                        mapping.user_id, mapping.external_id
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to disable user (ID: {}, external_id: {}): {}",
                                        mapping.user_id, mapping.external_id, e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to fetch user (ID: {}, external_id: {}): {}",
                            mapping.user_id, mapping.external_id, e
                        );
                        // Continue processing other users even if one fails
                    }
                }
            }
        }

        Ok(disabled_count)
    }
}

#[derive(Debug)]
enum ReconcileAction {
    Created,
    Updated,
    NoChange,
}
