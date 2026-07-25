use std::future::Future;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::Realm;

pub trait PasswordPolicyPolicy: Send + Sync {
    fn can_view_password_policy(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_password_policy(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
