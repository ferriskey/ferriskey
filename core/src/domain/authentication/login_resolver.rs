use ferriskey_domain::realm::{LoginAlias, LoginAliases, RealmId};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::user::entities::User;
use crate::domain::user::ports::UserRepository;

/// Resolve a user from a login identifier, trying each configured alias in order.
/// The first alias that yields a user wins (list order = precedence).
pub(crate) async fn resolve_user_by_identifier<U: UserRepository>(
    user_repository: &U,
    identifier: &str,
    realm_id: RealmId,
    aliases: &LoginAliases,
) -> Result<Option<User>, CoreError> {
    for alias in aliases.as_slice() {
        let found = match alias {
            LoginAlias::Username => {
                user_repository
                    .find_by_username(identifier.to_string(), realm_id)
                    .await?
            }
            LoginAlias::Email => user_repository.get_by_email(identifier, realm_id).await?,
        };
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use ferriskey_domain::realm::RealmId;

    use super::*;
    use crate::domain::user::ports::MockUserRepository;

    fn user_named(name: &str) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: RealmId::from(Uuid::new_v4()),
            client_id: None,
            username: name.to_string(),
            firstname: None,
            lastname: None,
            email: None,
            email_verified: false,
            enabled: true,
            roles: None,
            realm: None,
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn username_only_uses_username_lookup() {
        let realm_id = RealmId::from(uuid::Uuid::now_v7());
        let mut repo = MockUserRepository::new();
        repo.expect_find_by_username()
            .returning(|_, _| Box::pin(async { Ok(Some(user_named("bob"))) }));
        let aliases = LoginAliases::try_new(vec![LoginAlias::Username]).unwrap();

        let user = resolve_user_by_identifier(&repo, "bob", realm_id, &aliases)
            .await
            .unwrap();
        assert_eq!(user.unwrap().username, "bob");
    }

    #[tokio::test]
    async fn email_only_uses_email_lookup() {
        let realm_id = RealmId::from(uuid::Uuid::now_v7());
        let mut repo = MockUserRepository::new();
        repo.expect_get_by_email()
            .returning(|_, _| Box::pin(async { Ok(Some(user_named("bob"))) }));
        let aliases = LoginAliases::try_new(vec![LoginAlias::Email]).unwrap();

        let user = resolve_user_by_identifier(&repo, "bob@x.io", realm_id, &aliases)
            .await
            .unwrap();
        assert!(user.is_some());
    }

    #[tokio::test]
    async fn username_or_email_falls_back_to_email_in_order() {
        let realm_id = RealmId::from(uuid::Uuid::now_v7());
        let mut repo = MockUserRepository::new();
        repo.expect_find_by_username()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_get_by_email()
            .returning(|_, _| Box::pin(async { Ok(Some(user_named("via-email"))) }));
        let aliases = LoginAliases::try_new(vec![LoginAlias::Username, LoginAlias::Email]).unwrap();

        let user = resolve_user_by_identifier(&repo, "bob@x.io", realm_id, &aliases)
            .await
            .unwrap();
        assert_eq!(user.unwrap().username, "via-email");
    }

    #[tokio::test]
    async fn returns_none_when_no_alias_matches() {
        let realm_id = RealmId::from(uuid::Uuid::now_v7());
        let mut repo = MockUserRepository::new();
        repo.expect_find_by_username()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_get_by_email()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        let aliases = LoginAliases::try_new(vec![LoginAlias::Username, LoginAlias::Email]).unwrap();

        let user = resolve_user_by_identifier(&repo, "ghost", realm_id, &aliases)
            .await
            .unwrap();
        assert!(user.is_none());
    }
}
