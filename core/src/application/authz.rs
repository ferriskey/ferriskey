use ferriskey_authz::tenant::entities::{AuthzSchema, AuthzState, Policy, PolicyTemplate};
use ferriskey_authz::tenant::evaluation::EvaluationRequest;
use ferriskey_authz::tenant::ports::{
    AuthzManagementService, EvaluationOutcome, EvaluationService,
};
use ferriskey_authz::tenant::value_objects::{
    CreatePolicyInput, CreateTemplateInput, ReplaceSchemaInput, UpdatePolicyInput,
    UpdateTemplateInput,
};
use uuid::Uuid;

use crate::application::services::ApplicationService;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;

impl AuthzManagementService for ApplicationService {
    async fn get_state(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<AuthzState, CoreError> {
        self.authz_management_service
            .get_state(identity, realm_name)
            .await
    }

    async fn set_enabled(
        &self,
        identity: Identity,
        realm_name: String,
        enabled: bool,
    ) -> Result<AuthzState, CoreError> {
        self.authz_management_service
            .set_enabled(identity, realm_name, enabled)
            .await
    }

    async fn get_schema(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Option<AuthzSchema>, CoreError> {
        self.authz_management_service
            .get_schema(identity, realm_name)
            .await
    }

    async fn replace_schema(
        &self,
        identity: Identity,
        input: ReplaceSchemaInput,
    ) -> Result<AuthzSchema, CoreError> {
        self.authz_management_service
            .replace_schema(identity, input)
            .await
    }

    async fn list_policies(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<Policy>, CoreError> {
        self.authz_management_service
            .list_policies(identity, realm_name)
            .await
    }

    async fn get_policy(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> Result<Policy, CoreError> {
        self.authz_management_service
            .get_policy(identity, realm_name, id)
            .await
    }

    async fn create_policy(
        &self,
        identity: Identity,
        input: CreatePolicyInput,
    ) -> Result<Policy, CoreError> {
        self.authz_management_service
            .create_policy(identity, input)
            .await
    }

    async fn update_policy(
        &self,
        identity: Identity,
        input: UpdatePolicyInput,
    ) -> Result<Policy, CoreError> {
        self.authz_management_service
            .update_policy(identity, input)
            .await
    }

    async fn delete_policy(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> Result<(), CoreError> {
        self.authz_management_service
            .delete_policy(identity, realm_name, id)
            .await
    }

    async fn list_templates(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<PolicyTemplate>, CoreError> {
        self.authz_management_service
            .list_templates(identity, realm_name)
            .await
    }

    async fn create_template(
        &self,
        identity: Identity,
        input: CreateTemplateInput,
    ) -> Result<PolicyTemplate, CoreError> {
        self.authz_management_service
            .create_template(identity, input)
            .await
    }

    async fn update_template(
        &self,
        identity: Identity,
        input: UpdateTemplateInput,
    ) -> Result<PolicyTemplate, CoreError> {
        self.authz_management_service
            .update_template(identity, input)
            .await
    }

    async fn delete_template(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> Result<(), CoreError> {
        self.authz_management_service
            .delete_template(identity, realm_name, id)
            .await
    }
}

impl EvaluationService for ApplicationService {
    async fn evaluate(
        &self,
        identity: Identity,
        realm_name: String,
        request: EvaluationRequest,
    ) -> Result<EvaluationOutcome, CoreError> {
        self.authz_evaluation_service
            .evaluate(identity, realm_name, request)
            .await
    }
}
