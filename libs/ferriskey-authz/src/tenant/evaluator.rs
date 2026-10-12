use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::{Duration, Instant};

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::scope::RealmScope;
use tracing::error;
use uuid::Uuid;

use super::FERRISKEY_NAMESPACE;
use super::evaluation::{EvaluationRequest, Subject, load_rule_set};
use super::ports::{
    AuthzSchemaRepository, AuthzStateRepository, PolicyRepository, PolicyTemplateRepository,
    PrincipalSliceRepository, TenantDecisionEngine,
};
use crate::entities::{Decision, EntityRef};

pub const VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(1);

struct Compiled<C> {
    version: i64,
    checked_at: Instant,
    artefact: Option<Arc<C>>,
}

impl<C> Clone for Compiled<C> {
    fn clone(&self) -> Self {
        Self {
            version: self.version,
            checked_at: self.checked_at,
            artefact: self.artefact.clone(),
        }
    }
}

fn ensure_tenant_types(request: &EvaluationRequest) -> Result<(), CoreError> {
    let reserved = format!("{FERRISKEY_NAMESPACE}::");
    let is_reserved = |entity: &EntityRef| entity.ty.as_str().starts_with(&reserved);
    let refuse = |what: String| Err(CoreError::InvalidAuthorizationRequest(what));

    let tenant_entities = match &request.subject {
        Subject::Tenant(subject) => vec![&request.resource, subject],
        Subject::FerrisKey(_) => vec![&request.resource],
    };
    if let Some(entity) = tenant_entities.iter().find(|e| is_reserved(&e.entity)) {
        return refuse(format!(
            "{} is resolved by FerrisKey and cannot be described by the request",
            entity.entity.ty
        ));
    }
    if let Subject::Tenant(subject) = &request.subject
        && let Some(parent) = subject.parents.iter().find(|p| is_reserved(p))
    {
        return refuse(format!(
            "membership in {} is resolved by FerrisKey and cannot be claimed by the request",
            parent.ty
        ));
    }
    Ok(())
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
        ensure_tenant_types(request)?;

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

    fn forget(&self, realm_id: Uuid) {
        self.cache
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&realm_id);
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
            return Ok(entry.artefact.clone());
        }

        let state = AuthzStateRepository::get(self.rules.as_ref(), scope).await?;
        if !state.enabled {
            self.forget(realm_id);
            return Err(CoreError::AuthorizationNotEnabled);
        }
        let version = state.version;

        if let Some(entry) = cached.as_ref().filter(|entry| entry.version == version) {
            self.store(
                realm_id,
                Compiled {
                    version,
                    checked_at: now,
                    artefact: entry.artefact.clone(),
                },
            );
            return Ok(entry.artefact.clone());
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
            Ok(compiled) => Some(Arc::new(compiled)),
            Err(issues) => {
                error!(
                    realm_id = %realm_id,
                    version,
                    issues = issues.len(),
                    "authorization rules failed to compile, keeping the previous ones"
                );
                cached.and_then(|previous| previous.artefact)
            }
        };

        self.store(
            realm_id,
            Compiled {
                version,
                checked_at: now,
                artefact: artefact.clone(),
            },
        );
        Ok(artefact)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use ferriskey_domain::realm::Realm;
    use serde_json::Map;

    use super::*;
    use crate::entities::{EntityRef, EntityType};
    use crate::tenant::evaluation::TenantEntity;
    use crate::tenant::fakes::{FakeEngine, FakeRules, FakeSlices};

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
    async fn rules_that_never_compiled_are_retried_only_when_the_version_moves() {
        let f = fixture(true);
        f.engine.failing.store(true, Ordering::SeqCst);

        assert!(!f.evaluate_at(Duration::ZERO).await.allowed);
        assert!(!f.evaluate_at(VERSION_CHECK_INTERVAL / 2).await.allowed);
        assert!(!f.evaluate_at(VERSION_CHECK_INTERVAL * 2).await.allowed);
        assert_eq!(f.compilations(), 1);

        f.engine.failing.store(false, Ordering::SeqCst);
        f.rules.version.store(1, Ordering::SeqCst);
        assert!(f.evaluate_at(VERSION_CHECK_INTERVAL * 4).await.allowed);
        assert_eq!(f.compilations(), 2);
    }

    #[tokio::test]
    async fn a_disabled_realm_answers_not_enabled_once_the_interval_elapsed() {
        let f = fixture(true);
        f.evaluate_at(Duration::ZERO).await;
        f.rules.disabled.store(true, Ordering::SeqCst);

        let result = f
            .evaluator
            .evaluate(
                &f.scope,
                &ferriskey_subject(),
                f.start + VERSION_CHECK_INTERVAL * 2,
            )
            .await;
        assert!(
            matches!(result, Err(CoreError::AuthorizationNotEnabled)),
            "got {result:?}"
        );

        f.rules.disabled.store(false, Ordering::SeqCst);
        f.evaluate_at(VERSION_CHECK_INTERVAL * 2).await;
        assert_eq!(f.compilations(), 2);
    }

    #[tokio::test]
    async fn a_tenant_subject_cannot_claim_a_ferriskey_membership() {
        let f = fixture(true);
        let request = EvaluationRequest {
            subject: Subject::Tenant(TenantEntity {
                entity: EntityRef::new(EntityType::custom("App::Device"), "d1"),
                properties: Map::new(),
                parents: vec![EntityRef::new(
                    EntityType::custom("FerrisKey::Role"),
                    "admin",
                )],
            }),
            ..ferriskey_subject()
        };

        let result = f.evaluator.evaluate(&f.scope, &request, f.start).await;

        assert!(
            matches!(result, Err(CoreError::InvalidAuthorizationRequest(_))),
            "got {result:?}"
        );
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
    async fn the_request_cannot_describe_a_ferriskey_entity() {
        let f = fixture(true);
        let forged = TenantEntity {
            entity: EntityRef::new(EntityType::custom("FerrisKey::User"), "u1"),
            properties: Map::new(),
            parents: vec![EntityRef::new(
                EntityType::custom("FerrisKey::Role"),
                "admin",
            )],
        };
        let as_subject = EvaluationRequest {
            subject: Subject::Tenant(forged.clone()),
            ..ferriskey_subject()
        };
        let as_resource = EvaluationRequest {
            resource: forged,
            ..ferriskey_subject()
        };

        for request in [as_subject, as_resource] {
            let result = f.evaluator.evaluate(&f.scope, &request, f.start).await;
            assert!(
                matches!(result, Err(CoreError::InvalidAuthorizationRequest(_))),
                "got {result:?}"
            );
        }
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
