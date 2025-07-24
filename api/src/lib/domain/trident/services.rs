use std::time::{SystemTime, UNIX_EPOCH};

use base32::encode;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

use crate::domain::trident::{
    entities::{TotpError, TotpSecret},
    ports::TotpService,
};

type HmacSha256 = Hmac<Sha256>;

pub type DefaultTotpService = OauthTotpService;

#[derive(Debug, Clone)]
pub struct OauthTotpService;

impl OauthTotpService {
    pub fn new() -> Self {
        OauthTotpService {}
    }

    fn generate_totp_code(secret: &[u8], counter: u64, digits: u32) -> Result<u32, TotpError> {
        let mut mac = HmacSha256::new_from_slice(secret)
            .map_err(|_| TotpError::GenerationFailed("Failed to create HMAC".to_string()))?;

        let mut counter_bytes = [0u8; 8];

        counter_bytes.copy_from_slice(&counter.to_be_bytes());

        mac.update(&counter_bytes);
        let hmac_result = mac.finalize().into_bytes();

        let offset = (hmac_result[19] & 0x0f) as usize;
        let code = ((hmac_result[offset] as u32 & 0x7f) << 24)
            | ((hmac_result[offset + 1] as u32) << 16)
            | ((hmac_result[offset + 2] as u32) << 8)
            | (hmac_result[offset + 3] as u32);

        Ok(code % 10u32.pow(digits))
    }
}

impl TotpService for OauthTotpService {
    fn generate_secret(&self) -> Result<TotpSecret, TotpError> {
        let mut bytes = [0u8; 20];
        rand::thread_rng().try_fill_bytes(&mut bytes).map_err(|_| {
            TotpError::GenerationFailed("Failed to generate random bytes".to_string())
        })?;
        let base32 = encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes); // base32 sans padding
        Ok(TotpSecret::from_base32(&base32))
    }

    fn generate_otpauth_uri(&self, issuer: &str, user_email: &str, secret: &TotpSecret) -> String {
        let encoded_secret = secret.base32_encoded();

        let issuer_encoded = urlencoding::encode(issuer);
        let label_encoded = urlencoding::encode(user_email);

        format!(
            "otpauth://totp/{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
            label_encoded, encoded_secret, issuer_encoded
        )
    }

    fn verify(&self, secret: &TotpSecret, code: &str) -> Result<bool, TotpError> {
        let Ok(expected_code) = code.parse::<u32>() else {
            return Ok(false);
        };

        let Ok(secret_bytes) = secret.to_bytes() else {
            return Ok(false);
        };

        let time_step = 30;
        let digits = 6;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX_EPOCH")
            .as_secs();

        let counter = now / time_step;

        for i in -1..=1 {
            let adjusted_counter = counter.wrapping_add(i as u64);
            let generated = Self::generate_totp_code(&secret_bytes, adjusted_counter, digits)?;
            if generated == expected_code {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
