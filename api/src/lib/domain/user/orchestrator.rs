use crate::{
    application::auth::Identity,
    domain::{
        client::services::client_service::DefaultClientService,
        realm::services::realm_service::DefaultRealmService,
        user::{
            entities::error::UserError,
            services::{
                user_role_service::DefaultUserRoleService, user_service::DefaultUserService,
            },
            use_cases::assign_role_use_case::{AssignRoleUseCase, AssignRoleUseCaseParams},
        },
    },
};

#[derive(Clone)]
pub struct UserOrchestrator {
    assign_role_use_case: AssignRoleUseCase,
}

impl UserOrchestrator {
    pub fn new(
        realm_service: DefaultRealmService,
        user_role_service: DefaultUserRoleService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
    ) -> Self {
        let assign_role_use_case = AssignRoleUseCase::new(
            realm_service,
            user_role_service,
            user_service.clone(),
            client_service.clone(),
        );

        Self {
            assign_role_use_case,
        }
    }

    pub async fn assign_role(
        &self,
        identity: Identity,
        params: AssignRoleUseCaseParams,
    ) -> Result<(), UserError> {
        self.assign_role_use_case.execute(identity, params).await
    }
}
