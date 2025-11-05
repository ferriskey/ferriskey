use ldap3::{LdapConn, Scope, SearchEntry};

use crate::domain::{
    abyss::{
        entities::{ldap_provider::LdapProvider, ldap_user::LdapUser},
        ports::ldap::LdapConnector,
    },
    common::entities::app_errors::CoreError,
};

pub struct LdapAdapter;

impl LdapConnector for LdapAdapter {
    async fn authenticate(
        &self,
        provider: &LdapProvider,
        username: &str,
        password: &str,
    ) -> Result<bool, CoreError> {
        let mut ldap = LdapConn::new(&provider.url).map_err(|_| CoreError::InternalServerError)?;
        let user_dn = format!(
            "{}={},{}",
            provider.username_attr, username, provider.user_base_dn
        );
        ldap.simple_bind(&user_dn, password)
            .map_err(|_| CoreError::InternalServerError)?
            .success()
            .map_err(|_| CoreError::InternalServerError)?;
        Ok(true)
    }

    async fn fetch_users(&self, provider: &LdapProvider) -> Result<Vec<LdapUser>, CoreError> {
        let mut ldap = LdapConn::new(&provider.url).map_err(|_| CoreError::InternalServerError)?;

        ldap.simple_bind(&provider.bind_dn, &provider.bind_password)
            .map_err(|_| CoreError::InternalServerError)?
            .success()
            .map_err(|_| CoreError::InternalServerError)?;

        let (entries, _res) = ldap
            .search(
                &provider.user_base_dn,
                Scope::Subtree,
                &provider.user_filter,
                vec!["dn", &provider.username_attr, "mail", "cn"],
            )
            .map_err(|_| CoreError::InternalServerError)?
            .success()
            .map_err(|_| CoreError::InternalServerError)?;

        let users: Vec<LdapUser> = entries
            .into_iter()
            .map(|entry| {
                let e = SearchEntry::construct(entry);
                LdapUser {
                    dn: e.dn,
                    username: e
                        .attrs
                        .get(&provider.username_attr)
                        .and_then(|v| v.first().cloned())
                        .unwrap_or_default(),
                    email: e.attrs.get("mail").and_then(|v| v.first().cloned()),
                    display_name: e.attrs.get("cn").and_then(|v| v.first().cloned()),
                    realm_id: provider.realm_id,
                }
            })
            .collect();

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        domain::abyss::{entities::ldap_provider::LdapProvider, ports::ldap::LdapConnector},
        infrastructure::abyss::repositories::ldap::LdapAdapter,
    };

    fn create_test_provider() -> LdapProvider {
        LdapProvider {
            id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            url: "ldap://localhost:389".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "admin".to_string(),
            user_base_dn: "ou=users,dc=example,dc=com".to_string(),
            user_filter: "(objectClass=person)".to_string(),
            username_attr: "uid".to_string(),
            email_attr: Some("mail".to_string()),
            display_name_attr: Some("cn".to_string()),
            enabled: true,
        }
    }

    #[tokio::test]
    async fn test_ldap_adapter_creation() {
        let adapter = LdapAdapter;

        assert_eq!(std::mem::size_of_val(&adapter), 0);
    }

    #[tokio::test]
    async fn test_user_dn_formatting() {
        let provider = create_test_provider();
        let username = "johndoe";

        let expected_dn = format!(
            "{}={},{}",
            provider.username_attr, username, provider.user_base_dn
        );

        assert_eq!(expected_dn, "uid=johndoe,ou=users,dc=example,dc=com");
    }

    // Integration tests that require a real LDAP server
    // These will only run when the `ldap-integration` feature is enabled
    #[cfg(feature = "ldap-integration")]
    mod integration_tests {
        fn get_test_ldap_config() -> LdapProvider {
            LdapProvider {
                id: Uuid::new_v4(),
                realm_id: Uuid::new_v4(),
                url: env::var("LDAP_TEST_URL")
                    .unwrap_or_else(|_| "ldap://localhost:389".to_string()),
                bind_dn: env::var("LDAP_TEST_BIND_DN")
                    .unwrap_or_else(|_| "cn=admin,dc=example,dc=com".to_string()),
                bind_password: env::var("LDAP_TEST_BIND_PASSWORD")
                    .unwrap_or_else(|_| "admin".to_string()),
                user_base_dn: env::var("LDAP_TEST_USER_BASE_DN")
                    .unwrap_or_else(|_| "ou=users,dc=example,dc=com".to_string()),
                user_filter: env::var("LDAP_TEST_USER_FILTER")
                    .unwrap_or_else(|_| "(objectClass=person)".to_string()),
                username_attr: env::var("LDAP_TEST_USERNAME_ATTR")
                    .unwrap_or_else(|_| "uid".to_string()),
                email_attr: Some("mail".to_string()),
                display_name_attr: Some("cn".to_string()),
                enabled: true,
            }
        }

        #[tokio::test]
        #[ignore = "Requires LDAP server - run with --features ldap-integration"]
        async fn test_authenticate_with_real_ldap() {
            let adapter = LdapAdapter;
            let provder = get_test_ldap_config();

            let result = adapter
                .authenticate(&provider, "nonexistant", "wrongpassword")
                .await;

            assert!(result.is_err());
        }
    }
}
