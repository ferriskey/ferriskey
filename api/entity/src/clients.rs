//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "clients"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub client_id: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub client_type: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    RealmId,
    Name,
    ClientId,
    Secret,
    Enabled,
    Protocol,
    PublicClient,
    ServiceAccountEnabled,
    ClientType,
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
    AuthSessions,
    Realms,
    RedirectUris,
    Roles,
    Users,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Uuid.def(),
            Self::RealmId => ColumnType::Uuid.def(),
            Self::Name => ColumnType::String(StringLen::N(255u32)).def(),
            Self::ClientId => ColumnType::String(StringLen::N(255u32)).def(),
            Self::Secret => ColumnType::String(StringLen::N(255u32)).def().null(),
            Self::Enabled => ColumnType::Boolean.def(),
            Self::Protocol => ColumnType::String(StringLen::N(255u32)).def(),
            Self::PublicClient => ColumnType::Boolean.def(),
            Self::ServiceAccountEnabled => ColumnType::Boolean.def(),
            Self::ClientType => ColumnType::String(StringLen::N(255u32)).def(),
            Self::CreatedAt => ColumnType::DateTime.def(),
            Self::UpdatedAt => ColumnType::DateTime.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AuthSessions => Entity::has_many(super::auth_sessions::Entity).into(),
            Self::Realms => Entity::belongs_to(super::realms::Entity)
                .from(Column::RealmId)
                .to(super::realms::Column::Id)
                .into(),
            Self::RedirectUris => Entity::has_many(super::redirect_uris::Entity).into(),
            Self::Roles => Entity::has_many(super::roles::Entity).into(),
            Self::Users => Entity::has_one(super::users::Entity).into(),
        }
    }
}

impl Related<super::auth_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthSessions.def()
    }
}

impl Related<super::realms::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Realms.def()
    }
}

impl Related<super::redirect_uris::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RedirectUris.def()
    }
}

impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Roles.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
