# Implémentation des services et repositories aegis (client-scopes)

Implémente les services CRUD et les repositories d'infrastructure du module aegis dans `core/src/domain/aegis/` et `core/src/infrastructure/aegis/`. Le module gère 3 ressources indépendantes : **client scopes**, **protocol mappers**, et **client scope mappings**. Chaque ressource a son propre fichier de service.

## Périmètre

- Services domain + policies dans `core/src/domain/aegis/`
- SeaORM entities dans `core/src/entity/`
- Repositories d'infrastructure dans `core/src/infrastructure/aegis/`
- Ajoute un trait `ClientScopeService` dans `ports.rs`
- Ne touche PAS aux entities, value_objects ni aux repository traits existants dans le domain
- Ne câble PAS l'`ApplicationService` (on le fera plus tard)
- Vérifie que `cargo check -p ferriskey-core` compile

---

## 1. Ajouter les permissions dans `core/src/domain/role/entities/permission.rs`

Ajoute ces 3 variantes à l'enum `Permissions` (après `ViewWebhooks = 1 << 21`) :

```rust
ManageClientScopes = 1 << 22, // 1 << 22
QueryClientScopes = 1 << 23,  // 1 << 23
ViewClientScopes = 1 << 24,   // 1 << 24
```

Mets à jour **toutes** les méthodes de `Permissions` qui listent les variantes :

- `from_bitfield()` — ajoute les 3 dans le tableau `all_permissions`
- `name()` — ajoute les 3 match arms : `"manage_client_scopes"`, `"query_client_scopes"`, `"view_client_scopes"`
- `from_name()` — ajoute les 3 match arms inverses

---

## 2. Créer `core/src/domain/aegis/policies.rs`

Suis **exactement** le pattern de `core/src/domain/webhook/policies.rs` :

```rust
use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, Policy},
    },
    realm::entities::Realm,
    role::entities::permission::Permissions,
    user::ports::{UserRepository, UserRoleRepository},
    aegis::ports::ClientScopePolicy,
};

impl<U, C, UR> ClientScopePolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    // can_view_scope → [ManageRealm, ManageClientScopes, ViewClientScopes]
    // can_create_scope → [ManageRealm, ManageClientScopes]
    // can_update_scope → [ManageRealm, ManageClientScopes]
    // can_delete_scope → [ManageRealm, ManageClientScopes]
}
```

Chaque méthode suit ce pattern exact (copié de webhook) :

```rust
async fn can_view_scope(
    &self,
    identity: &Identity,
    target_realm: &Realm,
) -> Result<bool, CoreError> {
    let user = self.get_user_from_identity(identity).await?;

    let permissions = self
        .get_permission_for_target_realm(&user, target_realm)
        .await?;

    let has_permission = Permissions::has_one_of_permissions(
        &permissions.iter().cloned().collect::<Vec<Permissions>>(),
        &[
            Permissions::ManageRealm,
            Permissions::ManageClientScopes,
            Permissions::ViewClientScopes,
        ],
    );

    Ok(has_permission)
}
```

---

## 3. Ajouter le trait `ClientScopeService` dans `core/src/domain/aegis/ports.rs`

Ajoute ce trait **à la fin** du fichier `ports.rs` existant (après `ClientScopePolicy`). Ne crée PAS de traits séparés pour chaque sous-ressource — un seul trait regroupe toutes les opérations.

```rust
pub trait ClientScopeService: Send + Sync {
    // --- Client Scopes ---
    fn create_client_scope(
        &self,
        identity: Identity,
        input: CreateClientScopeInput,
    ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

    fn get_client_scope(
        &self,
        identity: Identity,
        input: GetClientScopeInput,
    ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

    fn get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientScopesInput,
    ) -> impl Future<Output = Result<Vec<ClientScope>, CoreError>> + Send;

    fn update_client_scope(
        &self,
        identity: Identity,
        input: UpdateClientScopeInput,
    ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

    fn delete_client_scope(
        &self,
        identity: Identity,
        input: DeleteClientScopeInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    // --- Protocol Mappers ---
    fn create_protocol_mapper(
        &self,
        identity: Identity,
        input: CreateProtocolMapperInput,
    ) -> impl Future<Output = Result<ProtocolMapper, CoreError>> + Send;

    fn update_protocol_mapper(
        &self,
        identity: Identity,
        input: UpdateProtocolMapperInput,
    ) -> impl Future<Output = Result<ProtocolMapper, CoreError>> + Send;

    fn delete_protocol_mapper(
        &self,
        identity: Identity,
        input: DeleteProtocolMapperInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    // --- Client Scope Mappings ---
    fn assign_scope_to_client(
        &self,
        identity: Identity,
        input: AssignClientScopeInput,
    ) -> impl Future<Output = Result<ClientScopeMapping, CoreError>> + Send;

    fn unassign_scope_from_client(
        &self,
        identity: Identity,
        input: UnassignClientScopeInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
```

