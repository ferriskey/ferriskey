use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::ensure_policy;
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::realm::scope::RealmScope;

use super::evaluation::EvaluationRequest;
use super::evaluator::TenantEvaluator;
use super::ports::{
    AuthzSchemaRepository, AuthzStateRepository, EvaluationOutcome, EvaluationService,
    PolicyRepository, PolicyTemplateRepository, PrincipalSliceRepository,
    TenantAuthorizationPolicy, TenantDecisionEngine,
};

const FORBIDDEN: &str = "insufficient permissions";

pub struct EvaluationServiceImpl<RR, P, R, PS, E>
where
    E: TenantDecisionEngine,
{
    realm_repository: Arc<RR>,
    policy: Arc<P>,
    rules: Arc<R>,
    evaluator: TenantEvaluator<R, PS, E>,
}

impl<RR, P, R, PS, E> std::fmt::Debug for EvaluationServiceImpl<RR, P, R, PS, E>
where
    E: TenantDecisionEngine,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvaluationServiceImpl")
            .finish_non_exhaustive()
    }
}

impl<RR, P, R, PS, E> EvaluationServiceImpl<RR, P, R, PS, E>
where
    RR: RealmRepository,
    P: TenantAuthorizationPolicy,
    R: AuthzStateRepository + AuthzSchemaRepository + PolicyRepository + PolicyTemplateRepository,
    PS: PrincipalSliceRepository,
    E: TenantDecisionEngine,
{
    pub fn new(
        realm_repository: Arc<RR>,
        policy: Arc<P>,
        rules: Arc<R>,
        slices: Arc<PS>,
        engine: Arc<E>,
    ) -> Self {
        Self {
            realm_repository,
            policy,
            evaluator: TenantEvaluator::new(Arc::clone(&rules), slices, engine),
            rules,
        }
    }

    async fn policy_names(
        &self,
        scope: &RealmScope,
        ids: Vec<String>,
    ) -> Result<Vec<String>, CoreError> {
        let names: HashMap<String, String> = PolicyRepository::list(self.rules.as_ref(), scope)
            .await?
            .into_iter()
            .map(|policy| (policy.id.to_string(), policy.name.to_string()))
            .collect();
        Ok(ids
            .into_iter()
            .map(|id| names.get(&id).cloned().unwrap_or(id))
            .collect())
    }
}

