use crate::domain::trident::entities::{TotpError, TotpSecret};

pub trait TotpService: Send + Sync + Clone + 'static {
    fn generate_secret(&self) -> Result<TotpSecret, TotpError>;
    fn generate_otpauth_uri(&self, issuer: &str, user_email: &str, secret: &TotpSecret) -> String;
    fn verify(&self, secret: &TotpSecret, code: &str) -> Result<bool, TotpError>;
}
