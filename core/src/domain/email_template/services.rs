use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    email_template::{
        entities::EmailTemplate,
        ports::{
            CreateEmailTemplateInput, DeleteEmailTemplateInput, EmailTemplatePolicy,
            EmailTemplateRepository, EmailTemplateService, GetEmailTemplateInput,
            GetEmailTemplatesInput, TemplateRenderer, UpdateEmailTemplateInput,
        },
    },
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

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
            .get_by_id(input.template_id)
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

        // Verify the template exists
        self.email_template_repository
            .get_by_id(input.template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        let mjml = self
            .template_renderer
            .render_to_intermediate(&input.structure)?;

        // Validate that the MJML can be converted to HTML
        self.template_renderer.render_to_html(&mjml)?;

        self.email_template_repository
            .update(input.template_id, input.name, input.structure, mjml)
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

        // Verify the template exists
        self.email_template_repository
            .get_by_id(input.template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        self.email_template_repository
            .delete(input.template_id)
            .await
    }

    async fn render_template_html(&self, template_id: Uuid) -> Result<String, CoreError> {
        let template = self
            .email_template_repository
            .get_by_id(template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        self.template_renderer.render_to_html(&template.mjml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        client::ports::MockClientRepository,
        email_template::{entities::EmailType, ports::MockEmailTemplateRepository},
        realm::{entities::Realm, ports::MockRealmRepository},
        role::entities::Role,
        user::{
            entities::User,
            ports::{MockUserRepository, MockUserRoleRepository},
        },
    };
    use chrono::Utc;
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
    }

    fn test_realm() -> Realm {
        Realm {
            id: uuid::Uuid::new_v4().into(),
            name: "test-realm".to_string(),
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
            firstname: "Admin".to_string(),
            lastname: "User".to_string(),
            email: "admin@test.com".to_string(),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            client_id: None,
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
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
        let realm = test_realm();
        let template = test_template(&realm);
        let template_id = template.id;
        let template_clone = template.clone();

        let realm_repo = MockRealmRepository::new();
        let user_repo = MockUserRepository::new();
        let user_role_repo = MockUserRoleRepository::new();
        let client_repo = MockClientRepository::new();

        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |_| {
            let t = template_clone.clone();
            Box::pin(async move { Ok(Some(t)) })
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

        let result = service.render_template_html(template_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<html><body>Test</body></html>");
    }

    #[tokio::test]
    async fn test_render_template_html_not_found() {
        let realm_repo = MockRealmRepository::new();
        let user_repo = MockUserRepository::new();
        let user_role_repo = MockUserRoleRepository::new();
        let client_repo = MockClientRepository::new();

        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo
            .expect_get_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));

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

        let result = service.render_template_html(uuid::Uuid::new_v4()).await;

        assert!(result.is_err());
        matches!(result.unwrap_err(), CoreError::EmailTemplateNotFound);
    }
}
