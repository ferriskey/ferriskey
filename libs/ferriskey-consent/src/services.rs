use std::collections::HashSet;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;

use crate::entities::{ConsentDecision, ConsentDecisionId, ScopeDescriptor};
use crate::ports::ConsentDecisionRepository;
use crate::value_objects::{
    ConsentDecisionOutcome, ConsentEvaluation, DEFAULT_CONSENT_TTL_DAYS, DecideConsentInput,
    EvaluateConsentInput, PendingConsentView,
};

pub trait ConsentService: Send + Sync {
    fn evaluate(
        &self,
        input: EvaluateConsentInput,
    ) -> impl Future<Output = Result<ConsentEvaluation, CoreError>> + Send;

    fn decide(
        &self,
        input: DecideConsentInput,
    ) -> impl Future<Output = Result<ConsentDecisionOutcome, CoreError>> + Send;

    fn find_decision(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Option<ConsentDecision>, CoreError>> + Send;
}

#[derive(Debug)]
pub struct ConsentServiceImpl<CD>
where
    CD: ConsentDecisionRepository,
{
    pub decision_repository: Arc<CD>,
}

impl<CD> ConsentServiceImpl<CD>
where
    CD: ConsentDecisionRepository,
{
    pub fn new(decision_repository: Arc<CD>) -> Self {
        Self {
            decision_repository,
        }
    }
}

fn names(scopes: &[ScopeDescriptor]) -> Vec<String> {
    scopes.iter().map(|scope| scope.name.clone()).collect()
}

impl<CD> ConsentService for ConsentServiceImpl<CD>
where
    CD: ConsentDecisionRepository,
{
    async fn evaluate(&self, input: EvaluateConsentInput) -> Result<ConsentEvaluation, CoreError> {
        let default_names = names(&input.default_scopes);

        if !input.consent_required {
            let mut granted = default_names;
            granted.extend(names(&input.requested_optional_scopes));
            return Ok(ConsentEvaluation::Skip {
                granted_scopes: granted,
            });
        }

        if input.requested_optional_scopes.is_empty() {
            return Ok(ConsentEvaluation::Skip {
                granted_scopes: default_names,
            });
        }

        let requested_optional_names = names(&input.requested_optional_scopes);

        if !input.force_screen {
            let existing = self
                .decision_repository
                .find(input.realm_id, input.user_id, input.client_id)
                .await?;

            if let Some(decision) = existing
                && decision.is_live(Utc::now())
                && decision.covers(&requested_optional_names)
            {
                let granted_set: HashSet<&String> = decision.granted_scopes.iter().collect();
                let mut granted = default_names;
                granted.extend(
                    requested_optional_names
                        .iter()
                        .filter(|name| granted_set.contains(name))
                        .cloned(),
                );

                return Ok(ConsentEvaluation::Skip {
                    granted_scopes: granted,
                });
            }
        }

        Ok(ConsentEvaluation::Show(PendingConsentView {
            default_scopes: input.default_scopes,
            optional_scopes: input.requested_optional_scopes,
        }))
    }

    async fn decide(&self, input: DecideConsentInput) -> Result<ConsentDecisionOutcome, CoreError> {
        let approved: Vec<String> = input
            .requested_optional_scope_names
            .iter()
            .filter(|name| input.approved_scope_names.contains(name))
            .cloned()
            .collect();

        let denied: Vec<String> = input
            .requested_optional_scope_names
            .iter()
            .filter(|name| !approved.contains(name))
            .cloned()
            .collect();

        let is_denied = !input.requested_optional_scope_names.is_empty() && approved.is_empty();

        let ttl_days = input.ttl_days.unwrap_or(DEFAULT_CONSENT_TTL_DAYS);
        let now = Utc::now();

        let decision = ConsentDecision {
            id: ConsentDecisionId::new(),
            realm_id: input.realm_id,
            user_id: input.user_id,
            client_id: input.client_id,
            granted_scopes: approved.clone(),
            denied_scopes: denied,
            expires_at: now + chrono::Duration::days(ttl_days),
            created_at: now,
            updated_at: now,
        };

        self.decision_repository.upsert(decision).await?;

        if is_denied {
            return Ok(ConsentDecisionOutcome::Denied);
        }

        let mut granted_scopes = input.default_scope_names;
        granted_scopes.extend(approved);

        Ok(ConsentDecisionOutcome::Approved { granted_scopes })
    }

    async fn find_decision(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<ConsentDecision>, CoreError> {
        self.decision_repository
            .find(realm_id, user_id, client_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::MockConsentDecisionRepository;
    use ferriskey_domain::realm::RealmId;

    fn scope(name: &str) -> ScopeDescriptor {
        ScopeDescriptor {
            name: name.to_string(),
            description: Some(format!("{name} description")),
        }
    }

    fn base_input(consent_required: bool, force_screen: bool) -> EvaluateConsentInput {
        EvaluateConsentInput {
            realm_id: RealmId::from(Uuid::new_v4()),
            user_id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            consent_required,
            default_scopes: vec![scope("profile")],
            requested_optional_scopes: vec![scope("contacts")],
            force_screen,
        }
    }

    #[tokio::test]
    async fn a_client_that_does_not_require_consent_never_reaches_the_screen() {
        let repository = MockConsentDecisionRepository::new();
        let service = ConsentServiceImpl::new(Arc::new(repository));

        let evaluation = service
            .evaluate(base_input(false, false))
            .await
            .expect("evaluate succeeds");

        assert_eq!(
            evaluation,
            ConsentEvaluation::Skip {
                granted_scopes: vec!["profile".to_string(), "contacts".to_string()],
            }
        );
    }

    #[tokio::test]
    async fn a_stored_unexpired_consent_covering_the_requested_scopes_skips_the_screen() {
        let mut repository = MockConsentDecisionRepository::new();
        repository
            .expect_find()
            .returning(|realm_id, user_id, client_id| {
                Box::pin(async move {
                    Ok(Some(ConsentDecision {
                        id: ConsentDecisionId::new(),
                        realm_id,
                        user_id,
                        client_id,
                        granted_scopes: vec!["contacts".to_string()],
                        denied_scopes: vec![],
                        expires_at: Utc::now() + chrono::Duration::days(1),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }))
                })
            });
        let service = ConsentServiceImpl::new(Arc::new(repository));

        let evaluation = service
            .evaluate(base_input(true, false))
            .await
            .expect("evaluate succeeds");

        assert_eq!(
            evaluation,
            ConsentEvaluation::Skip {
                granted_scopes: vec!["profile".to_string(), "contacts".to_string()],
            }
        );
    }

    #[tokio::test]
    async fn an_expired_stored_consent_does_not_skip_the_screen() {
        let mut repository = MockConsentDecisionRepository::new();
        repository
            .expect_find()
            .returning(|realm_id, user_id, client_id| {
                Box::pin(async move {
                    Ok(Some(ConsentDecision {
                        id: ConsentDecisionId::new(),
                        realm_id,
                        user_id,
                        client_id,
                        granted_scopes: vec!["contacts".to_string()],
                        denied_scopes: vec![],
                        expires_at: Utc::now() - chrono::Duration::days(1),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }))
                })
            });
        let service = ConsentServiceImpl::new(Arc::new(repository));

        let evaluation = service
            .evaluate(base_input(true, false))
            .await
            .expect("evaluate succeeds");

        assert!(matches!(evaluation, ConsentEvaluation::Show(_)));
    }

    #[tokio::test]
    async fn prompt_equals_consent_forces_the_screen_even_with_a_valid_stored_consent() {
        let repository = MockConsentDecisionRepository::new();
        let service = ConsentServiceImpl::new(Arc::new(repository));

        let evaluation = service
            .evaluate(base_input(true, true))
            .await
            .expect("evaluate succeeds");

        assert!(matches!(evaluation, ConsentEvaluation::Show(_)));
    }

    #[tokio::test]
    async fn denying_every_optional_scope_yields_a_denied_outcome() {
        let mut repository = MockConsentDecisionRepository::new();
        repository
            .expect_upsert()
            .returning(|decision| Box::pin(async move { Ok(decision) }));
        let service = ConsentServiceImpl::new(Arc::new(repository));

        let outcome = service
            .decide(DecideConsentInput {
                realm_id: RealmId::from(Uuid::new_v4()),
                user_id: Uuid::new_v4(),
                client_id: Uuid::new_v4(),
                default_scope_names: vec!["profile".to_string()],
                requested_optional_scope_names: vec!["contacts".to_string()],
                approved_scope_names: vec![],
                ttl_days: None,
            })
            .await
            .expect("decide succeeds");

        assert_eq!(outcome, ConsentDecisionOutcome::Denied);
    }

    #[tokio::test]
    async fn a_default_scope_never_appears_in_the_deniable_optional_list() {
        let repository = MockConsentDecisionRepository::new();
        let service = ConsentServiceImpl::new(Arc::new(repository));

        let evaluation = service
            .evaluate(base_input(true, true))
            .await
            .expect("evaluate succeeds");

        let ConsentEvaluation::Show(view) = evaluation else {
            panic!("expected the consent screen to be shown");
        };

        assert!(!view.optional_scopes.iter().any(|s| s.name == "profile"));
        assert_eq!(view.optional_scopes, vec![scope("contacts")]);
    }
}
