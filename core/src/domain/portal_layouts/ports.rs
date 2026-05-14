use std::future::Future;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity, common::entities::app_errors::CoreError,
    portal_layouts::entities::PortalLayout, realm::entities::Realm,
};

pub trait PortalLayoutsService: Send + Sync {
    fn list_layouts(
        &self,
        identity: Identity,
        input: ListLayoutsInput,
    ) -> impl Future<Output = Result<Vec<PortalLayout>, CoreError>> + Send;

    fn get_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> impl Future<Output = Result<PortalLayout, CoreError>> + Send;

    fn create_layout(
        &self,
        identity: Identity,
        input: CreateLayoutInput,
    ) -> impl Future<Output = Result<PortalLayout, CoreError>> + Send;

    fn update_layout(
        &self,
        identity: Identity,
        input: UpdateLayoutInput,
    ) -> impl Future<Output = Result<PortalLayout, CoreError>> + Send;

    fn set_default_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> impl Future<Output = Result<PortalLayout, CoreError>> + Send;

    fn delete_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_public_default_layout(
        &self,
        input: ListLayoutsInput,
    ) -> impl Future<Output = Result<Option<PortalLayout>, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait PortalLayoutsRepository: Send + Sync {
    fn list_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<PortalLayout>, CoreError>> + Send;

    fn get_by_id(
        &self,
        realm_id: Uuid,
        layout_id: Uuid,
    ) -> impl Future<Output = Result<Option<PortalLayout>, CoreError>> + Send;

    fn get_default(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<PortalLayout>, CoreError>> + Send;

    fn create(
        &self,
        realm_id: Uuid,
        name: String,
        tree: serde_json::Value,
        is_default: bool,
    ) -> impl Future<Output = Result<PortalLayout, CoreError>> + Send;

    fn update(
        &self,
        realm_id: Uuid,
        layout_id: Uuid,
        name: String,
        tree: serde_json::Value,
    ) -> impl Future<Output = Result<PortalLayout, CoreError>> + Send;

    fn set_default(
        &self,
        realm_id: Uuid,
        layout_id: Uuid,
    ) -> impl Future<Output = Result<PortalLayout, CoreError>> + Send;

    fn delete(
        &self,
        realm_id: Uuid,
        layout_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait PortalLayoutsPolicy: Send + Sync {
    fn can_view_layouts(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_layouts(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub struct ListLayoutsInput {
    pub realm_name: String,
}

pub struct GetLayoutInput {
    pub realm_name: String,
    pub layout_id: Uuid,
}

pub struct CreateLayoutInput {
    pub realm_name: String,
    pub name: String,
    pub tree: serde_json::Value,
}

pub struct UpdateLayoutInput {
    pub realm_name: String,
    pub layout_id: Uuid,
    pub name: String,
    pub tree: serde_json::Value,
}
