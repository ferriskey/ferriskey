//! `SeaORM` Entity. Hand-written to match `20260818120000_create_login_action_tokens`;
//! regenerate with sea-orm-cli against a live database when convenient.

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "login_action_tokens"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub jti: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub auth_session_id: Uuid,
    pub expires_at: DateTime,
    pub consumed_at: Option<DateTime>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Jti,
    UserId,
    RealmId,
    AuthSessionId,
    ExpiresAt,
    ConsumedAt,
    CreatedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Jti,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = Uuid;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Users,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Jti => ColumnType::Uuid.def(),
            Self::UserId => ColumnType::Uuid.def(),
            Self::RealmId => ColumnType::Uuid.def(),
            Self::AuthSessionId => ColumnType::Uuid.def(),
            Self::ExpiresAt => ColumnType::DateTime.def(),
            Self::ConsumedAt => ColumnType::DateTime.def().null(),
            Self::CreatedAt => ColumnType::DateTime.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Users => Entity::belongs_to(super::users::Entity)
                .from(Column::UserId)
                .to(super::users::Column::Id)
                .into(),
        }
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
