use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: usize,
    pub jti: String,
}

impl RefreshClaims {
    pub fn new(sub: String, exp: usize, jti: String) -> Self {
        Self { sub, exp, jti }
    }
}
