use crate::domain::jwt::entities::{jwt::Jwt, jwt_claim::JwtClaim, jwt_error::JwtError};
use crate::domain::jwt::ports::jwt_repository::JwtRepository;
use crate::domain::jwt::ports::jwt_service::JwtService;
use crate::infrastructure::repositories::jwt_repository::StaticJwtRepository;

pub type DefaultJwtService = JwtServiceImpl<StaticJwtRepository>;

#[derive(Clone)]
pub struct JwtServiceImpl<R>
where
    R: JwtRepository,
{
    pub repository: R,
}

impl<R: JwtRepository> JwtServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: JwtRepository> JwtService for JwtServiceImpl<R> {
    async fn generate_token(&self, claims: JwtClaim) -> Result<Jwt, JwtError> {
        self.repository.generate_jwt_token(&claims).await
    }

    async fn verify_token(&self, token: String) -> Result<JwtClaim, JwtError> {
        self.repository.verify_token(token).await
    }
}