Ajoute les imports nécessaires depuis `super::value_objects` et `super::entities`.

---

## 4. Créer le dossier `core/src/domain/aegis/services/`

### Structure

```
core/src/domain/aegis/services/
├── mod.rs
├── client_scope_service.rs
├── protocol_mapper_service.rs
└── scope_mapping_service.rs
```

### `services/mod.rs`

```rust
mod client_scope_service;
mod protocol_mapper_service;
mod scope_mapping_service;
```

Ce fichier définit aussi la **struct principale** `ClientScopeServiceImpl` et son constructeur `new()`, puis implémente `ClientScopeService` en déléguant aux méthodes privées définies dans chaque sous-fichier.

### Architecture de la struct

Suis le pattern exact de `core/src/domain/webhook/services.rs` (struct avec `Arc`, generics, `FerriskeyPolicy`) :

```rust
use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
    aegis::{
        entities::{ClientScope, ClientScopeMapping, ProtocolMapper},
        ports::{
            ClientScopePolicy, ClientScopeMappingRepository, ClientScopeRepository,
            ClientScopeService, ProtocolMapperRepository,
        },
        value_objects::*,
    },
};

#[derive(Clone, Debug)]
pub struct ClientScopeServiceImpl<R, U, C, UR, CS, PM, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
    CSM: ClientScopeMappingRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_scope_repository: Arc<CS>,
    pub(crate) protocol_mapper_repository: Arc<PM>,
    pub(crate) scope_mapping_repository: Arc<CSM>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}
```

Constructeur `new(realm_repository, client_scope_repository, protocol_mapper_repository, scope_mapping_repository, policy)`.

### Implémentation du trait `ClientScopeService`

L'impl du trait est dans `services/mod.rs`. Chaque méthode appelle une méthode privée `async fn` définie dans le fichier de service correspondant via un `impl` block séparé sur `ClientScopeServiceImpl`.

---

### `services/client_scope_service.rs`

Implémente les méthodes privées pour le CRUD des client scopes. **Chaque méthode** suit ce flow exact (pattern webhook) :

1. **Résolution du realm** : `self.realm_repository.get_by_name(input.realm_name).await.map_err(|_| CoreError::InvalidRealm)?.ok_or(CoreError::InvalidRealm)?`
2. **Vérification de la policy** : `ensure_policy(self.policy.can_xxx_scope(&identity, &realm).await, "insufficient permissions")?`
3. **Opération repository**
4. **Retour du résultat**

Méthodes à implémenter :

```rust
impl<R, U, C, UR, CS, PM, CSM> ClientScopeServiceImpl<R, U, C, UR, CS, PM, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CS: ClientScopeRepository,
    PM: ProtocolMapperRepository,
    CSM: ClientScopeMappingRepository,
{
    pub(super) async fn handle_create_client_scope(
        &self,
        identity: Identity,
        input: CreateClientScopeInput,
    ) -> Result<ClientScope, CoreError> { ... }

    pub(super) async fn handle_get_client_scope(
        &self,
        identity: Identity,
        input: GetClientScopeInput,
    ) -> Result<ClientScope, CoreError> { ... }

    pub(super) async fn handle_get_client_scopes(
        &self,
        identity: Identity,
        input: GetClientScopesInput,
    ) -> Result<Vec<ClientScope>, CoreError> { ... }

    pub(super) async fn handle_update_client_scope(
        &self,
        identity: Identity,
        input: UpdateClientScopeInput,
    ) -> Result<ClientScope, CoreError> { ... }

    pub(super) async fn handle_delete_client_scope(
        &self,
        identity: Identity,
        input: DeleteClientScopeInput,
    ) -> Result<(), CoreError> { ... }
}
```

Pour `handle_create_client_scope` :

- Résoudre le realm
- Policy `can_create_scope`
- Appeler `self.client_scope_repository.create(CreateClientScopeRequest { realm_id: realm.id, name: input.name, description: input.description, protocol: input.protocol, is_default: input.is_default })`

Pour `handle_get_client_scope` :

