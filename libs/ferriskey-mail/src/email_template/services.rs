use std::sync::Arc;

use crate::email_template::entities::EmailTemplate;
use crate::email_template::ports::{
    CreateEmailTemplateInput, DeleteEmailTemplateInput, EmailTemplatePolicy,
    EmailTemplateRepository, EmailTemplateService, EmailTemplateSource, GetEmailTemplateInput,
    GetEmailTemplatesInput, ImportEmailTemplateInput, RenderEmailTemplateInput, TemplateRenderer,
    UpdateEmailTemplateInput,
};
use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, ensure_policy};
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

#[derive(Clone, Debug)]
pub struct EmailTemplateServiceImpl<R, U, C, UR, ET, TR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    ET: EmailTemplateRepository,
    TR: TemplateRenderer,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) email_template_repository: Arc<ET>,
    pub(crate) template_renderer: Arc<TR>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, ET, TR> EmailTemplateServiceImpl<R, U, C, UR, ET, TR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    ET: EmailTemplateRepository,
    TR: TemplateRenderer,
{
    pub fn new(
        realm_repository: Arc<R>,
        email_template_repository: Arc<ET>,
        template_renderer: Arc<TR>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            email_template_repository,
            template_renderer,
            policy,
        }
    }
}

impl<R, U, C, UR, ET, TR> EmailTemplateService for EmailTemplateServiceImpl<R, U, C, UR, ET, TR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    ET: EmailTemplateRepository,
    TR: TemplateRenderer,
{
    async fn get_templates_by_realm(
        &self,
        identity: Identity,
        input: GetEmailTemplatesInput,
    ) -> Result<Vec<EmailTemplate>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_email_template(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.email_template_repository
            .fetch_by_realm(realm.id.into())
            .await
    }

    async fn get_template(
        &self,
        identity: Identity,
        input: GetEmailTemplateInput,
    ) -> Result<EmailTemplate, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_email_template(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.email_template_repository
            .get_by_id(realm.id.into(), input.template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)
    }

    async fn create_template(
        &self,
        identity: Identity,
        input: CreateEmailTemplateInput,
    ) -> Result<EmailTemplate, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy
                .can_manage_email_template(&identity, &realm)
                .await,
            "insufficient permissions",
        )?;

        let mjml = self
            .template_renderer
            .render_to_intermediate(&input.structure)?;

        // Validate that the MJML can be converted to HTML
        self.template_renderer.render_to_html(&mjml)?;

        self.email_template_repository
            .create(
                realm.id.into(),
                input.name,
                input.email_type.to_string(),
                input.structure,
                mjml,
            )
            .await
    }

    async fn update_template(
        &self,
        identity: Identity,
        input: UpdateEmailTemplateInput,
    ) -> Result<EmailTemplate, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy
                .can_manage_email_template(&identity, &realm)
                .await,
            "insufficient permissions",
        )?;

        self.email_template_repository
            .get_by_id(realm.id.into(), input.template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        let mjml = self
            .template_renderer
            .render_to_intermediate(&input.structure)?;

        // Validate that the MJML can be converted to HTML
        self.template_renderer.render_to_html(&mjml)?;

        self.email_template_repository
            .update(
                realm.id.into(),
                input.template_id,
                input.name,
                input.structure,
                mjml,
            )
            .await
    }

    async fn delete_template(
        &self,
        identity: Identity,
        input: DeleteEmailTemplateInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy
                .can_manage_email_template(&identity, &realm)
                .await,
            "insufficient permissions",
        )?;

        self.email_template_repository
            .get_by_id(realm.id.into(), input.template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        self.email_template_repository
            .delete(realm.id.into(), input.template_id)
            .await
    }

    async fn render_template_html(
        &self,
        identity: Identity,
        input: RenderEmailTemplateInput,
    ) -> Result<String, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_email_template(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let template = self
            .email_template_repository
            .get_by_id(realm.id.into(), input.template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        self.template_renderer.render_to_html(&template.mjml)
    }

    async fn import_template(
        &self,
        identity: Identity,
        input: ImportEmailTemplateInput,
    ) -> Result<EmailTemplate, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy
                .can_manage_email_template(&identity, &realm)
                .await,
            "insufficient permissions",
        )?;

        // MJML imports are parsed back into a builder structure so the imported
        // template stays editable in the builder, like any other template.
        let structure = match input.source {
            EmailTemplateSource::Structure(structure) => structure,
            EmailTemplateSource::Mjml(mjml) => self.template_renderer.parse_intermediate(&mjml)?,
        };

        let mjml = self.template_renderer.render_to_intermediate(&structure)?;

        // Validate that the MJML can be converted to HTML
        self.template_renderer.render_to_html(&mjml)?;

        self.email_template_repository
            .create(
                realm.id.into(),
                input.name,
                input.email_type.to_string(),
                structure,
                mjml,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email_template::entities::EmailType;
    use crate::email_template::ports::MockEmailTemplateRepository;
    use chrono::Utc;
    use ferriskey_domain::client::ports::MockClientRepository;
    use ferriskey_domain::realm::{Realm, ports::MockRealmRepository};
    use ferriskey_domain::role::entities::Role;
    use ferriskey_domain::user::{
        entities::User,
        ports::{MockUserRepository, MockUserRoleRepository},
    };
    use mockall::predicate::*;
    use serde_json::json;

    struct TestRenderer;

    impl TemplateRenderer for TestRenderer {
        fn render_to_intermediate(
            &self,
            _structure: &serde_json::Value,
        ) -> Result<String, CoreError> {
            Ok("<mjml><mj-body><mj-section><mj-column><mj-text>Test</mj-text></mj-column></mj-section></mj-body></mjml>".to_string())
        }

        fn render_to_html(&self, _intermediate: &str) -> Result<String, CoreError> {
            Ok("<html><body>Test</body></html>".to_string())
        }

        fn parse_intermediate(&self, _intermediate: &str) -> Result<serde_json::Value, CoreError> {
            Ok(
                json!({"children": [{"type": "mj-section", "props": {}, "styles": {}, "children": []}]}),
            )
        }
    }

    fn test_realm() -> Realm {
        Realm {
            id: uuid::Uuid::new_v4().into(),
            name: "test-realm".to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_user(realm: &Realm) -> User {
        User {
            id: uuid::Uuid::new_v4(),
            realm_id: realm.id,
            username: "admin".to_string(),
            firstname: Some("Admin".to_string()),
            lastname: Some("User".to_string()),
            email: Some("admin@test.com".to_string()),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            client_id: None,
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            failed_login_attempts: 0,
            locked_until: None,
        }
    }

    fn test_template(realm: &Realm) -> EmailTemplate {
        EmailTemplate {
            id: uuid::Uuid::new_v4(),
            realm_id: realm.id.into(),
            name: "Test Template".to_string(),
            email_type: EmailType::ResetPassword,
            structure: json!({"type": "root", "children": []}),
            mjml: "<mjml></mjml>".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_template() {
        let realm = test_realm();
        let user = test_user(&realm);
        let template = test_template(&realm);
        let realm_clone = realm.clone();
        let template_clone = template.clone();

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let realm = realm_clone.clone();
            Box::pin(async move { Ok(Some(realm)) })
        });

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let role_realm_id = realm.id;
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let rid = role_realm_id;
            Box::pin(async move {
                Ok(vec![Role {
                    id: uuid::Uuid::new_v4(),
                    name: "admin".to_string(),
                    description: None,
                    permissions: vec!["manage_realm".to_string()],
                    realm_id: rid,
                    client_id: None,
                    client: None,
                    require_mfa: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }])
            })
        });

        let client_repo = MockClientRepository::new();

        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_create().returning(move |_, _, _, _, _| {
            let t = template_clone.clone();
            Box::pin(async move { Ok(t) })
        });

        let policy = Arc::new(FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        ));

        let service = EmailTemplateServiceImpl::new(
            Arc::new(realm_repo),
            Arc::new(et_repo),
            Arc::new(TestRenderer),
            policy,
        );

        let identity = Identity::User(test_user(&realm));

        let result = service
            .create_template(
                identity,
                CreateEmailTemplateInput {
                    realm_name: "test-realm".to_string(),
                    name: "Test Template".to_string(),
                    email_type: EmailType::ResetPassword,
                    structure: json!({"type": "root", "children": []}),
                },
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_render_template_html() {
        let realm = named_realm("test-realm");
        let user = test_user(&realm);
        let template = test_template(&realm);
        let template_id = template.id;

        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |realm_id, _| {
            let t = template.clone();
            Box::pin(async move {
                Ok(if realm_id == t.realm_id {
                    Some(t)
                } else {
                    None
                })
            })
        });

        let service = EmailTemplateServiceImpl::new(
            Arc::new(realm_repo_for(&realm)),
            Arc::new(et_repo),
            Arc::new(TestRenderer),
            admin_policy_for(&realm, &user),
        );

        let result = service
            .render_template_html(
                Identity::User(user),
                RenderEmailTemplateInput {
                    realm_name: "test-realm".to_string(),
                    template_id,
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<html><body>Test</body></html>");
    }

    // ---------------------------------------------------------------------
    // Cross-realm isolation (FK-005)
    // ---------------------------------------------------------------------

    type TestPolicy =
        FerriskeyPolicy<MockUserRepository, MockClientRepository, MockUserRoleRepository>;

    fn named_realm(name: &str) -> Realm {
        Realm {
            id: uuid::Uuid::new_v4().into(),
            name: name.to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Realm repository that resolves the attacker's own realm (the one in the URL).
    fn realm_repo_for(realm: &Realm) -> MockRealmRepository {
        let realm = realm.clone();
        let mut repo = MockRealmRepository::new();
        repo.expect_get_by_name().returning(move |_| {
            let realm = realm.clone();
            Box::pin(async move { Ok(Some(realm)) })
        });
        repo
    }

    /// Policy granting `manage_realm` on `realm` to `user` — the attacker is a
    /// legitimate admin *of their own realm*, so `ensure_policy` passes.
    fn admin_policy_for(realm: &Realm, user: &User) -> Arc<TestPolicy> {
        let mut user_repo = MockUserRepository::new();
        let u = user.clone();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = u.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let rid = realm.id;
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let rid = rid;
            Box::pin(async move {
                Ok(vec![Role {
                    id: uuid::Uuid::new_v4(),
                    name: "admin".to_string(),
                    description: None,
                    permissions: vec!["manage_realm".to_string()],
                    realm_id: rid,
                    client_id: None,
                    client: None,
                    require_mfa: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }])
            })
        });

        Arc::new(FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(MockClientRepository::new()),
            Arc::new(user_role_repo),
        ))
    }

    #[tokio::test]
    async fn test_get_template_rejects_foreign_realm_template() {
        let attacker_realm = named_realm("attacker-realm");
        let victim_realm = named_realm("victim-realm");
        let attacker = test_user(&attacker_realm);
        let victim_template = test_template(&victim_realm);
        let victim_template_id = victim_template.id;

        // Simulates the realm-scoped SQL: the row only matches for its owning realm.
        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |realm_id, _| {
            let t = victim_template.clone();
            Box::pin(async move {
                Ok(if realm_id == t.realm_id {
                    Some(t)
                } else {
                    None
                })
            })
        });

        let service = EmailTemplateServiceImpl::new(
            Arc::new(realm_repo_for(&attacker_realm)),
            Arc::new(et_repo),
            Arc::new(TestRenderer),
            admin_policy_for(&attacker_realm, &attacker),
        );

        let result = service
            .get_template(
                Identity::User(attacker),
                GetEmailTemplateInput {
                    realm_name: "attacker-realm".to_string(),
                    template_id: victim_template_id,
                },
            )
            .await;

        assert!(
            result.is_err(),
            "reading another realm's email template must be refused"
        );
    }

    #[tokio::test]
    async fn test_update_template_rejects_foreign_realm_template() {
        let attacker_realm = named_realm("attacker-realm");
        let victim_realm = named_realm("victim-realm");
        let attacker = test_user(&attacker_realm);
        let victim_template = test_template(&victim_realm);
        let victim_template_id = victim_template.id;
        let updated = victim_template.clone();

        // Simulates the realm-scoped SQL: the row only matches for its owning realm.
        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |realm_id, _| {
            let t = victim_template.clone();
            Box::pin(async move {
                Ok(if realm_id == t.realm_id {
                    Some(t)
                } else {
                    None
                })
            })
        });
        et_repo.expect_update().returning(move |_, _, _, _, _| {
            let t = updated.clone();
            Box::pin(async move { Ok(t) })
        });

        let service = EmailTemplateServiceImpl::new(
            Arc::new(realm_repo_for(&attacker_realm)),
            Arc::new(et_repo),
            Arc::new(TestRenderer),
            admin_policy_for(&attacker_realm, &attacker),
        );

        let result = service
            .update_template(
                Identity::User(attacker),
                UpdateEmailTemplateInput {
                    realm_name: "attacker-realm".to_string(),
                    template_id: victim_template_id,
                    name: "pwned".to_string(),
                    structure: json!({"type": "root", "children": []}),
                },
            )
            .await;

        assert!(
            result.is_err(),
            "overwriting another realm's email template must be refused"
        );
    }

    #[tokio::test]
    async fn test_delete_template_rejects_foreign_realm_template() {
        let attacker_realm = named_realm("attacker-realm");
        let victim_realm = named_realm("victim-realm");
        let attacker = test_user(&attacker_realm);
        let victim_template = test_template(&victim_realm);
        let victim_template_id = victim_template.id;

        // Simulates the realm-scoped SQL: the row only matches for its owning realm.
        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |realm_id, _| {
            let t = victim_template.clone();
            Box::pin(async move {
                Ok(if realm_id == t.realm_id {
                    Some(t)
                } else {
                    None
                })
            })
        });
        et_repo
            .expect_delete()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = EmailTemplateServiceImpl::new(
            Arc::new(realm_repo_for(&attacker_realm)),
            Arc::new(et_repo),
            Arc::new(TestRenderer),
            admin_policy_for(&attacker_realm, &attacker),
        );

        let result = service
            .delete_template(
                Identity::User(attacker),
                DeleteEmailTemplateInput {
                    realm_name: "attacker-realm".to_string(),
                    template_id: victim_template_id,
                },
            )
            .await;

        assert!(
            result.is_err(),
            "deleting another realm's email template must be refused"
        );
    }

    #[tokio::test]
    async fn test_render_template_html_rejects_foreign_realm_template() {
        let attacker_realm = named_realm("attacker-realm");
        let victim_realm = named_realm("victim-realm");
        let attacker = test_user(&attacker_realm);
        let victim_template = test_template(&victim_realm);
        let victim_template_id = victim_template.id;

        // Simulates the realm-scoped SQL: the row only matches for its owning realm.
        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |realm_id, _| {
            let t = victim_template.clone();
            Box::pin(async move {
                Ok(if realm_id == t.realm_id {
                    Some(t)
                } else {
                    None
                })
            })
        });

        let service = EmailTemplateServiceImpl::new(
            Arc::new(realm_repo_for(&attacker_realm)),
            Arc::new(et_repo),
            Arc::new(TestRenderer),
            admin_policy_for(&attacker_realm, &attacker),
        );

        let result = service
            .render_template_html(
                Identity::User(attacker),
                RenderEmailTemplateInput {
                    realm_name: "attacker-realm".to_string(),
                    template_id: victim_template_id,
                },
            )
            .await;

        assert!(
            result.is_err(),
            "rendering another realm's email template must be refused"
        );
    }

    #[tokio::test]
    async fn test_render_template_html_not_found() {
        let realm = named_realm("test-realm");
        let user = test_user(&realm);

        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo
            .expect_get_by_id()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let service = EmailTemplateServiceImpl::new(
            Arc::new(realm_repo_for(&realm)),
            Arc::new(et_repo),
            Arc::new(TestRenderer),
            admin_policy_for(&realm, &user),
        );

        let result = service
            .render_template_html(
                Identity::User(user),
                RenderEmailTemplateInput {
                    realm_name: "test-realm".to_string(),
                    template_id: uuid::Uuid::new_v4(),
                },
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::EmailTemplateNotFound
        ));
    }
}
