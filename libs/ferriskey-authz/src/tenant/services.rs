use std::sync::Arc;

use chrono::Utc;
use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::{CoreError, PolicyIssue};
use ferriskey_domain::common::policies::ensure_policy;
use ferriskey_domain::generate_uuid_v7;
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::realm::scope::RealmScope;
use uuid::Uuid;

use super::entities::{AuthzSchema, AuthzState, Policy, PolicyTemplate};
use super::ports::{
    AuthzManagementService, AuthzSchemaRepository, AuthzStateRepository, PolicyCompiler,
    PolicyRepository, PolicyTemplateRepository, TenantAuthorizationPolicy,
};
use super::value_objects::{
    CreatePolicyInput, CreateTemplateInput, ReplaceSchemaInput, UpdatePolicyInput,
    UpdateTemplateInput,
};

const FORBIDDEN: &str = "insufficient permissions";

/// The realm's whole rule set, read at one version.
struct RuleSet {
    version: i64,
    schema: Option<AuthzSchema>,
    policies: Vec<Policy>,
    templates: Vec<PolicyTemplate>,
}

#[derive(Clone, Debug)]
pub struct AuthzManagementServiceImpl<R, P, S, SC, PR, T>
where
    R: RealmRepository,
    P: TenantAuthorizationPolicy,
    S: AuthzStateRepository,
    SC: AuthzSchemaRepository,
    PR: PolicyRepository,
    T: PolicyTemplateRepository,
{
    realm_repository: Arc<R>,
    policy: Arc<P>,
    state_repository: Arc<S>,
    schema_repository: Arc<SC>,
    policy_repository: Arc<PR>,
    template_repository: Arc<T>,
    compiler: Arc<dyn PolicyCompiler>,
}

impl<R, P, S, SC, PR, T> AuthzManagementServiceImpl<R, P, S, SC, PR, T>
where
    R: RealmRepository,
    P: TenantAuthorizationPolicy,
    S: AuthzStateRepository,
    SC: AuthzSchemaRepository,
    PR: PolicyRepository,
    T: PolicyTemplateRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        policy: Arc<P>,
        state_repository: Arc<S>,
        schema_repository: Arc<SC>,
        policy_repository: Arc<PR>,
        template_repository: Arc<T>,
        compiler: Arc<dyn PolicyCompiler>,
    ) -> Self {
        Self {
            realm_repository,
            policy,
            state_repository,
            schema_repository,
            policy_repository,
            template_repository,
            compiler,
        }
    }

    async fn resolve(&self, realm_name: &str) -> Result<RealmScope, CoreError> {
        RealmScope::resolve(self.realm_repository.as_ref(), realm_name).await
    }

    /// The version is read first: any write landing after it bumps it, so the
    /// repository refuses ours instead of storing a set nobody validated.
    async fn load(&self, scope: &RealmScope) -> Result<RuleSet, CoreError> {
        let version = self.state_repository.get(scope).await?.version;
        let schema = self.schema_repository.get(scope).await?;
        let policies = self.policy_repository.list(scope).await?;
        let templates = self.template_repository.list(scope).await?;

        Ok(RuleSet {
            version,
            schema,
            policies,
            templates,
        })
    }

    fn validate(
        &self,
        schema: Option<&AuthzSchema>,
        policies: Vec<Policy>,
        templates: &[PolicyTemplate],
    ) -> Result<(), CoreError> {
        let enabled: Vec<Policy> = policies.into_iter().filter(|p| p.enabled).collect();

        self.compiler
            .validate_policies(schema, &enabled, templates)
            .map_err(CoreError::InvalidAuthorizationPolicy)
    }
}

fn unknown_template(policy: &Policy) -> CoreError {
    CoreError::InvalidAuthorizationPolicy(vec![PolicyIssue {
        policy: Some(policy.name.to_string()),
        message: "links a template that does not exist".to_string(),
        line: None,
        column: None,
    }])
}

fn ensure_template_exists(policy: &Policy, templates: &[PolicyTemplate]) -> Result<(), CoreError> {
    match policy.body.template_id() {
        Some(id) if !templates.iter().any(|t| t.id == id) => Err(unknown_template(policy)),
        _ => Ok(()),
    }
}

