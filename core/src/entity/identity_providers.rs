//! `SeaORM` Entity for identity_providers table

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "identity_providers"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub alias: String,
    pub provider_id: String,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: bool,
    pub add_read_token_role_on_create: bool,
    pub trust_email: bool,
    pub link_only: bool,
    pub config: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    RealmId,
    Alias,
    ProviderId,
    Enabled,
    DisplayName,
    FirstBrokerLoginFlowAlias,
    PostBrokerLoginFlowAlias,
    StoreToken,
    AddReadTokenRoleOnCreate,
    TrustEmail,
    LinkOnly,
    Config,
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
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Uuid.def(),
            Self::RealmId => ColumnType::Uuid.def(),
            Self::Alias => ColumnType::String(StringLen::N(255u32)).def(),
            Self::ProviderId => ColumnType::String(StringLen::N(255u32)).def(),
            Self::Enabled => ColumnType::Boolean.def(),
            Self::DisplayName => ColumnType::String(StringLen::N(255u32)).def().null(),
            Self::FirstBrokerLoginFlowAlias => {
                ColumnType::String(StringLen::N(255u32)).def().null()
            }
            Self::PostBrokerLoginFlowAlias => ColumnType::String(StringLen::N(255u32)).def().null(),
            Self::StoreToken => ColumnType::Boolean.def(),
            Self::AddReadTokenRoleOnCreate => ColumnType::Boolean.def(),
            Self::TrustEmail => ColumnType::Boolean.def(),
            Self::LinkOnly => ColumnType::Boolean.def(),
            Self::Config => ColumnType::JsonBinary.def(),
            Self::CreatedAt => ColumnType::TimestampWithTimeZone.def(),
            Self::UpdatedAt => ColumnType::TimestampWithTimeZone.def(),
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
        }
    }
}

impl Related<super::realms::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Realms.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