- Résoudre le realm
- Policy `can_view_scope`
- Appeler `self.client_scope_repository.get_by_id(input.scope_id)` puis `.ok_or(CoreError::NotFound)?`

Pour `handle_get_client_scopes` :

- Résoudre le realm
- Policy `can_view_scope`
- Appeler `self.client_scope_repository.find_by_realm_id(realm.id)`

Pour `handle_update_client_scope` :

- Résoudre le realm
- Policy `can_update_scope`
- Appeler `self.client_scope_repository.update_by_id(input.scope_id, input.payload)`

Pour `handle_delete_client_scope` :

- Résoudre le realm
- Policy `can_delete_scope`
- Appeler `self.client_scope_repository.delete_by_id(input.scope_id)`

---

### `services/protocol_mapper_service.rs`

Même pattern. Méthodes privées :

```rust
impl<...> ClientScopeServiceImpl<...> {
    pub(super) async fn handle_create_protocol_mapper(...) -> Result<ProtocolMapper, CoreError> { ... }
    pub(super) async fn handle_update_protocol_mapper(...) -> Result<ProtocolMapper, CoreError> { ... }
    pub(super) async fn handle_delete_protocol_mapper(...) -> Result<(), CoreError> { ... }
}
```

Pour `handle_create_protocol_mapper` :

- Résoudre le realm
- Policy `can_update_scope` (modifier les mappers d'un scope = modifier le scope)
- Vérifier que le scope existe : `self.client_scope_repository.get_by_id(input.scope_id).await?.ok_or(CoreError::NotFound)?`
- Appeler `self.protocol_mapper_repository.create(CreateProtocolMapperRequest { client_scope_id: input.scope_id, name: input.name, mapper_type: input.mapper_type, config: input.config })`

Pour `handle_update_protocol_mapper` :

- Résoudre le realm
- Policy `can_update_scope`
- Vérifier que le scope existe
- Appeler `self.protocol_mapper_repository.update_by_id(input.mapper_id, input.payload)`

Pour `handle_delete_protocol_mapper` :

- Résoudre le realm
- Policy `can_update_scope`
- Vérifier que le scope existe
- Appeler `self.protocol_mapper_repository.delete_by_id(input.mapper_id)`

---

### `services/scope_mapping_service.rs`

Méthodes privées :

```rust
impl<...> ClientScopeServiceImpl<...> {
    pub(super) async fn handle_assign_scope_to_client(...) -> Result<ClientScopeMapping, CoreError> { ... }
    pub(super) async fn handle_unassign_scope_from_client(...) -> Result<(), CoreError> { ... }
}
```

Pour `handle_assign_scope_to_client` :

- Résoudre le realm
- Policy `can_update_scope`
- Vérifier que le scope existe
- Appeler `self.scope_mapping_repository.assign_scope_to_client(input.client_id, input.scope_id, input.is_default, input.is_optional)`

Pour `handle_unassign_scope_from_client` :

- Résoudre le realm
- Policy `can_update_scope`
- Appeler `self.scope_mapping_repository.remove_scope_from_client(input.client_id, input.scope_id)`

---

## 5. Mettre à jour `core/src/domain/aegis/mod.rs`

```rust
pub mod entities;
pub mod policies;
pub mod ports;
pub mod services;
pub mod value_objects;
```

---

## 6. SeaORM Entities dans `core/src/entity/`

Les tables aegis n'ont pas encore d'entities SeaORM auto-générées (on ne peut pas lancer `sea-orm-cli generate` sans DB live). Crée-les **manuellement** en suivant exactement le pattern des entities existantes (ex: `core/src/entity/webhooks.rs`, `core/src/entity/security_events.rs`).

Tous les fichiers commencent par `//! \`SeaORM\` Entity, @generated by sea-orm-codegen 1.1.14`et utilisent`use sea_orm::entity::prelude::\*;`.

### `core/src/entity/client_scopes.rs`

```rust
//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.14

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "client_scopes"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub protocol: String,
    pub is_default: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    RealmId,
    Name,
    Description,
    Protocol,
    IsDefault,
    CreatedAt,
    UpdatedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = Uuid;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Realms,
    ClientScopeAttributes,
    ClientScopeProtocolMappers,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Uuid.def(),
            Self::RealmId => ColumnType::Uuid.def(),
            Self::Name => ColumnType::String(StringLen::N(255u32)).def(),
            Self::Description => ColumnType::Text.def().null(),
            Self::Protocol => ColumnType::String(StringLen::N(255u32)).def(),
            Self::IsDefault => ColumnType::Boolean.def(),
            Self::CreatedAt => ColumnType::DateTime.def(),
            Self::UpdatedAt => ColumnType::DateTime.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Realms => Entity::belongs_to(super::realms::Entity)
                .from(Column::RealmId)
                .to(super::realms::Column::Id)
                .into(),
            Self::ClientScopeAttributes => {
                Entity::has_many(super::client_scope_attributes::Entity).into()
            }
            Self::ClientScopeProtocolMappers => {
                Entity::has_many(super::client_scope_protocol_mappers::Entity).into()
            }
        }
    }
}

impl Related<super::realms::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Realms.def()
    }
}

impl Related<super::client_scope_attributes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClientScopeAttributes.def()
    }
}

impl Related<super::client_scope_protocol_mappers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClientScopeProtocolMappers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### `core/src/entity/client_scope_attributes.rs`

```rust
//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.14

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "client_scope_attributes"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: Uuid,
    pub scope_id: Uuid,
    pub name: String,
    pub value: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    ScopeId,
    Name,
    Value,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = Uuid;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    ClientScopes,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Uuid.def(),
            Self::ScopeId => ColumnType::Uuid.def(),
            Self::Name => ColumnType::String(StringLen::N(255u32)).def(),
            Self::Value => ColumnType::String(StringLen::N(2048u32)).def().null(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::ClientScopes => Entity::belongs_to(super::client_scopes::Entity)
                .from(Column::ScopeId)
                .to(super::client_scopes::Column::Id)
                .into(),
        }
    }
}

