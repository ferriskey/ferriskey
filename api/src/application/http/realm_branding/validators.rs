use ferriskey_core::domain::realm_branding::entities::BrandingConfig;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateBrandingValidator {
    pub config: BrandingConfig,
}
