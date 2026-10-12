use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::{Duration, Instant};

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::scope::RealmScope;
use tracing::error;
use uuid::Uuid;

use super::evaluation::{EvaluationRequest, Subject, load_rule_set};
use super::ports::{
    AuthzSchemaRepository, AuthzStateRepository, PolicyRepository, PolicyTemplateRepository,
    PrincipalSliceRepository, TenantDecisionEngine,
};
use crate::entities::Decision;

pub const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(1);

struct Compiled<C> {
    version: i64,
    checked_at: Instant,
    artefact: Arc<C>,
}

impl<C> Clone for Compiled<C> {
    fn clone(&self) -> Self {
        Self {
            version: self.version,
            checked_at: self.checked_at,
            artefact: Arc::clone(&self.artefact),
        }
    }
}

pub struct TenantEvaluator<R, PS, E>
where
    E: TenantDecisionEngine,
{
    rules: Arc<R>,
    slices: Arc<PS>,
    engine: Arc<E>,
    cache: RwLock<HashMap<Uuid, Compiled<E::Compiled>>>,
}

impl<R, PS, E> TenantEvaluator<R, PS, E>
where
    R: AuthzStateRepository + AuthzSchemaRepository + PolicyRepository + PolicyTemplateRepository,
    PS: PrincipalSliceRepository,
    E: TenantDecisionEngine,
{
    pub fn new(rules: Arc<R>, slices: Arc<PS>, engine: Arc<E>) -> Self {
        Self {
            rules,
            slices,
            engine,
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn evaluate(
        &self,
        scope: &RealmScope,
        request: &EvaluationRequest,
        now: Instant,
    ) -> Result<Decision, CoreError> {
        let Some(compiled) = self.compiled(scope, now).await? else {
            return Ok(Decision::deny());
        };

        let slice = match &request.subject {
            Subject::FerrisKey(principal) => match self.slices.load(scope, principal).await? {
                Some(slice) => Some(slice),
                None => return Ok(Decision::deny()),
            },
            Subject::Tenant(_) => None,
        };

        self.engine.decide(&compiled, slice.as_ref(), request)
    }

    fn cached(&self, realm_id: Uuid) -> Option<Compiled<E::Compiled>> {
        self.cache
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&realm_id)
            .cloned()
    }

    fn store(&self, realm_id: Uuid, compiled: Compiled<E::Compiled>) {
        self.cache
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(realm_id, compiled);
    }

    async fn compiled(
        &self,
        scope: &RealmScope,
        now: Instant,
    ) -> Result<Option<Arc<E::Compiled>>, CoreError> {
        let realm_id = Uuid::from(scope.id());
        let cached = self.cached(realm_id);

        if let Some(entry) = &cached
            && now.saturating_duration_since(entry.checked_at) < VERSION_CHECK_INTERVAL
        {
            return Ok(Some(Arc::clone(&entry.artefact)));
        }

        let version = AuthzStateRepository::get(self.rules.as_ref(), scope)
            .await?
            .version;

        if let Some(entry) = cached.as_ref().filter(|entry| entry.version == version) {
            let artefact = Arc::clone(&entry.artefact);
            self.store(
                realm_id,
                Compiled {
                    version,
                    checked_at: now,
                    artefact: Arc::clone(&artefact),
                },
            );
            return Ok(Some(artefact));
        }

        let rules = load_rule_set(
            scope,
            version,
            self.rules.as_ref(),
            self.rules.as_ref(),
            self.rules.as_ref(),
        )
        .await?;

        let artefact = match self.engine.compile(&rules) {
            Ok(compiled) => Arc::new(compiled),
            Err(issues) => {
                error!(
                    realm_id = %realm_id,
                    version,
                    issues = issues.len(),
                    "authorization rules failed to compile, keeping the previous ones"
                );
                match cached {
                    Some(previous) => previous.artefact,
                    None => return Ok(None),
                }
            }
        };

        self.store(
            realm_id,
            Compiled {
                version,
                checked_at: now,
                artefact: Arc::clone(&artefact),
            },
        );
        Ok(Some(artefact))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};

    use ferriskey_domain::common::app_errors::PolicyIssue;
    use ferriskey_domain::realm::Realm;
    use serde_json::Map;

    use super::*;
    use crate::entities::{EntityRef, EntityType};
    use crate::tenant::entities::{AuthzSchema, AuthzState, Policy, PolicyTemplate};
    use crate::tenant::evaluation::{
        ClientFacts, PrincipalFacts, PrincipalSlice, RuleSet, TenantEntity,
    };

    #[derive(Default)]
    struct FakeRules {
        version: AtomicI64,
        state_reads: AtomicUsize,
    }

    impl AuthzStateRepository for FakeRules {
        async fn get(&self, _: &RealmScope) -> Result<AuthzState, CoreError> {
            self.state_reads.fetch_add(1, Ordering::SeqCst);
            Ok(AuthzState {
                enabled: true,
                version: self.version.load(Ordering::SeqCst),
            })
        }

        async fn set_enabled(&self, _: &RealmScope, _: bool) -> Result<AuthzState, CoreError> {
            unreachable!()
        }
    }

    impl AuthzSchemaRepository for FakeRules {
        async fn get(&self, _: &RealmScope) -> Result<Option<AuthzSchema>, CoreError> {
            Ok(None)
        }

        async fn replace(
            &self,
            _: &RealmScope,
            _: AuthzSchema,
            _: i64,
        ) -> Result<AuthzSchema, CoreError> {
            unreachable!()
        }
    }

    impl PolicyRepository for FakeRules {
        async fn list(&self, _: &RealmScope) -> Result<Vec<Policy>, CoreError> {
            Ok(Vec::new())
        }

        async fn save(&self, _: &RealmScope, _: Policy, _: i64) -> Result<Policy, CoreError> {
            unreachable!()
        }

        async fn delete(&self, _: &RealmScope, _: Uuid, _: i64) -> Result<(), CoreError> {
            unreachable!()
        }
    }

    impl PolicyTemplateRepository for FakeRules {
        async fn list(&self, _: &RealmScope) -> Result<Vec<PolicyTemplate>, CoreError> {
            Ok(Vec::new())
        }

        async fn save(
            &self,
            _: &RealmScope,
            _: PolicyTemplate,
            _: i64,
        ) -> Result<PolicyTemplate, CoreError> {
            unreachable!()
        }

        async fn delete(&self, _: &RealmScope, _: Uuid, _: i64) -> Result<(), CoreError> {
            unreachable!()
        }
    }

    #[derive(Default)]
    struct FakeSlices {
        known: bool,
    }

    impl PrincipalSliceRepository for FakeSlices {
        async fn load(
            &self,
            _: &RealmScope,
            principal: &EntityRef,
        ) -> Result<Option<PrincipalSlice>, CoreError> {
            Ok(self.known.then(|| PrincipalSlice {
                principal: PrincipalFacts::Client(ClientFacts {
                    id: Uuid::nil(),
                    client_id: principal.id.clone(),
                    role_ids: Vec::new(),
                }),
                roles: Vec::new(),
                groups: Vec::new(),
                organizations: Vec::new(),
            }))
        }
    }

    #[derive(Default)]
    struct FakeEngine {
        compilations: AtomicUsize,
        failing: AtomicBool,
        decided_with: Mutex<Vec<(i64, bool)>>,
    }

    impl TenantDecisionEngine for FakeEngine {
        type Compiled = i64;

        fn compile(&self, rules: &RuleSet) -> Result<i64, Vec<PolicyIssue>> {
            self.compilations.fetch_add(1, Ordering::SeqCst);
            if self.failing.load(Ordering::SeqCst) {
                return Err(vec![PolicyIssue {
                    policy: None,
                    message: "broken".to_string(),
                    line: None,
                    column: None,
                }]);
            }
            Ok(rules.version)
        }

        fn decide(
            &self,
            compiled: &i64,
            slice: Option<&PrincipalSlice>,
            _: &EvaluationRequest,
        ) -> Result<Decision, CoreError> {
            self.decided_with
                .lock()
                .expect("lock")
                .push((*compiled, slice.is_some()));
            Ok(Decision::allow())
        }
    }

    struct Fixture {
        rules: Arc<FakeRules>,
        engine: Arc<FakeEngine>,
        evaluator: TenantEvaluator<FakeRules, FakeSlices, FakeEngine>,
        scope: RealmScope,
        start: Instant,
    }

    fn fixture(known_subject: bool) -> Fixture {
        let rules = Arc::new(FakeRules::default());
        let engine = Arc::new(FakeEngine::default());
        let evaluator = TenantEvaluator::new(
            Arc::clone(&rules),
            Arc::new(FakeSlices {
                known: known_subject,
            }),
            Arc::clone(&engine),
        );
        Fixture {
            rules,
            engine,
            evaluator,
            scope: RealmScope::from_realm(Realm::new("acme".to_string())),
            start: Instant::now(),
        }
    }

    fn document() -> TenantEntity {
        TenantEntity {
            entity: EntityRef::new(EntityType::custom("App::Document"), "42"),
            properties: Map::new(),
            parents: Vec::new(),
        }
    }

    fn ferriskey_subject() -> EvaluationRequest {
        EvaluationRequest {
            subject: Subject::FerrisKey(EntityRef::new(
                EntityType::custom("FerrisKey::Client"),
                "svc",
            )),
            action: "document:read".to_string(),
            resource: document(),
            context: Map::new(),
        }
    }

    fn tenant_subject() -> EvaluationRequest {
        EvaluationRequest {
            subject: Subject::Tenant(TenantEntity {
                entity: EntityRef::new(EntityType::custom("App::Device"), "d1"),
                properties: Map::new(),
                parents: Vec::new(),
            }),
            ..ferriskey_subject()
        }
    }

    impl Fixture {
        async fn evaluate_at(&self, offset: Duration) -> Decision {
            self.evaluator
                .evaluate(&self.scope, &ferriskey_subject(), self.start + offset)
                .await
                .expect("decision")
        }

        fn compilations(&self) -> usize {
            self.engine.compilations.load(Ordering::SeqCst)
        }

        fn last_artefact(&self) -> Option<i64> {
            self.engine
                .decided_with
                .lock()
                .expect("lock")
                .last()
                .map(|(artefact, _)| *artefact)
        }
    }

    #[tokio::test]
    async fn within_the_interval_the_version_is_not_read_again() {
        let f = fixture(true);
        f.evaluate_at(Duration::ZERO).await;
        f.evaluate_at(VERSION_CHECK_INTERVAL / 2).await;

        assert_eq!(f.rules.state_reads.load(Ordering::SeqCst), 1);
        assert_eq!(f.compilations(), 1);
    }

    #[tokio::test]
    async fn an_unchanged_version_is_not_recompiled() {
        let f = fixture(true);
        f.evaluate_at(Duration::ZERO).await;
        f.evaluate_at(VERSION_CHECK_INTERVAL * 2).await;

        assert_eq!(f.rules.state_reads.load(Ordering::SeqCst), 2);
        assert_eq!(f.compilations(), 1);
    }

    #[tokio::test]
    async fn a_new_version_is_compiled_once_the_interval_elapsed() {
        let f = fixture(true);
        f.evaluate_at(Duration::ZERO).await;
        f.rules.version.store(3, Ordering::SeqCst);
        f.evaluate_at(VERSION_CHECK_INTERVAL / 2).await;
        assert_eq!(f.last_artefact(), Some(0));

        f.evaluate_at(VERSION_CHECK_INTERVAL * 2).await;
        assert_eq!(f.compilations(), 2);
        assert_eq!(f.last_artefact(), Some(3));
    }

    #[tokio::test]
    async fn rules_that_fail_to_compile_keep_the_previous_ones() {
        let f = fixture(true);
        f.evaluate_at(Duration::ZERO).await;
        f.rules.version.store(1, Ordering::SeqCst);
        f.engine.failing.store(true, Ordering::SeqCst);

        let decision = f.evaluate_at(VERSION_CHECK_INTERVAL * 2).await;

        assert!(decision.allowed);
        assert_eq!(f.last_artefact(), Some(0));
    }

    #[tokio::test]
    async fn rules_that_never_compiled_deny() {
        let f = fixture(true);
        f.engine.failing.store(true, Ordering::SeqCst);

        let decision = f.evaluate_at(Duration::ZERO).await;

        assert!(!decision.allowed);
        assert_eq!(f.last_artefact(), None);
    }

    #[tokio::test]
    async fn an_unknown_ferriskey_subject_is_denied_without_asking_the_engine() {
        let f = fixture(false);

        let decision = f.evaluate_at(Duration::ZERO).await;

        assert!(!decision.allowed);
        assert_eq!(f.last_artefact(), None);
    }

    #[tokio::test]
    async fn a_tenant_subject_is_decided_without_a_slice() {
        let f = fixture(false);

        let decision = f
            .evaluator
            .evaluate(&f.scope, &tenant_subject(), f.start)
            .await
            .expect("decision");

        assert!(decision.allowed);
        assert_eq!(
            f.engine.decided_with.lock().expect("lock").as_slice(),
            &[(0, false)]
        );
    }
}