impl Related<super::client_scopes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClientScopes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### `core/src/entity/client_scope_protocol_mappers.rs`

```rust
//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.14

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "client_scope_protocol_mappers"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: Uuid,
    pub client_scope_id: Uuid,
    pub name: String,
    pub mapper_type: String,
    pub config: Json,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    ClientScopeId,
    Name,
    MapperType,
    Config,
    CreatedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = Uuid;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    ClientScopes,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Uuid.def(),
            Self::ClientScopeId => ColumnType::Uuid.def(),
            Self::Name => ColumnType::String(StringLen::N(255u32)).def(),
            Self::MapperType => ColumnType::String(StringLen::N(255u32)).def(),
            Self::Config => ColumnType::JsonBinary.def(),
            Self::CreatedAt => ColumnType::DateTime.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::ClientScopes => Entity::belongs_to(super::client_scopes::Entity)
                .from(Column::ClientScopeId)
                .to(super::client_scopes::Column::Id)
                .into(),
        }
    }
}

impl Related<super::client_scopes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClientScopes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### `core/src/entity/client_scope_mappings.rs`

```rust
//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.14

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "client_scope_mappings"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub client_id: Uuid,
    pub scope_id: Uuid,
    pub is_default: bool,
    pub is_optional: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    ClientId,
    ScopeId,
    IsDefault,
    IsOptional,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    ClientId,
    ScopeId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = (Uuid, Uuid);
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Clients,
    ClientScopes,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::ClientId => ColumnType::Uuid.def(),
            Self::ScopeId => ColumnType::Uuid.def(),
            Self::IsDefault => ColumnType::Boolean.def(),
            Self::IsOptional => ColumnType::Boolean.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Clients => Entity::belongs_to(super::clients::Entity)
                .from(Column::ClientId)
                .to(super::clients::Column::Id)
                .into(),
            Self::ClientScopes => Entity::belongs_to(super::client_scopes::Entity)
                .from(Column::ScopeId)
                .to(super::client_scopes::Column::Id)
                .into(),
        }
    }
}

impl Related<super::clients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Clients.def()
    }
}

impl Related<super::client_scopes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClientScopes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### Mettre à jour `core/src/entity/mod.rs`

Ajoute (ordre alphabétique) :

```rust
pub mod client_scope_attributes;
pub mod client_scope_mappings;
pub mod client_scope_protocol_mappers;
pub mod client_scopes;
```

---

## 7. Infrastructure — `core/src/infrastructure/aegis/`

### Structure

```
core/src/infrastructure/aegis/
├── mod.rs
├── mappers.rs
└── repositories/
    ├── mod.rs
    ├── client_scope_postgres_repository.rs
    ├── client_scope_attribute_postgres_repository.rs
    ├── protocol_mapper_postgres_repository.rs
    └── scope_mapping_postgres_repository.rs
```

### `infrastructure/aegis/mod.rs`

```rust
mod mappers;
pub mod repositories;
```

