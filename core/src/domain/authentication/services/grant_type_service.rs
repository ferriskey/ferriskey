use crate::domain::{
    authentication::{
        entities::{AuthenticationError, GrantType, JwtToken},
        ports::{AuthSessionRepository, GrantTypeService, GrantTypeStrategy},
        strategies::{
            authorization_code_strategy::AuthorizationCodeStrategy,
            client_credentials_strategy::ClientCredentialsStrategy,
            password_strategy::PasswordStrategy, refresh_token_strategy::RefreshTokenStrategy,
        },
        value_objects::GrantTypeParams,
    },
    client::ports::ClientRepository,
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
};

#[derive(Clone)]
pub struct GrantTypeServiceImpl<RR, K, R, C, U, UR, URA, CR, H, A>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    A: AuthSessionRepository,
{
    authorization_code_strategy: AuthorizationCodeStrategy<RR, K, R, C, U, UR, URA, CR, H, A>,
    client_credentials_strategy: ClientCredentialsStrategy<C, U, R, UR, URA, RR, K>,
    password_strategy: PasswordStrategy<RR, K, R, U, UR, URA, C, H, CR>,
    refresh_token_strategy: RefreshTokenStrategy<RR, K, R, C, U, UR, URA>,
}

impl<RR, K, R, C, U, UR, URA, CR, H, A> GrantTypeServiceImpl<RR, K, R, C, U, UR, URA, CR, H, A>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    A: AuthSessionRepository,
{
    pub fn new(
        authorization_code_strategy: AuthorizationCodeStrategy<RR, K, R, C, U, UR, URA, CR, H, A>,
        client_credentials_strategy: ClientCredentialsStrategy<C, U, R, UR, URA, RR, K>,
        password_strategy: PasswordStrategy<RR, K, R, U, UR, URA, C, H, CR>,
        refresh_token_strategy: RefreshTokenStrategy<RR, K, R, C, U, UR, URA>,
    ) -> Self {
        Self {
            authorization_code_strategy,
            client_credentials_strategy,
            password_strategy,
            refresh_token_strategy,
        }
    }
}

impl<RR, K, R, C, U, UR, URA, CR, H, A> GrantTypeService
    for GrantTypeServiceImpl<RR, K, R, C, U, UR, URA, CR, H, A>
where
    RR: RefreshTokenRepository,
    K: KeyStoreRepository,
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    A: AuthSessionRepository,
{
    async fn authenticate_with_grant_type(
        &self,
        grant_type: GrantType,
        params: GrantTypeParams,
    ) -> Result<JwtToken, AuthenticationError> {
        match grant_type {
            GrantType::Code => self.authorization_code_strategy.execute(params).await,
            GrantType::Password => self.password_strategy.execute(params).await,
            GrantType::Credentials => self.client_credentials_strategy.execute(params).await,
            GrantType::RefreshToken => self.refresh_token_strategy.execute(params).await,
        }
    }
}
