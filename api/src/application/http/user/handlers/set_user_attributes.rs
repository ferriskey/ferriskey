use std::collections::HashMap;

use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::{SetUserAttributesInput, UserAttribute};
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct SetUserAttributesResponse {
    pub data: Vec<UserAttribute>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetUserAttributesValidator {
    /// Key-value pairs to upsert. Existing attributes with the same key are updated.
    pub attributes: HashMap<String, String>,
}

impl validator::Validate for SetUserAttributesValidator {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();
        for key in self.attributes.keys() {
            if key.is_empty() {
                let mut error = validator::ValidationError::new("length");
                error.message = Some("attribute key must not be empty".into());
                errors.add("attributes", error);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[utoipa::path(
    put,
    path = "/{user_id}/attributes",
    tag = "user",
    summary = "Set custom attributes for a user",
    description = "Upserts a set of key-value attributes on the user. Existing keys are updated, new keys are created.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    request_body(
        content = SetUserAttributesValidator,
        description = "Map of attributes to upsert",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Attributes updated successfully", body = SetUserAttributesResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn set_user_attributes(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<SetUserAttributesValidator>,
) -> Result<Response<SetUserAttributesResponse>, ApiError> {
    let attributes = state
        .service
        .set_user_attributes(
            identity,
            SetUserAttributesInput {
                realm_name,
                user_id,
                attributes: payload.attributes,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(SetUserAttributesResponse { data: attributes }))
}
