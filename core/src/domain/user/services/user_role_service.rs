use uuid::Uuid;

use crate::domain::{
    realm::ports::RealmRepository,
    role::{entities::Role, ports::RoleRepository},
    user::{
        entities::UserError,
        ports::{UserRepository, UserRoleRepository, UserRoleService},
    },
};

#[derive(Clone)]
pub struct UserRoleServiceImpl<U, R, RM, UR>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
    UR: UserRoleRepository,
{
    pub user_repository: U,
    pub role_repository: R,
    pub realm_repository: RM,
    pub user_role_repository: UR,
}

impl<U, R, RM, UR> UserRoleServiceImpl<U, R, RM, UR>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
    UR: UserRoleRepository,
{
    pub fn new(
        user_repository: U,
        role_repository: R,
        realm_repository: RM,
        user_role_repository: UR,
    ) -> Self {
        Self {
            user_repository,
            role_repository,
            realm_repository,
            user_role_repository,
        }
    }
}

impl<U, R, RM, UR> UserRoleService for UserRoleServiceImpl<U, R, RM, UR>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
    UR: UserRoleRepository,
{
    async fn assign_role(
        &self,
        realm_name: String,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), UserError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?
            .ok_or(UserError::InternalServerError)?;

        let role = self
            .role_repository
            .get_by_id(role_id)
            .await
            .map_err(|_| UserError::InternalServerError)?
            .ok_or(UserError::InternalServerError)?;

        let user = self.user_repository.get_by_id(user_id).await?;

        if user.realm_id != realm.id || role.realm_id != realm.id {
            return Err(UserError::InternalServerError);
        }

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, UserError> {
        self.user_role_repository.get_user_roles(user_id).await
    }

    async fn has_role(&self, _user_id: Uuid, _role_id: Uuid) -> Result<bool, UserError> {
        unimplemented!("has_role method is not implemented yet");
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserError> {
        self.user_role_repository
            .revoke_role(user_id, role_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realm::entities::Realm;
    use crate::domain::realm::ports::test::get_mock_realm_repository_with_clone_expectations;
    use crate::domain::role::ports::test::get_mock_role_repository_with_clone_expectations;
    use crate::domain::user::entities::{User, UserConfig};
    use crate::domain::user::ports::test::{
        get_mock_user_repository_with_clone_expectations,
        get_mock_user_role_repository_with_clone_expectations,
    };
    use crate::domain::realm::ports::test::MockRealmRepository;
    use crate::domain::role::ports::test::MockRoleRepository;
    use crate::domain::user::ports::test::MockUserRepository;
    use crate::domain::user::ports::test::MockUserRoleRepository;
    use std::future::ready;

    fn build_service() -> (
        MockRealmRepository,
        MockRoleRepository,
        MockUserRepository,
        MockUserRoleRepository,
    ) {
        let realm_repo = get_mock_realm_repository_with_clone_expectations();
        let role_repo = get_mock_role_repository_with_clone_expectations();
        let user_repo = get_mock_user_repository_with_clone_expectations();
        let user_role_repo = get_mock_user_role_repository_with_clone_expectations();

        (
            realm_repo,
            role_repo,
            user_repo,
            user_role_repo,
        )
    }

    #[tokio::test]
    async fn assign_role_success_calls_repo() {
        let (mut realm_repo, mut role_repo, mut user_repo, mut user_role_repo) =
            build_service();

        let realm = Realm::new("master".into());
        let role = Role {
            id: Uuid::new_v4(),
            name: "reader".into(),
            description: None,
            permissions: vec![],
            realm_id: realm.id,
            client_id: None,
            client: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let user = User::new(UserConfig {
            realm_id: realm.id,
            client_id: None,
            username: "alice".into(),
            firstname: "Alice".into(),
            lastname: "Doe".into(),
            email: "alice@example.com".into(),
            email_verified: true,
            enabled: true,
        });

        let role_clone = role.clone();
        role_repo
            .expect_get_by_id()
            .returning(move |rid: Uuid| {
                assert_eq!(rid, role_clone.id);
                Box::pin(ready(Ok(Some(role_clone.clone()))))
            });
        let user_clone = user.clone();
        user_repo
            .expect_get_by_id()
            .returning(move |uid: Uuid| {
                assert_eq!(uid, user_clone.id);
                Box::pin(ready(Ok(user_clone.clone())))
            });
        let realm_clone = realm.clone();
        realm_repo
            .expect_get_by_name()
            .returning(move |name: String| {
                assert_eq!(name, "master");
                Box::pin(ready(Ok(Some(realm_clone.clone()))))
            });

        let user_id_for_assert = user.id;
        let role_id_for_assert = role.id;
        user_role_repo
            .expect_assign_role()
            .returning(move |uid: Uuid, rid: Uuid| {
                assert_eq!(uid, user_id_for_assert);
                assert_eq!(rid, role_id_for_assert);
                Box::pin(ready(Ok(())))
            });

        let service = UserRoleServiceImpl::<MockUserRepository, MockRoleRepository, MockRealmRepository, MockUserRoleRepository>::new(user_repo, role_repo, realm_repo, user_role_repo);

        let out = service
            .assign_role("master".into(), user.id, role.id)
            .await
            .unwrap();
        assert_eq!(out, ());
    }

    #[tokio::test]
    async fn assign_role_realm_mismatch_errors() {
        let (mut realm_repo, mut role_repo, mut user_repo, _urr) = build_service();

        let realm = Realm::new("master".into());
        let other_realm_id = Uuid::new_v4();
        let role = Role {
            id: Uuid::new_v4(),
            name: "reader".into(),
            description: None,
            permissions: vec![],
            realm_id: other_realm_id, // mismatch
            client_id: None,
            client: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let user = User::new(UserConfig {
            realm_id: realm.id,
            client_id: None,
            username: "alice".into(),
            firstname: "Alice".into(),
            lastname: "Doe".into(),
            email: "alice@example.com".into(),
            email_verified: true,
            enabled: true,
        });

        let role_clone = role.clone();
        role_repo
            .expect_get_by_id()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(Some(role_clone.clone())))));
        let user_clone = user.clone();
        user_repo
            .expect_get_by_id()
            .returning(move |_uid: Uuid| Box::pin(ready(Ok(user_clone.clone()))));
        let realm_clone = realm.clone();
        realm_repo
            .expect_get_by_name()
            .returning(move |_name: String| Box::pin(ready(Ok(Some(realm_clone.clone())))));

        let service = UserRoleServiceImpl::<MockUserRepository, MockRoleRepository, MockRealmRepository, MockUserRoleRepository>::new(user_repo, role_repo, realm_repo, _urr);

        let err = service
            .assign_role("master".into(), user.id, role.id)
            .await
            .err()
            .unwrap();
        assert!(matches!(err, UserError::InternalServerError));
    }

    #[tokio::test]
    async fn get_user_roles_forwards() {
        let (_rr, _ro, _ur, mut user_role_repo) = build_service();
        let user_id = Uuid::new_v4();
        let roles = vec![Role {
            id: Uuid::new_v4(),
            name: "reader".into(),
            description: None,
            permissions: vec![],
            realm_id: Uuid::new_v4(),
            client_id: None,
            client: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }];
        let roles_clone = roles.clone();
        user_role_repo
            .expect_get_user_roles()
            .returning(move |uid: Uuid| {
                assert_eq!(uid, user_id);
                Box::pin(ready(Ok(roles_clone.clone())))
            });

        let service = UserRoleServiceImpl::<MockUserRepository, MockRoleRepository, MockRealmRepository, MockUserRoleRepository>::new(_ur, _ro, _rr, user_role_repo);
        let out = service.get_user_roles(user_id).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "reader");
    }

    #[tokio::test]
    async fn revoke_role_forwards() {
        let (_rr, _ro, _ur, mut user_role_repo) = build_service();
        let user_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();
        let uid_assert = user_id;
        let rid_assert = role_id;
        user_role_repo
            .expect_revoke_role()
            .returning(move |uid: Uuid, rid: Uuid| {
                assert_eq!(uid, uid_assert);
                assert_eq!(rid, rid_assert);
                Box::pin(ready(Ok(())))
            });

        let service = UserRoleServiceImpl::<MockUserRepository, MockRoleRepository, MockRealmRepository, MockUserRoleRepository>::new(_ur, _ro, _rr, user_role_repo);
        let out = service.revoke_role(user_id, role_id).await.unwrap();
        assert_eq!(out, ());
    }
}
