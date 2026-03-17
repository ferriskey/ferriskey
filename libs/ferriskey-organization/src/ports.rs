use uuid::Uuid;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;

use crate::entities::{
    AddOrganizationMemberInput, CreateOrganizationInput, DeleteOrganizationInput,
    GetOrganizationInput, ListOrganizationMembersInput, ListOrganizationsInput, Organization,
    OrganizationId, OrganizationMember, RemoveOrganizationMemberInput, UpdateOrganizationInput,
};

/// Repository trait for Organization persistence
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait OrganizationRepository: Send + Sync {
    fn create_organization(
        &self,
        realm_id: RealmId,
        name: String,
        display_name: Option<String>,
        description: Option<String>,
        enabled: bool,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn get_organization_by_id(
        &self,
        id: OrganizationId,
    ) -> impl Future<Output = Result<Option<Organization>, CoreError>> + Send;

    fn get_organization_by_realm_and_name(
        &self,
        realm_id: RealmId,
        name: &str,
    ) -> impl Future<Output = Result<Option<Organization>, CoreError>> + Send;

    fn list_organizations_by_realm(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Organization>, CoreError>> + Send;

    fn update_organization(
        &self,
        id: OrganizationId,
        display_name: Option<String>,
        description: Option<String>,
        enabled: Option<bool>,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn delete_organization(
        &self,
        id: OrganizationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn exists_organization_by_realm_and_name(
        &self,
        realm_id: RealmId,
        name: &str,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

/// Repository trait for OrganizationMember persistence
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait OrganizationMemberRepository: Send + Sync {
    fn add_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<OrganizationMember, CoreError>> + Send;

    fn remove_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_members(
        &self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<OrganizationMember>, CoreError>> + Send;

    fn get_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<OrganizationMember>, CoreError>> + Send;
}

/// Service trait for Organization business logic
pub trait OrganizationService: Send + Sync {
    fn create_organization(
        &self,
        identity: Identity,
        input: CreateOrganizationInput,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn get_organization(
        &self,
        identity: Identity,
        input: GetOrganizationInput,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn list_organizations(
        &self,
        identity: Identity,
        input: ListOrganizationsInput,
    ) -> impl Future<Output = Result<Vec<Organization>, CoreError>> + Send;

    fn update_organization(
        &self,
        identity: Identity,
        input: UpdateOrganizationInput,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn delete_organization(
        &self,
        identity: Identity,
        input: DeleteOrganizationInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn add_member(
        &self,
        identity: Identity,
        input: AddOrganizationMemberInput,
    ) -> impl Future<Output = Result<OrganizationMember, CoreError>> + Send;

    fn remove_member(
        &self,
        identity: Identity,
        input: RemoveOrganizationMemberInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_members(
        &self,
        identity: Identity,
        input: ListOrganizationMembersInput,
    ) -> impl Future<Output = Result<Vec<OrganizationMember>, CoreError>> + Send;
}

/// Policy trait for Organization authorization
pub trait OrganizationPolicy: Send + Sync {
    fn can_create_organization(
        &self,
        identity: &Identity,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_organization(
        &self,
        identity: &Identity,
        organization: &Organization,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_organization(
        &self,
        identity: &Identity,
        organization: &Organization,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_organization(
        &self,
        identity: &Identity,
        organization: &Organization,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_members(
        &self,
        identity: &Identity,
        organization: &Organization,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