impl<RR, P, R, PS, E> EvaluationService for EvaluationServiceImpl<RR, P, R, PS, E>
where
    RR: RealmRepository,
    P: TenantAuthorizationPolicy,
    R: AuthzStateRepository + AuthzSchemaRepository + PolicyRepository + PolicyTemplateRepository,
    PS: PrincipalSliceRepository,
    E: TenantDecisionEngine,
{
    async fn evaluate(
        &self,
        identity: Identity,
        realm_name: String,
        request: EvaluationRequest,
    ) -> Result<EvaluationOutcome, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy
                .can_evaluate_authorization(&identity, scope.realm())
                .await,
            FORBIDDEN,
        )?;

        let decision = self
            .evaluator
            .evaluate(&scope, &request, Instant::now())
            .await?;

        let reason_admin = if self
            .policy
            .can_view_authorization(&identity, scope.realm())
            .await?
        {
            Some(
                self.policy_names(&scope, decision.determining_policies)
                    .await?,
            )
        } else {
            None
        };

        Ok(EvaluationOutcome {
            allowed: decision.allowed,
            reason_admin,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use ferriskey_domain::realm::Realm;
    use ferriskey_domain::realm::ports::MockRealmRepository;
    use ferriskey_domain::user::entities::{User, UserConfig};
    use serde_json::Map;
    use uuid::Uuid;

    use super::*;
    use crate::entities::{EntityRef, EntityType, PolicyOrigin};
    use crate::tenant::entities::{Policy, PolicyBody};
    use crate::tenant::evaluation::{Subject, TenantEntity};
    use crate::tenant::fakes::{FakeEngine, FakeRules, FakeSlices};
    use crate::tenant::names::PolicyName;

    struct Governance {
        evaluate: bool,
        view: bool,
    }

    impl TenantAuthorizationPolicy for Governance {
        async fn can_view_authorization(&self, _: &Identity, _: &Realm) -> Result<bool, CoreError> {
            Ok(self.view)
        }

        async fn can_manage_authorization(
            &self,
            _: &Identity,
            _: &Realm,
        ) -> Result<bool, CoreError> {
            Ok(false)
        }

        async fn can_evaluate_authorization(
            &self,
            _: &Identity,
            _: &Realm,
        ) -> Result<bool, CoreError> {
            Ok(self.evaluate)
        }
    }

    type Service =
        EvaluationServiceImpl<MockRealmRepository, Governance, FakeRules, FakeSlices, FakeEngine>;

    fn service(governance: Governance, rules: Arc<FakeRules>, engine: Arc<FakeEngine>) -> Service {
        let mut realms = MockRealmRepository::new();
        realms.expect_get_by_name().returning(|name| {
            let realm = Realm::new(name.to_string());
            Box::pin(async move { Ok(Some(realm)) })
        });
        EvaluationServiceImpl::new(
            Arc::new(realms),
            Arc::new(governance),
            rules,
            Arc::new(FakeSlices { known: true }),
            engine,
        )
    }

    fn identity() -> Identity {
        Identity::User(User::new(UserConfig {
            id: None,
            realm_id: Realm::new("acme".to_string()).id,
            client_id: None,
            username: "pep".to_string(),
            firstname: None,
            lastname: None,
            email: None,
            email_verified: true,
            enabled: true,
        }))
    }

    fn request() -> EvaluationRequest {
        EvaluationRequest {
            subject: Subject::FerrisKey(EntityRef::new(
                EntityType::custom("FerrisKey::User"),
                Uuid::nil().to_string(),
            )),
            action: "document:read".to_string(),
            resource: TenantEntity {
                entity: EntityRef::new(EntityType::custom("App::Document"), "42"),
                properties: Map::new(),
                parents: Vec::new(),
            },
            context: Map::new(),
        }
    }

    fn editors_policy() -> Policy {
        Policy {
            id: Uuid::new_v4(),
            name: PolicyName::try_from("editors-read".to_string()).expect("name"),
            description: None,
            origin: PolicyOrigin::Custom,
            enabled: true,
            body: PolicyBody::Static {
                source: "permit (principal, action, resource);".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn a_caller_that_cannot_evaluate_is_forbidden_before_any_rule_is_read() {
        let rules = Arc::new(FakeRules::default());
        let engine = Arc::new(FakeEngine::default());
        let service = service(
            Governance {
                evaluate: false,
                view: true,
            },
            Arc::clone(&rules),
            Arc::clone(&engine),
        );

        let result = service
            .evaluate(identity(), "acme".to_string(), request())
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "got {result:?}"
        );
        assert_eq!(rules.state_reads.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn a_disabled_realm_is_not_enabled() {
        let rules = Arc::new(FakeRules::default());
        rules.disabled.store(true, Ordering::SeqCst);
        let service = service(
            Governance {
                evaluate: true,
                view: false,
            },
            rules,
            Arc::new(FakeEngine::default()),
        );

        let result = service
            .evaluate(identity(), "acme".to_string(), request())
            .await;

        assert!(
            matches!(result, Err(CoreError::AuthorizationNotEnabled)),
            "got {result:?}"
        );
    }

    #[tokio::test]
    async fn the_reasons_name_the_policies_only_for_a_caller_that_may_view_them() {
        let policy = editors_policy();
        let rules = Arc::new(FakeRules::default());
        *rules.policies.lock().expect("lock") = vec![policy.clone()];
        let engine = Arc::new(FakeEngine::default());
        *engine.determining.lock().expect("lock") = vec![policy.id.to_string()];

        for (view, expected) in [
            (false, None),
            (true, Some(vec!["editors-read".to_string()])),
        ] {
            let service = service(
                Governance {
                    evaluate: true,
                    view,
                },
                Arc::clone(&rules),
                Arc::clone(&engine),
            );

            let outcome = service
                .evaluate(identity(), "acme".to_string(), request())
                .await
                .expect("outcome");

            assert!(outcome.allowed);
            assert_eq!(outcome.reason_admin, expected);
        }
    }
}
