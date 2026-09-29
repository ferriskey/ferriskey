use std::collections::HashMap;

use crate::domain::authentication::token_exchange::entities::TokenExchangeError;
use crate::domain::authentication::token_exchange::value_objects::{
    TokenExchangeOutput, TokenExchangeParams,
};
use crate::domain::client::entities::Client;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::jwt::entities::{Jwt, JwtClaim};
use crate::domain::realm::entities::{RealmId, RealmScope};
use crate::domain::user::entities::User;

/// The token primitives the exchange borrows from the auth service: checking a
/// token FerrisKey signed, running a client's protocol mappers, and signing a
/// new token. The exchange decides what goes in the token.
#[cfg_attr(test, mockall::automock)]
pub trait SubjectTokenIssuer: Send + Sync {
    /// Verify a token signed by this realm: signature, expiry, the `sid`
    /// session it is bound to, and revocation.
    fn verify_subject_token(
        &self,
        token: String,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<JwtClaim, CoreError>> + Send;

    /// The claims `client`'s protocol mappers produce for `user`, limited to
    /// the client scopes that `scope` names.
    fn mapped_claims(
        &self,
        realm: RealmScope,
        user: User,
        client: Client,
        scope: Option<String>,
    ) -> impl Future<Output = Result<HashMap<String, serde_json::Value>, CoreError>> + Send;

    /// Sign `claims` as an access token and persist it, so it can be
    /// introspected and revoked like any other.
    fn issue_access_token(
        &self,
        realm_id: RealmId,
        claims: JwtClaim,
    ) -> impl Future<Output = Result<Jwt, CoreError>> + Send;
}

pub trait TokenExchangeService: Send + Sync {
    /// Swap a subject token for a new access token (RFC 8693 §2).
    fn exchange(
        &self,
        params: TokenExchangeParams,
    ) -> impl Future<Output = Result<TokenExchangeOutput, TokenExchangeError>> + Send;
}
