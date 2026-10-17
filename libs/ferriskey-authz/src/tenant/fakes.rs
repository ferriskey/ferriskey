use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};

use ferriskey_domain::common::app_errors::{CoreError, PolicyIssue};
use ferriskey_domain::realm::scope::RealmScope;
use uuid::Uuid;

use super::entities::{AuthzSchema, AuthzState, Policy, PolicyTemplate};
use super::evaluation::{ClientFacts, EvaluationRequest, PrincipalFacts, PrincipalSlice, RuleSet};
use super::ports::{
    AuthzSchemaRepository, AuthzStateRepository, PolicyRepository, PolicyTemplateRepository,
    PrincipalSliceRepository, TenantDecisionEngine,
};
use crate::entities::{Decision, EntityRef};

#[derive(Default)]
pub(crate) struct FakeRules {
    pub version: AtomicI64,
    pub state_reads: AtomicUsize,
    pub disabled: AtomicBool,
    pub policies: Mutex<Vec<Policy>>,
}

impl AuthzStateRepository for FakeRules {
    async fn get(&self, _: &RealmScope) -> Result<AuthzState, CoreError> {
        self.state_reads.fetch_add(1, Ordering::SeqCst);
        Ok(AuthzState {
            enabled: !self.disabled.load(Ordering::SeqCst),
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
        Ok(self.policies.lock().expect("lock").clone())
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
pub(crate) struct FakeSlices {
    pub known: bool,
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
pub(crate) struct FakeEngine {
    pub compilations: AtomicUsize,
    pub failing: AtomicBool,
    pub decided_with: Mutex<Vec<(i64, bool)>>,
    pub determining: Mutex<Vec<String>>,
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
        Ok(Decision::allow().with_policies(self.determining.lock().expect("lock").clone()))
    }
}
