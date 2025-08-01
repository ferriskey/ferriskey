use uuid::Uuid;

use crate::domain::authentication::{
    entities::{AuthSession, AuthenticationError, GrantType, JwtToken},
    value_objects::{CreateAuthSessionRequest, GrantTypeParams},
};

/// A strategy for handling different OAuth2 grant types during authentication.
///
/// This trait defines the contract for implementing specific grant type strategies,
/// such as `AuthorizationCode`, `ClientCredentials`, or `Password` grant types.
/// Each implementation of this trait should handle the logic for its respective grant type.
pub trait GrantTypeStrategy: Clone + Send + Sync + 'static {
    /// Executes the grant type strategy to authenticate a user or client.
    ///
    /// # Parameters
    /// - `params`: The parameters required to execute the grant type strategy. These
    ///   parameters are encapsulated in the `GrantTypeParams` value object.
    ///
    /// # Returns
    /// A future that resolves to a `Result` containing either:
    /// - `JwtToken`: The generated JWT token upon successful authentication.
    /// - `AuthenticationError`: An error indicating why the authentication failed.
    ///
    /// # Examples
    /// ```
    /// // Example usage of a GrantTypeStrategy implementation
    /// let strategy: Box<dyn GrantTypeStrategy> = ...;
    /// let params = GrantTypeParams::new(...);
    /// let result = strategy.execute(params).await;
    /// match result {
    ///     Ok(token) => println!("Authentication successful: {:?}", token),
    ///     Err(err) => eprintln!("Authentication failed: {:?}", err),
    /// }
    /// ```
    fn execute(
        &self,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}

pub trait GrantTypeService: Clone + Send + Sync + 'static {
    fn authenticate_with_grant_type(
        &self,
        grant_type: GrantType,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}

pub trait AuthSessionService: Clone + Send + Sync + 'static {
    fn create_session(
        &self,
        dto: CreateAuthSessionRequest,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;

    fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;

    fn get_by_code(
        &self,
        code: String,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;

    fn update_code(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
}

pub trait AuthSessionRepository: Clone + Send + Sync + 'static {
    fn create(
        &self,
        session: &AuthSession,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
    fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
    fn get_by_code(
        &self,
        code: String,
    ) -> impl Future<Output = Result<Option<AuthSession>, AuthenticationError>> + Send;
    fn update_code_and_user_id(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<AuthSession, AuthenticationError>> + Send;
}
