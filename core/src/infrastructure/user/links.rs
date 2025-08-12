use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug)]
pub struct UserToRole;

impl Linked for UserToRole {
    type FromEntity = crate::infrastructure::entities::users::Entity;
    type ToEntity = crate::infrastructure::entities::roles::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            crate::infrastructure::entities::users::Relation::UserRole.def(),
            crate::infrastructure::entities::user_role::Relation::Roles.def(),
        ]
    }
}
