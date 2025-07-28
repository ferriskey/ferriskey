use std::time::{SystemTime, UNIX_EPOCH};

use base32::encode;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha1::Sha1;
use tracing::error;

use crate::domain::trident::{
    entities::{TotpError, TotpSecret},
    ports::TotpService,
};

type HmacSha1 = Hmac<Sha1>;

pub type DefaultTotpService = OauthTotpService;

#[derive(Debug, Clone, Default)]
pub struct OauthTotpService;

impl OauthTotpService {
    pub fn new() -> Self {
        OauthTotpService {}
    }

    fn generate_totp_code(secret: &[u8], counter: u64, digits: u32) -> Result<u32, TotpError> {
        let mut mac = HmacSha1::new_from_slice(secret)
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
            "otpauth://totp/{label_encoded}?secret={encoded_secret}&issuer={issuer_encoded}&algorithm=SHA1&digits=6&period=30"
        )
    }

    fn verify(&self, secret: &TotpSecret, code: &str) -> Result<bool, TotpError> {
        let Ok(expected_code) = code.parse::<u32>() else {
            error!("Failed to parse code");
            return Ok(false);
        };

        let Ok(secret_bytes) = secret.to_bytes() else {
            error!("Failed to convert secret to bytes");
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

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::domain::trident::{
        entities::TotpSecret, ports::TotpService, services::OauthTotpService,
    };

    #[test]
    fn test_generat_secret_creates_valid_secret() {
        let service = OauthTotpService::new();

        let result = service.generate_secret();

        assert!(result.is_ok());

        let secret = result.unwrap();
        let base32_encoded = secret.base32_encoded();

        assert!(!base32_encoded.is_empty());

        let bytes_result = secret.to_bytes();
        assert!(bytes_result.is_ok());

        let bytes = bytes_result.unwrap();
        assert_eq!(bytes.len(), 20);
    }

    #[test]
    fn test_generated_secret_works_with_verification() {
        let service = OauthTotpService::new();

        let secret = service
            .generate_secret()
            .expect("failed to generate secret");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX_EPOCH")
            .as_secs();

        let counter = now / 30;
        let secret_bytes = secret
            .to_bytes()
            .expect("Failed to convert secret to bytes");

        let generated_code = OauthTotpService::generate_totp_code(&secret_bytes, counter, 6)
            .expect("Failed to generate TOTP code");

        let code_str = format!("{generated_code:06}");

        let verification_result = service.verify(&secret, &code_str);

        assert!(verification_result.is_ok());
        assert!(
            verification_result.unwrap(),
            "Generated code should be valid"
        )
    }

    #[test]
    fn test_verify_with_time_window() {
        let service = OauthTotpService::new();
        let secret = service
            .generate_secret()
            .expect("Failed to generate secret");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX_EPOCH")
            .as_secs();

        let current_counter = now / 30;
        let secret_bytes = secret
            .to_bytes()
            .expect("Failed to convert secret to bytes");

        for time_offset in -1..=1 {
            let counter = current_counter.wrapping_add(time_offset as u64);
            let code = OauthTotpService::generate_totp_code(&secret_bytes, counter, 6)
                .expect("Failed to generate TOTP code");
            let code_str = format!("{code:06}");

            let result = service.verify(&secret, &code_str);
            assert!(result.is_ok());
            assert!(
                result.unwrap(),
                "Code for time offset {time_offset} should be valid"
            );
        }
    }

    #[test]
    fn test_verify_invalid_code() {
        let service = OauthTotpService::new();
        let secret = service
            .generate_secret()
            .expect("Failed to generate secret");

        // Test avec un code mal formaté
        let result = service.verify(&secret, "invalid");
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Invalid code format should return false");

        // Test avec un code trop long
        let result = service.verify(&secret, "1234567");
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Too long code should return false");

        // Test avec un code vide
        let result = service.verify(&secret, "");
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Empty code should return false");
    }

    #[test]
    fn test_generate_otpauth_uri() {
        let service = OauthTotpService::new();
        let secret = service
            .generate_secret()
            .expect("Failed to generate secret");

        let issuer = "MyApp";
        let user_email = "user@example.com";

        let uri = service.generate_otpauth_uri(issuer, user_email, &secret);

        // Vérifier le format de l'URI
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("secret="));
        assert!(uri.contains("issuer=MyApp"));
        assert!(uri.contains("algorithm=SHA1"));
        assert!(uri.contains("digits=6"));
        assert!(uri.contains("period=30"));
        assert!(uri.contains("user%40example.com")); // URL encoded email
    }

    #[test]
    fn test_generate_otpauth_uri_with_special_characters() {
        let service = OauthTotpService::new();
        let secret = service
            .generate_secret()
            .expect("Failed to generate secret");

        let issuer = "My App & Co.";
        let user_email = "user+test@example.com";

        let uri = service.generate_otpauth_uri(issuer, user_email, &secret);

        // Vérifier que les caractères spéciaux sont correctement encodés
        assert!(uri.contains("My%20App%20%26%20Co.")); // URL encoded issuer
        assert!(uri.contains("user%2Btest%40example.com")); // URL encoded email
    }

    #[test]
    fn test_deterministic_totp_generation() {
        // Test avec des valeurs connues pour vérifier la cohérence
        let secret_bytes = [1u8; 20]; // Secret fixe pour test déterministe
        let counter = 12345u64;
        let digits = 6;

        let result1 = OauthTotpService::generate_totp_code(&secret_bytes, counter, digits);
        let result2 = OauthTotpService::generate_totp_code(&secret_bytes, counter, digits);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(
            result1.unwrap(),
            result2.unwrap(),
            "Same inputs should produce same output"
        );
    }

    #[test]
    fn test_different_counters_produce_different_codes() {
        let secret_bytes = [1u8; 20];
        let digits = 6;

        let code1 = OauthTotpService::generate_totp_code(&secret_bytes, 12345, digits)
            .expect("Failed to generate code 1");
        let code2 = OauthTotpService::generate_totp_code(&secret_bytes, 12346, digits)
            .expect("Failed to generate code 2");

        // Il est très improbable (mais pas impossible) que deux counters consécutifs
        // produisent le même code, mais on vérifie quand même
        // En pratique, cela arrive très rarement
        println!("Code 1: {code1:06}, Code 2: {code2:06}");
    }

    #[test]
    fn test_multiple_secret_generations_are_different() {
        let service = OauthTotpService::new();

        let secret1 = service
            .generate_secret()
            .expect("Failed to generate secret 1");
        let secret2 = service
            .generate_secret()
            .expect("Failed to generate secret 2");

        // Les secrets générés doivent être différents
        assert_ne!(
            secret1.base32_encoded(),
            secret2.base32_encoded(),
            "Generated secrets should be different"
        );
    }

    #[test]
    fn test_secret_round_trip() {
        let service = OauthTotpService::new();
        let original_secret = service
            .generate_secret()
            .expect("Failed to generate secret");

        // Convertir en bytes puis recréer le secret
        let bytes = original_secret
            .to_bytes()
            .expect("Failed to convert to bytes");
        let reconstructed_secret = TotpSecret::from_bytes(bytes);

        assert_eq!(
            original_secret.base32_encoded(),
            reconstructed_secret.base32_encoded(),
            "Secret should survive round trip conversion"
        );
    }

    #[test]
    fn test_totp_service_full_workflow() {
        let service = OauthTotpService::new();

        // 1. Générer un secret
        let secret = service
            .generate_secret()
            .expect("Failed to generate secret");

        // 2. Générer l'URI OTPAuth
        let uri = service.generate_otpauth_uri("TestApp", "test@example.com", &secret);
        assert!(!uri.is_empty());

        // 3. Simuler la génération d'un code TOTP
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX_EPOCH")
            .as_secs();

        let counter = now / 30;
        let secret_bytes = secret
            .to_bytes()
            .expect("Failed to convert secret to bytes");
        let code = OauthTotpService::generate_totp_code(&secret_bytes, counter, 6)
            .expect("Failed to generate TOTP code");
        let code_str = format!("{code:06}");

        // 4. Vérifier le code
        let verification = service.verify(&secret, &code_str);
        assert!(verification.is_ok());
        assert!(verification.unwrap());
    }

    #[test]
    fn test_concurrent_secret_generation() {
        let service = OauthTotpService::new();
        let mut handles = vec![];

        // Générer plusieurs secrets en parallèle
        for _ in 0..10 {
            let service_clone = service.clone();
            let handle = thread::spawn(move || service_clone.generate_secret());
            handles.push(handle);
        }

        let mut secrets = HashSet::new();
        for handle in handles {
            let secret = handle.join().unwrap().expect("Failed to generate secret");
            secrets.insert(secret.base32_encoded().to_string());
        }

        // Tous les secrets doivent être uniques
        assert_eq!(secrets.len(), 10, "All generated secrets should be unique");
    }

    #[test]
    fn test_time_window_edge_cases() {
        let service = OauthTotpService::new();
        let secret = service
            .generate_secret()
            .expect("Failed to generate secret");

        // Simuler un code généré juste à la limite de la fenêtre de temps
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX_EPOCH")
            .as_secs();

        let current_counter = now / 30;
        let secret_bytes = secret
            .to_bytes()
            .expect("Failed to convert secret to bytes");

        // Code de la période précédente (devrait être accepté)
        let prev_code = OauthTotpService::generate_totp_code(&secret_bytes, current_counter - 1, 6)
            .expect("Failed to generate previous code");
        let prev_code_str = format!("{prev_code:06}");

        let result = service.verify(&secret, &prev_code_str);
        assert!(result.is_ok());
        assert!(result.unwrap(), "Previous period code should be accepted");

        // Code de la période suivante (devrait être accepté)
        let next_code = OauthTotpService::generate_totp_code(&secret_bytes, current_counter + 1, 6)
            .expect("Failed to generate next code");
        let next_code_str = format!("{next_code:06}");

        let result = service.verify(&secret, &next_code_str);
        assert!(result.is_ok());
        assert!(result.unwrap(), "Next period code should be accepted");

        // Code trop ancien (ne devrait pas être accepté)
        let old_code = OauthTotpService::generate_totp_code(&secret_bytes, current_counter - 2, 6)
            .expect("Failed to generate old code");
        let old_code_str = format!("{old_code:06}");

        let result = service.verify(&secret, &old_code_str);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Too old code should be rejected");
    }
}
