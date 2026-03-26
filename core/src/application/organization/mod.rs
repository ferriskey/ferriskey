use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        organization::ports::{
            AddOrganizationMemberInput, CreateOrganizationInput, DeleteOrganizationAttributeInput,
            DeleteOrganizationInput, GetOrganizationInput, ListOrganizationAttributesInput,
            ListOrganizationMembersInput, ListOrganizationsInput, ListUserOrganizationsInput,
            Organization, OrganizationAttribute, OrganizationMember, OrganizationService,
            RemoveOrganizationMemberInput, UpdateOrganizationInput,
            UpsertOrganizationAttributeInput,
        },
    },
};

impl OrganizationService for ApplicationService {
    async fn create_organization(
        &self,
        identity: Identity,
        input: CreateOrganizationInput,
    ) -> Result<Organization, CoreError> {
        self.organization_service
            .create_organization(identity, input)
            .await
    }

    async fn get_organization(
        &self,
        identity: Identity,
        input: GetOrganizationInput,
    ) -> Result<Organization, CoreError> {
        self.organization_service
            .get_organization(identity, input)
            .await
    }

    async fn list_organizations(
        &self,
        identity: Identity,
        input: ListOrganizationsInput,
    ) -> Result<Vec<Organization>, CoreError> {
        self.organization_service
            .list_organizations(identity, input)
            .await
    }

    async fn update_organization(
        &self,
        identity: Identity,
        input: UpdateOrganizationInput,
    ) -> Result<Organization, CoreError> {
        self.organization_service
            .update_organization(identity, input)
            .await
    }

    async fn delete_organization(
        &self,
        identity: Identity,
        input: DeleteOrganizationInput,
    ) -> Result<(), CoreError> {
        self.organization_service
            .delete_organization(identity, input)
            .await
    }

    async fn list_attributes(
        &self,
        identity: Identity,
        input: ListOrganizationAttributesInput,
    ) -> Result<Vec<OrganizationAttribute>, CoreError> {
        self.organization_service
            .list_attributes(identity, input)
            .await
    }

    async fn upsert_attribute(
        &self,
        identity: Identity,
        input: UpsertOrganizationAttributeInput,
    ) -> Result<OrganizationAttribute, CoreError> {
        self.organization_service
            .upsert_attribute(identity, input)
            .await
    }

    async fn delete_attribute(
        &self,
        identity: Identity,
        input: DeleteOrganizationAttributeInput,
    ) -> Result<(), CoreError> {
        self.organization_service
            .delete_attribute(identity, input)
            .await
    }

    async fn add_member(
        &self,
        identity: Identity,
        input: AddOrganizationMemberInput,
    ) -> Result<OrganizationMember, CoreError> {
        self.organization_service.add_member(identity, input).await
    }

    async fn remove_member(
        &self,
        identity: Identity,
        input: RemoveOrganizationMemberInput,
    ) -> Result<(), CoreError> {
        self.organization_service
            .remove_member(identity, input)
            .await
    }

    async fn list_members(
        &self,
        identity: Identity,
        input: ListOrganizationMembersInput,
    ) -> Result<Vec<OrganizationMember>, CoreError> {
        self.organization_service
            .list_members(identity, input)
            .await
    }

    async fn list_user_organizations(
        &self,
        identity: Identity,
        input: ListUserOrganizationsInput,
    ) -> Result<Vec<OrganizationMember>, CoreError> {
        self.organization_service
            .list_user_organizations(identity, input)
            .await
    }
}
