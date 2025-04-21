use uuid::Uuid;

use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};
use crate::domain::jwt::ports::jwt_repository::{JwtRepository, RefreshTokenRepository};
use crate::domain::jwt::ports::jwt_service::JwtService;
use crate::infrastructure::repositories::jwt_repository::StaticJwtRepository;
use crate::infrastructure::repositories::refresh_token_repository::PostgresRefreshTokenRepository;

pub type DefaultJwtService = JwtServiceImpl<StaticJwtRepository, PostgresRefreshTokenRepository>;

#[derive(Clone)]
pub struct JwtServiceImpl<JR, RR>
where
    JR: JwtRepository,
    RR: RefreshTokenRepository,
{
    pub jwt_repository: JR,
    pub refresh_token_repository: RR,
}

impl<JR: JwtRepository, RR: RefreshTokenRepository> JwtServiceImpl<JR, RR> {
    pub fn new(jwt_repository: JR, refresh_token_repository: RR) -> Self {
        Self {
            jwt_repository,
            refresh_token_repository,
        }
    }
}

impl<JR: JwtRepository, RR: RefreshTokenRepository> JwtService for JwtServiceImpl<JR, RR> {
    async fn generate_token(&self, claims: JwtClaim) -> Result<Jwt, JwtError> {
        self.jwt_repository.generate_jwt_token(&claims).await
    }

    async fn verify_token(&self, token: String) -> Result<JwtClaim, JwtError> {
        self.jwt_repository.verify_token(token).await
    }

    async fn generate_refresh_token(&self, user_id: Uuid) -> Result<Jwt, JwtError> {
        let claims = JwtClaim::new_refresh_token(
            user_id,
            "http://localhost:3333/realms/master".to_string(),
            vec!["master-realm".to_string(), "account".to_string()],
            "master-realm".to_string(),
        );

        self.jwt_repository.generate_jwt_token(&claims).await
    }
}