impl<R, P, S, SC, PR, T> AuthzManagementService for AuthzManagementServiceImpl<R, P, S, SC, PR, T>
where
    R: RealmRepository,
    P: TenantAuthorizationPolicy,
    S: AuthzStateRepository,
    SC: AuthzSchemaRepository,
    PR: PolicyRepository,
    T: PolicyTemplateRepository,
{
    async fn get_state(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<AuthzState, CoreError> {
        let scope = self.resolve(&realm_name).await?;

        ensure_policy(
            self.policy
                .can_view_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        self.state_repository.get(&scope).await
    }

    async fn set_enabled(
        &self,
        identity: Identity,
        realm_name: String,
        enabled: bool,
    ) -> Result<AuthzState, CoreError> {
        let scope = self.resolve(&realm_name).await?;

        ensure_policy(
            self.policy
                .can_manage_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        self.state_repository.set_enabled(&scope, enabled).await
    }

    async fn get_schema(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Option<AuthzSchema>, CoreError> {
        let scope = self.resolve(&realm_name).await?;

        ensure_policy(
            self.policy
                .can_view_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        self.schema_repository.get(&scope).await
    }

    async fn replace_schema(
        &self,
        identity: Identity,
        input: ReplaceSchemaInput,
    ) -> Result<AuthzSchema, CoreError> {
        let scope = self.resolve(&input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_manage_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        let rules = self.load(&scope).await?;
        let schema = AuthzSchema {
            id: rules
                .schema
                .as_ref()
                .map_or_else(generate_uuid_v7, |current| current.id),
            namespace: input.namespace,
            source: input.source,
            updated_at: Utc::now(),
        };

        self.compiler
            .validate_schema(&schema)
            .map_err(CoreError::InvalidAuthorizationPolicy)?;
        self.validate(Some(&schema), rules.policies, &rules.templates)?;

        self.schema_repository
            .replace(&scope, schema, rules.version)
            .await
    }

    async fn list_policies(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<Policy>, CoreError> {
        let scope = self.resolve(&realm_name).await?;

        ensure_policy(
            self.policy
                .can_view_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        self.policy_repository.list(&scope).await
    }

    async fn get_policy(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> Result<Policy, CoreError> {
        let scope = self.resolve(&realm_name).await?;

        ensure_policy(
            self.policy
                .can_view_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        self.policy_repository
            .list(&scope)
            .await?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or(CoreError::NotFound)
    }

    async fn create_policy(
        &self,
        identity: Identity,
        input: CreatePolicyInput,
    ) -> Result<Policy, CoreError> {
        let scope = self.resolve(&input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_manage_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        let mut rules = self.load(&scope).await?;
        if rules.policies.iter().any(|p| p.name == input.name) {
            return Err(CoreError::AlreadyExists);
        }

        let policy = Policy {
            id: generate_uuid_v7(),
            name: input.name,
            description: input.description,
            origin: input.body.origin(),
            enabled: input.enabled,
            body: input.body,
        };
        ensure_template_exists(&policy, &rules.templates)?;

        rules.policies.push(policy.clone());
        self.validate(rules.schema.as_ref(), rules.policies, &rules.templates)?;

        self.policy_repository
            .save(&scope, policy, rules.version)
            .await
    }

    async fn update_policy(
        &self,
        identity: Identity,
        input: UpdatePolicyInput,
    ) -> Result<Policy, CoreError> {
        let scope = self.resolve(&input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_manage_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        let mut rules = self.load(&scope).await?;
        let index = rules
            .policies
            .iter()
            .position(|p| p.id == input.id)
            .ok_or(CoreError::NotFound)?;
        if rules
            .policies
            .iter()
            .any(|p| p.id != input.id && p.name == input.name)
        {
            return Err(CoreError::AlreadyExists);
        }

        let policy = Policy {
            id: input.id,
            name: input.name,
            description: input.description,
            origin: input.body.origin(),
            enabled: input.enabled,
            body: input.body,
        };
        ensure_template_exists(&policy, &rules.templates)?;

        rules.policies[index] = policy.clone();
        self.validate(rules.schema.as_ref(), rules.policies, &rules.templates)?;

        self.policy_repository
            .save(&scope, policy, rules.version)
            .await
    }

    async fn delete_policy(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let scope = self.resolve(&realm_name).await?;

        ensure_policy(
            self.policy
                .can_manage_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        let rules = self.load(&scope).await?;
        if !rules.policies.iter().any(|p| p.id == id) {
            return Err(CoreError::NotFound);
        }

        self.policy_repository
            .delete(&scope, id, rules.version)
            .await
    }

    async fn list_templates(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<PolicyTemplate>, CoreError> {
        let scope = self.resolve(&realm_name).await?;

        ensure_policy(
            self.policy
                .can_view_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        self.template_repository.list(&scope).await
    }

    async fn create_template(
        &self,
        identity: Identity,
        input: CreateTemplateInput,
    ) -> Result<PolicyTemplate, CoreError> {
        let scope = self.resolve(&input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_manage_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        let mut rules = self.load(&scope).await?;
        if rules.templates.iter().any(|t| t.name == input.name) {
            return Err(CoreError::AlreadyExists);
        }

        let template = PolicyTemplate {
            id: generate_uuid_v7(),
            name: input.name,
            description: input.description,
            source: input.source,
        };

        rules.templates.push(template.clone());
        self.validate(rules.schema.as_ref(), rules.policies, &rules.templates)?;

        self.template_repository
            .save(&scope, template, rules.version)
            .await
    }

    async fn update_template(
        &self,
        identity: Identity,
        input: UpdateTemplateInput,
    ) -> Result<PolicyTemplate, CoreError> {
        let scope = self.resolve(&input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_manage_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        let mut rules = self.load(&scope).await?;
        let index = rules
            .templates
            .iter()
            .position(|t| t.id == input.id)
            .ok_or(CoreError::NotFound)?;
        if rules
            .templates
            .iter()
            .any(|t| t.id != input.id && t.name == input.name)
        {
            return Err(CoreError::AlreadyExists);
        }

        let template = PolicyTemplate {
            id: input.id,
            name: input.name,
            description: input.description,
            source: input.source,
        };

        rules.templates[index] = template.clone();
        self.validate(rules.schema.as_ref(), rules.policies, &rules.templates)?;

        self.template_repository
            .save(&scope, template, rules.version)
            .await
    }

    async fn delete_template(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let scope = self.resolve(&realm_name).await?;

        ensure_policy(
            self.policy
                .can_manage_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        let rules = self.load(&scope).await?;
        if !rules.templates.iter().any(|t| t.id == id) {
            return Err(CoreError::NotFound);
        }
        if rules
            .policies
            .iter()
            .any(|p| p.body.template_id() == Some(id))
        {
            return Err(CoreError::AuthorizationTemplateInUse);
        }

        self.template_repository
            .delete(&scope, id, rules.version)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::Utc;
    use ferriskey_domain::realm::Realm;
    use ferriskey_domain::realm::ports::MockRealmRepository;
    use ferriskey_domain::user::entities::{User, UserConfig};

    use super::*;
    use crate::entities::EntityRef;
    use crate::tenant::entities::PolicyBody;
    use crate::tenant::names::{Namespace, PolicyName, SlotId};
    use crate::tenant::ports::{
        MockAuthzSchemaRepository, MockAuthzStateRepository, MockPolicyCompiler,
        MockPolicyRepository, MockPolicyTemplateRepository,
    };

    const VERSION: i64 = 7;

    struct Governance {
        view: bool,
        manage: bool,
    }

    impl TenantAuthorizationPolicy for Governance {
        async fn can_view_authorization(
            &self,
            _identity: &Identity,
            _target_realm: &Realm,
        ) -> Result<bool, CoreError> {
            Ok(self.view)
        }

        async fn can_manage_authorization(
            &self,
            _identity: &Identity,
            _target_realm: &Realm,
        ) -> Result<bool, CoreError> {
            Ok(self.manage)
        }
    }

    type Service = AuthzManagementServiceImpl<
        MockRealmRepository,
        Governance,
        MockAuthzStateRepository,
        MockAuthzSchemaRepository,
        MockPolicyRepository,
        MockPolicyTemplateRepository,
    >;

    /// Mocks for one test. Every repository answers reads with `policies` and
    /// `templates`; writes are not expected unless a test sets them up.
    struct Fixture {
        governance: Governance,
        schema: Option<AuthzSchema>,
        policies: Vec<Policy>,
        templates: Vec<PolicyTemplate>,
        state: MockAuthzStateRepository,
        schemas: MockAuthzSchemaRepository,
        policy_repo: MockPolicyRepository,
        template_repo: MockPolicyTemplateRepository,
        compiler: MockPolicyCompiler,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                governance: Governance {
                    view: true,
                    manage: true,
                },
                schema: None,
                policies: Vec::new(),
                templates: Vec::new(),
                state: MockAuthzStateRepository::new(),
                schemas: MockAuthzSchemaRepository::new(),
                policy_repo: MockPolicyRepository::new(),
                template_repo: MockPolicyTemplateRepository::new(),
                compiler: MockPolicyCompiler::new(),
            }
        }

        fn build(mut self) -> Service {
            let mut realms = MockRealmRepository::new();
            realms.expect_get_by_name().returning(|name| {
                let realm = Realm::new(name.to_string());
                Box::pin(async move { Ok(Some(realm)) })
            });

            self.state.expect_get().returning(|_| {
                Box::pin(async {
                    Ok(AuthzState {
                        enabled: true,
                        version: VERSION,
                    })
                })
            });
            let schema = self.schema;
            self.schemas.expect_get().returning(move |_| {
                let schema = schema.clone();
                Box::pin(async move { Ok(schema) })
            });
            let policies = self.policies;
            self.policy_repo.expect_list().returning(move |_| {
                let policies = policies.clone();
                Box::pin(async move { Ok(policies) })
            });
            let templates = self.templates;
            self.template_repo.expect_list().returning(move |_| {
                let templates = templates.clone();
                Box::pin(async move { Ok(templates) })
            });

            AuthzManagementServiceImpl::new(
                Arc::new(realms),
                Arc::new(self.governance),
                Arc::new(self.state),
                Arc::new(self.schemas),
                Arc::new(self.policy_repo),
                Arc::new(self.template_repo),
                Arc::new(self.compiler),
            )
        }
    }

    fn identity() -> Identity {
        Identity::User(User::new(UserConfig {
            id: None,
            realm_id: Realm::new("acme".to_string()).id,
            client_id: None,
            username: "admin".to_string(),
            firstname: None,
            lastname: None,
            email: None,
            email_verified: true,
            enabled: true,
        }))
    }

    fn name(value: &str) -> PolicyName {
        PolicyName::try_from(value.to_string()).expect("valid policy name")
    }

    fn static_policy(policy_name: &str, enabled: bool) -> Policy {
        Policy {
            id: generate_uuid_v7(),
            name: name(policy_name),
            description: None,
            origin: crate::entities::PolicyOrigin::Custom,
            enabled,
            body: PolicyBody::Static {
                source: "permit(principal, action, resource);".to_string(),
            },
        }
    }

    fn template(template_name: &str) -> PolicyTemplate {
        PolicyTemplate {
            id: generate_uuid_v7(),
            name: name(template_name),
            description: None,
            source: "permit(principal == ?principal, action, resource);".to_string(),
        }
    }

    fn linked_to(template: &PolicyTemplate, policy_name: &str) -> Policy {
        Policy {
            id: generate_uuid_v7(),
            name: name(policy_name),
            description: None,
            origin: crate::entities::PolicyOrigin::Generated,
            enabled: true,
            body: PolicyBody::Linked {
                template_id: template.id,
                links: BTreeMap::from([(
                    SlotId::Principal,
                    EntityRef::new(crate::entities::EntityType::custom("FerrisKey::Role"), "r1"),
                )]),
            },
        }
    }

    fn schema() -> AuthzSchema {
        AuthzSchema {
            id: generate_uuid_v7(),
            namespace: Namespace::default(),
            source: "entity Document;".to_string(),
            updated_at: Utc::now(),
        }
    }

    fn issue(policy: &str) -> PolicyIssue {
        PolicyIssue {
            policy: Some(policy.to_string()),
            message: "unknown entity type App::Folder".to_string(),
            line: Some(1),
            column: Some(14),
        }
    }

    fn create_input(policy_name: &str, enabled: bool) -> CreatePolicyInput {
        CreatePolicyInput {
            realm_name: "acme".to_string(),
            name: name(policy_name),
            description: None,
            enabled,
            body: PolicyBody::Static {
                source: "permit(principal, action, resource);".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn a_policy_the_compiler_rejects_is_not_stored() {
        let mut f = Fixture::new();
        f.compiler
            .expect_validate_policies()
            .returning(|_, _, _| Err(vec![issue("bad")]));
        f.policy_repo.expect_save().never();

        let result = f
            .build()
            .create_policy(identity(), create_input("bad", true))
            .await;

        match result {
            Err(CoreError::InvalidAuthorizationPolicy(issues)) => {
                assert_eq!(issues, vec![issue("bad")]);
            }
            other => panic!("expected InvalidAuthorizationPolicy, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn a_valid_policy_is_stored_at_the_version_that_was_read() {
        let mut f = Fixture::new();
        f.compiler
            .expect_validate_policies()
            .returning(|_, _, _| Ok(()));
        f.policy_repo
            .expect_save()
            .withf(|_, policy, expected_version| {
                policy.name.as_str() == "ok"
                    && policy.origin == crate::entities::PolicyOrigin::Custom
                    && *expected_version == VERSION
            })
            .times(1)
            .returning(|_, policy, _| Box::pin(async move { Ok(policy) }));

        let stored = f
            .build()
            .create_policy(identity(), create_input("ok", true))
            .await
            .expect("policy stored");

        assert_eq!(stored.name.as_str(), "ok");
    }

    #[tokio::test]
    async fn enabling_a_policy_validates_it_with_every_enabled_policy() {
        let enabled = static_policy("already-on", true);
        let draft = static_policy("draft", false);
        let (enabled_id, draft_id) = (enabled.id, draft.id);

        let mut f = Fixture::new();
        f.policies = vec![enabled, draft, static_policy("still-off", false)];
        f.compiler
            .expect_validate_policies()
            .withf(move |_, policies, _| {
                let mut ids: Vec<Uuid> = policies.iter().map(|p| p.id).collect();
                ids.sort();
                let mut expected = vec![enabled_id, draft_id];
                expected.sort();
                ids == expected
            })
            .times(1)
            .returning(|_, _, _| Ok(()));
        f.policy_repo
            .expect_save()
            .returning(|_, policy, _| Box::pin(async move { Ok(policy) }));

        let updated = f
            .build()
            .update_policy(
                identity(),
                UpdatePolicyInput {
                    realm_name: "acme".to_string(),
                    id: draft_id,
                    name: name("draft"),
                    description: None,
                    enabled: true,
                    body: PolicyBody::Static {
                        source: "permit(principal, action, resource);".to_string(),
                    },
                },
            )
            .await
            .expect("policy enabled");

        assert!(updated.enabled);
    }

    #[tokio::test]
    async fn a_schema_that_breaks_an_enabled_policy_is_refused_and_names_it() {
        let mut f = Fixture::new();
        f.schema = Some(schema());
        f.policies = vec![static_policy("folder-owners", true)];
        f.compiler.expect_validate_schema().returning(|_| Ok(()));
        f.compiler
            .expect_validate_policies()
            .withf(|schema, _, _| schema.is_some_and(|s| s.source == "entity Report;"))
            .returning(|_, _, _| Err(vec![issue("folder-owners")]));
        f.schemas.expect_replace().never();

        let result = f
            .build()
            .replace_schema(
                identity(),
                ReplaceSchemaInput {
                    realm_name: "acme".to_string(),
                    namespace: Namespace::default(),
                    source: "entity Report;".to_string(),
                },
            )
            .await;

        match result {
            Err(CoreError::InvalidAuthorizationPolicy(issues)) => {
                assert_eq!(issues[0].policy.as_deref(), Some("folder-owners"));
            }
            other => panic!("expected InvalidAuthorizationPolicy, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn replacing_the_schema_keeps_its_id() {
        let current = schema();
        let current_id = current.id;

        let mut f = Fixture::new();
        f.schema = Some(current);
        f.compiler.expect_validate_schema().returning(|_| Ok(()));
        f.compiler
            .expect_validate_policies()
            .returning(|_, _, _| Ok(()));
        f.schemas
            .expect_replace()
            .withf(move |_, schema, expected_version| {
                schema.id == current_id && *expected_version == VERSION
            })
            .times(1)
            .returning(|_, schema, _| Box::pin(async move { Ok(schema) }));

        f.build()
            .replace_schema(
                identity(),
                ReplaceSchemaInput {
                    realm_name: "acme".to_string(),
                    namespace: Namespace::default(),
                    source: "entity Report;".to_string(),
                },
            )
            .await
            .expect("schema replaced");
    }

    #[tokio::test]
    async fn without_manage_authorization_nothing_is_written() {
        let mut f = Fixture::new();
        f.governance = Governance {
            view: true,
            manage: false,
        };
        f.policy_repo.expect_save().never();
        f.schemas.expect_replace().never();
        f.template_repo.expect_delete().never();
        f.state.expect_set_enabled().never();
        let service = f.build();

        let results = [
            service
                .create_policy(identity(), create_input("p", true))
                .await
                .map(|_| ()),
            service
                .replace_schema(
                    identity(),
                    ReplaceSchemaInput {
                        realm_name: "acme".to_string(),
                        namespace: Namespace::default(),
                        source: String::new(),
                    },
                )
                .await
                .map(|_| ()),
            service
                .delete_template(identity(), "acme".to_string(), Uuid::nil())
                .await,
            service
                .set_enabled(identity(), "acme".to_string(), true)
                .await
                .map(|_| ()),
        ];

        for result in results {
            assert!(
                matches!(result, Err(CoreError::Forbidden(_))),
                "expected Forbidden, got {result:?}"
            );
        }
    }

    #[tokio::test]
    async fn without_view_authorization_nothing_is_read() {
        let mut f = Fixture::new();
        f.governance = Governance {
            view: false,
            manage: false,
        };
        let service = f.build();

        assert!(matches!(
            service.list_policies(identity(), "acme".to_string()).await,
            Err(CoreError::Forbidden(_))
        ));
        assert!(matches!(
            service.get_schema(identity(), "acme".to_string()).await,
            Err(CoreError::Forbidden(_))
        ));
        assert!(matches!(
            service.get_state(identity(), "acme".to_string()).await,
            Err(CoreError::Forbidden(_))
        ));
    }

    #[tokio::test]
    async fn a_template_still_linked_by_a_policy_cannot_be_deleted() {
        let linked = template("owner-can-read");
        let linked_id = linked.id;

        let mut f = Fixture::new();
        f.policies = vec![linked_to(&linked, "alice-reads")];
        f.templates = vec![linked];
        f.template_repo.expect_delete().never();

        let result = f
            .build()
            .delete_template(identity(), "acme".to_string(), linked_id)
            .await;

        assert!(matches!(result, Err(CoreError::AuthorizationTemplateInUse)));
    }

    #[tokio::test]
    async fn a_policy_cannot_link_a_template_that_does_not_exist() {
        let missing = template("never-stored");

        let mut f = Fixture::new();
        f.policy_repo.expect_save().never();

        let result = f
            .build()
            .create_policy(
                identity(),
                CreatePolicyInput {
                    realm_name: "acme".to_string(),
                    name: name("dangling"),
                    description: None,
                    enabled: true,
                    body: linked_to(&missing, "dangling").body,
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(CoreError::InvalidAuthorizationPolicy(_))
        ));
    }

    #[tokio::test]
    async fn two_policies_cannot_share_a_name() {
        let mut f = Fixture::new();
        f.policies = vec![static_policy("taken", false)];
        f.policy_repo.expect_save().never();

        let result = f
            .build()
            .create_policy(identity(), create_input("taken", false))
            .await;

        assert!(matches!(result, Err(CoreError::AlreadyExists)));
    }

    #[tokio::test]
    async fn a_policy_of_another_realm_is_not_found() {
        let result = Fixture::new()
            .build()
            .get_policy(identity(), "acme".to_string(), generate_uuid_v7())
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
