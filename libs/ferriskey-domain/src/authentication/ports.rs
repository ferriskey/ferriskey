use crate::authentication::entities::{AuthenticationError, GrantType, JwtToken};
use crate::authentication::value_objects::GrantTypeParams;

/// A strategy for handling different OAuth2 grant types during authentication.
///
/// This trait defines the contract for implementing specific grant type strategies,
/// such as `AuthorizationCode`, `ClientCredentials`, or `Password` grant types.
/// Each implementation of this trait should handle the logic for its respective grant type.
pub trait GrantTypeService: Send + Sync {
    fn authenticate_with_grant_type(
        &self,
        grant_type: GrantType,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}
