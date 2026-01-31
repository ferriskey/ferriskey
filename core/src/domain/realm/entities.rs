pub use ferriskey_domain::realm::{Realm, RealmId, RealmSetting};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::identity_provider::entities::IdentityProviderPresentation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct RealmLoginSetting {
    pub user_registration_enabled: bool,
    pub forgot_password_enabled: bool,
    pub remember_me_enabled: bool,
    pub identity_providers: Vec<IdentityProviderPresentation>,
}

impl From<RealmSetting> for RealmLoginSetting {
    fn from(value: RealmSetting) -> Self {
        Self {
            forgot_password_enabled: value.forgot_password_enabled,
            remember_me_enabled: value.remember_me_enabled,
            user_registration_enabled: value.user_registration_enabled,
            identity_providers: Vec::new(),
        }
    }
}
