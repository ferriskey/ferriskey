use sea_orm::entity::prelude::*;

#[derive(Debug)]
pub struct UserToRole;

impl Linked for UserToRole {
    type FromEntity = crate::entity::users::Entity;
    type ToEntity = crate::entity::roles::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            crate::entity::users::Relation::UserRole.def(),
            crate::entity::user_role::Relation::Roles.def(),
        ]
    }
}
