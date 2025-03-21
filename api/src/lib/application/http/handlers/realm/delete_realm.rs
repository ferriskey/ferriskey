use std::sync::Arc;

use axum::Extension;
use axum_macros::TypedPath;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::http::{
        errors::{ApiError, ValidateJson},
        validation::realm::DeleteRealmValidator,
    },
    domain::realm::ports::RealmService,
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{id}")]
pub struct DeleteRealmRoute {
    pub id: Uuid,
}

pub async fn delete_realm<R: RealmService>(
    _: DeleteRealmRoute,
    Extension(realm_service): Extension<Arc<R>>,
    ValidateJson(payload): ValidateJson<DeleteRealmValidator>,
) -> Result<(), ApiError> {
    realm_service
        .delete_realm(payload.id)
        .await
        .map_err(ApiError::from)?;

    Ok(())
}
