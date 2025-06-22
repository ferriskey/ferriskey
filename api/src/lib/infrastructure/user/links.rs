use sea_orm::entity::prelude::*;

#[derive(Debug)]
pub struct UserToRole;

impl Linked for UserToRole {
    type FromEntity = entity::users::Entity;
    type ToEntity = entity::roles::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            entity::users::Relation::UserRole.def(),
            entity::user_role::Relation::Roles.def(),
        ]
    }
}