### Enregistrer dans `core/src/infrastructure/mod.rs`

Ajoute `pub mod aegis;` (ordre alphabétique, avant `client`).

---

### `infrastructure/aegis/mappers.rs`

Suis exactement le pattern de `core/src/infrastructure/seawatch/mapper.rs` et `core/src/infrastructure/client/mappers/client_mapper.rs` :

- Timestamps : `Utc.from_utc_datetime(&model.created_at)` pour Model → Domain
- Timestamps : `Set(value.naive_utc())` pour Domain → ActiveModel
- UUIDs : `.into()` pour les conversions `RealmId` ↔ `Uuid`

**Implémente ces conversions :**

```rust
use chrono::{TimeZone, Utc};
use sea_orm::ActiveValue::Set;

use crate::domain::aegis::entities::{
    ClientScope, ClientScopeAttribute, ClientScopeMapping, ProtocolMapper,
};
use crate::entity::{
    client_scope_attributes, client_scope_mappings, client_scope_protocol_mappers, client_scopes,
};
```

**`From<client_scopes::Model> for ClientScope`** :

- `realm_id: model.realm_id.into()`
- `attributes: None` et `protocol_mappers: None` (chargés séparément)
- Timestamps : `Utc.from_utc_datetime(&model.created_at)`

**`From<client_scope_attributes::Model> for ClientScopeAttribute`** :

- Mapping direct des champs

**`From<client_scope_protocol_mappers::Model> for ProtocolMapper`** :

- `config: model.config` (Json → serde_json::Value, même type)
- `created_at: Utc.from_utc_datetime(&model.created_at)`

**`From<client_scope_mappings::Model> for ClientScopeMapping`** :

- Mapping direct des champs

---

### `infrastructure/aegis/repositories/mod.rs`

```rust
pub mod client_scope_postgres_repository;
pub mod client_scope_attribute_postgres_repository;
pub mod protocol_mapper_postgres_repository;
pub mod scope_mapping_postgres_repository;
```

---

### `infrastructure/aegis/repositories/client_scope_postgres_repository.rs`

Suis le pattern exact de `core/src/infrastructure/client/repositories/client_postgres_repository.rs`.

```rust
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::aegis::entities::ClientScope;
use crate::domain::aegis::ports::ClientScopeRepository;
use crate::domain::aegis::value_objects::{CreateClientScopeRequest, UpdateClientScopeRequest};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::domain::realm::entities::RealmId;
use crate::entity::client_scopes;

#[derive(Debug, Clone)]
pub struct PostgresClientScopeRepository {
    pub db: DatabaseConnection,
}

impl PostgresClientScopeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
```

**Implémente `ClientScopeRepository` :**

- **`create`** : Construis un `client_scopes::ActiveModel` avec `Set()` pour chaque champ, `id: Set(generate_uuid_v7())`, timestamps `Set(Utc::now().naive_utc())`. Utilise `.insert(&self.db)` puis `.into()` pour convertir en `ClientScope`.

- **`get_by_id`** : `client_scopes::Entity::find().filter(client_scopes::Column::Id.eq(id)).one(&self.db)` puis `.map(ClientScope::from)`.

- **`find_by_realm_id`** : `.filter(client_scopes::Column::RealmId.eq::<Uuid>(realm_id.into())).all(&self.db)` puis `.into_iter().map(ClientScope::from).collect()`.

- **`find_by_name`** : `.filter(client_scopes::Column::Name.eq(name)).filter(client_scopes::Column::RealmId.eq::<Uuid>(realm_id.into())).one(&self.db)` puis `.map(ClientScope::from)`.

- **`update_by_id`** : Fetch par ID, `.ok_or(CoreError::NotFound)?`, convertir en ActiveModel, appliquer les champs `Option` avec le pattern `match data.field { Some(v) => Set(v), None => active.field }`, set `updated_at`, `.update(&self.db)`, `.into()`.

- **`delete_by_id`** : `client_scopes::Entity::delete_many().filter(client_scopes::Column::Id.eq(id)).exec(&self.db)`, vérifier `rows_affected == 0` → `CoreError::NotFound`.

Toutes les erreurs SeaORM : `.map_err(|e| { tracing::error!("...": {}", e); CoreError::InternalServerError })`.

---

### `infrastructure/aegis/repositories/client_scope_attribute_postgres_repository.rs`

Même struct pattern (`PostgresClientScopeAttributeRepository`).

**Implémente `ClientScopeAttributeRepository` :**

