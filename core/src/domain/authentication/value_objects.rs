use crate::domain::jwt::entities::JwtClaim;

// Plain OAuth2/OIDC value objects now live in the shared `ferriskey-domain` crate.
// Re-exported here so existing `crate::domain::authentication::value_objects::*` call sites
// keep compiling. `GetUserInfoInput` stays in `core` because it carries a `JwtClaim`
// (defined in the `ferriskey-security` crate, out of scope for the leaf `ferriskey-domain`).
pub use ferriskey_domain::auth::{Identity, IdentityKind};
pub use ferriskey_domain::authentication::value_objects::{
    AuthenticateRequest, AuthenticationResult, CodeChallengeMethod, CreateAuthSessionRequest,
    EndSessionInput, EndSessionOutput, EvaluateClientScopesInput, EvaluateClientScopesRequest,
    EvaluateClientScopesResult, EvaluatedMapper, EvaluatedRoles, EvaluatedScope,
    GenerateTokenInput, GenerateTokensForUserInput, GrantTypeParams, IntrospectTokenInput,
    RegisterUserInput, RegisterUserOutput, RegisterUserUrlContext, RevokeTokenInput,
    UserInfoResponse,
};

pub struct GetUserInfoInput {
    pub realm_name: String,
    pub token: String,
    pub claims: JwtClaim,
}
