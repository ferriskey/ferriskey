use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        organization::ports::{
            AddGroupMemberInput, AddOrganizationMemberInput, AssignGroupRoleInput,
            CreateGroupInput, CreateOrganizationInput, DeleteGroupAttributeInput, DeleteGroupInput,
            DeleteOrganizationAttributeInput, DeleteOrganizationInput, GetGroupInput,
            GetOrganizationInput, Group, GroupAttribute, GroupMember, GroupMemberPage, GroupNode,
            GroupService, ListGroupAttributesInput, ListGroupMembersInput, ListGroupRolesInput,
            ListGroupsInput, ListOrganizationAttributesInput, ListOrganizationMembersInput,
            ListOrganizationsInput, ListUserOrganizationsInput, Organization,
            OrganizationAttribute, OrganizationMember, OrganizationService, RemoveGroupMemberInput,
            RemoveOrganizationMemberInput, RevokeGroupRoleInput, UpdateGroupInput,
            UpdateOrganizationInput, UpsertGroupAttributeInput, UpsertOrganizationAttributeInput,
        },
        role::entities::Role,
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

impl GroupService for ApplicationService {
    async fn create_group(
        &self,
        identity: Identity,
        input: CreateGroupInput,
    ) -> Result<Group, CoreError> {
        self.group_service.create_group(identity, input).await
    }

    async fn get_group(
        &self,
        identity: Identity,
        input: GetGroupInput,
    ) -> Result<Group, CoreError> {
        self.group_service.get_group(identity, input).await
    }

    async fn list_groups(
        &self,
        identity: Identity,
        input: ListGroupsInput,
    ) -> Result<Vec<GroupNode>, CoreError> {
        self.group_service.list_groups(identity, input).await
    }

    async fn update_group(
        &self,
        identity: Identity,
        input: UpdateGroupInput,
    ) -> Result<Group, CoreError> {
        self.group_service.update_group(identity, input).await
    }

    async fn delete_group(
        &self,
        identity: Identity,
        input: DeleteGroupInput,
    ) -> Result<(), CoreError> {
        self.group_service.delete_group(identity, input).await
    }

    async fn add_member(
        &self,
        identity: Identity,
        input: AddGroupMemberInput,
    ) -> Result<GroupMember, CoreError> {
        self.group_service.add_member(identity, input).await
    }

    async fn remove_member(
        &self,
        identity: Identity,
        input: RemoveGroupMemberInput,
    ) -> Result<(), CoreError> {
        self.group_service.remove_member(identity, input).await
    }

    async fn list_members(
        &self,
        identity: Identity,
        input: ListGroupMembersInput,
    ) -> Result<GroupMemberPage, CoreError> {
        self.group_service.list_members(identity, input).await
    }

    async fn assign_role(
        &self,
        identity: Identity,
        input: AssignGroupRoleInput,
    ) -> Result<(), CoreError> {
        self.group_service.assign_role(identity, input).await
    }

    async fn revoke_role(
        &self,
        identity: Identity,
        input: RevokeGroupRoleInput,
    ) -> Result<(), CoreError> {
        self.group_service.revoke_role(identity, input).await
    }

    async fn list_roles(
        &self,
        identity: Identity,
        input: ListGroupRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        self.group_service.list_roles(identity, input).await
    }

    async fn list_attributes(
        &self,
        identity: Identity,
        input: ListGroupAttributesInput,
    ) -> Result<Vec<GroupAttribute>, CoreError> {
        self.group_service.list_attributes(identity, input).await
    }

    async fn upsert_attribute(
        &self,
        identity: Identity,
        input: UpsertGroupAttributeInput,
    ) -> Result<GroupAttribute, CoreError> {
        self.group_service.upsert_attribute(identity, input).await
    }

    async fn delete_attribute(
        &self,
        identity: Identity,
        input: DeleteGroupAttributeInput,
    ) -> Result<(), CoreError> {
        self.group_service.delete_attribute(identity, input).await
    }
}