- **`set_attribute`** : Cherche d'abord si l'attribut existe (`.filter(scope_id).filter(name)`). S'il existe → update value. Sinon → insert. Retourne `ClientScopeAttribute`.

- **`get_attributes`** : `.filter(client_scope_attributes::Column::ScopeId.eq(scope_id)).all(&self.db)`.

- **`remove_attribute`** : `delete_many().filter(scope_id).filter(name)`.

---

### `infrastructure/aegis/repositories/protocol_mapper_postgres_repository.rs`

Même struct pattern (`PostgresProtocolMapperRepository`).

**Implémente `ProtocolMapperRepository` :**

- **`create`** : ActiveModel avec `Set()`, `config: Set(payload.config)` (serde_json::Value → Json direct). Insert.
- **`get_by_id`** : find + filter Id.
- **`get_by_scope_id`** : find + filter ClientScopeId.
- **`update_by_id`** : Fetch, convert ActiveModel, apply Optional fields, update.
- **`delete_by_id`** : delete_many + filter.

---

### `infrastructure/aegis/repositories/scope_mapping_postgres_repository.rs`

Même struct pattern (`PostgresScopeMappingRepository`).

**Implémente `ClientScopeMappingRepository` :**

- **`assign_scope_to_client`** : Insert ActiveModel avec les 4 champs (composite PK, pas d'UUID généré). Retourne `ClientScopeMapping`.

- **`remove_scope_from_client`** : `delete_many().filter(client_id).filter(scope_id)`.

- **`get_client_scopes`** : `.filter(client_scope_mappings::Column::ClientId.eq(client_id)).all(&self.db)` → `Vec<ClientScopeMapping>`.

- **`get_default_scopes`** : JOIN `client_scope_mappings` avec `client_scopes` via `client_scope_mappings::Relation::ClientScopes.def()`. Filter `ClientId.eq(client_id)` + `IsDefault.eq(true)`. Retourne `Vec<ClientScope>` (map le Model client_scopes, pas le mapping).

- **`get_optional_scopes`** : Même chose avec `IsOptional.eq(true)`.

Pour les JOINs, utilise ce pattern (comme `webhook_repository.rs`) :

```rust
use sea_orm::{JoinType, QuerySelect, RelationTrait};

client_scope_mappings::Entity::find()
    .join(JoinType::InnerJoin, client_scope_mappings::Relation::ClientScopes.def())
    .filter(client_scope_mappings::Column::ClientId.eq(client_id))
    .filter(client_scope_mappings::Column::IsDefault.eq(true))
    // ...
```

Note : Le résultat du join retourne des `client_scope_mappings::Model` par défaut. Pour obtenir les `client_scopes::Model`, utilise `.find_also_related(client_scopes::Entity)` ou construis une query qui sélectionne depuis `client_scopes::Entity` avec le join inversé.

Alternative plus simple : query depuis `client_scopes::Entity` en sous-requête :

```rust
// Récupère d'abord les scope_ids, puis query client_scopes
let mappings = client_scope_mappings::Entity::find()
    .filter(client_scope_mappings::Column::ClientId.eq(client_id))
    .filter(client_scope_mappings::Column::IsDefault.eq(true))
    .all(&self.db)
    .await?;

let scope_ids: Vec<Uuid> = mappings.iter().map(|m| m.scope_id).collect();

let scopes = client_scopes::Entity::find()
    .filter(client_scopes::Column::Id.is_in(scope_ids))
    .all(&self.db)
    .await?;
```

---

## Contraintes

- Respecte strictement les patterns existants (imports, derives, generics, `Arc`, `ensure_policy`, error handling)
- **Ne crée PAS** de services.rs à la racine — c'est un **dossier** `services/`
- L'impl du trait `ClientScopeService` dans `services/mod.rs` délègue à `self.handle_*()` (une ligne par méthode)
- Toutes les méthodes handle sont `pub(super)` pour n'être visibles que depuis `services/mod.rs`
- Pas de webhooks ni security events dans ces services (on les ajoutera plus tard)
- Les errors utilisées : `CoreError::InvalidRealm` (realm), `CoreError::NotFound` (entity), `CoreError::Forbidden` (via `ensure_policy`), `CoreError::InternalServerError` (DB)
- Les entity SeaORM dans `core/src/entity/` sont marqués comme "generated" mais écrits manuellement ici puisqu'on n'a pas de DB live
- Ne câble PAS `ApplicationService` — on ajoutera le wiring plus tard
- Vérifie la compilation avec `cargo check -p ferriskey-core`
