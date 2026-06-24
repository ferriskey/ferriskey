use secrecy::SecretString;

use crate::SecurityError;

/// Abstraction over any secret-delivery backend (env vars, files, Vault, KMS …).
///
/// ## Implementations shipped in this crate
/// - [`EnvSecretsProvider`] — reads secrets from environment variables or file
///   paths.  This is the default for most deployments.
///
/// ## Vault / KMS integration path (follow-up)
/// To add HashiCorp Vault support, implement this trait as `VaultSecretsProvider`
/// in a new crate (or behind a feature flag in this one). The `get_secret` method
/// would call the Vault HTTP API (`GET /v1/secret/data/<name>`) and return the
/// decoded value. Key lifecycle — rotation and re-encryption — should be wired
/// through a separate job that iterates encrypted rows, decrypts with the old
/// key, and re-encrypts with the new key before bumping `secret_key_id`.
///
/// All Vault or KMS communication MUST be over TLS; the TLS certificates for
/// the Vault endpoint should be pinned or validated against the system trust
/// store, never skipped.
pub trait SecretsProvider: Send + Sync {
    /// Retrieve a named secret. Fails closed: returns `Err` if the secret is
    /// absent rather than returning an empty string.
    fn get_secret(&self, name: &str) -> Result<SecretString, SecurityError>;
}

/// Reads secrets from environment variables.
///
/// The variable name is `FERRISKEY_SECRET_<NAME>` where `<NAME>` is the
/// upper-cased form of the requested secret name. For example, requesting
/// `"master_key"` looks up `FERRISKEY_SECRET_MASTER_KEY`.
///
/// An alternative file-based path is also supported: if the environment variable
/// `FERRISKEY_SECRET_<NAME>_FILE` is set, the secret is read from that file
/// path. This is convenient for Docker secrets and Kubernetes `secretKeyRef`
/// volume mounts.
#[derive(Debug, Clone, Default)]
pub struct EnvSecretsProvider;

impl EnvSecretsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl SecretsProvider for EnvSecretsProvider {
    fn get_secret(&self, name: &str) -> Result<SecretString, SecurityError> {
        let env_name = format!("FERRISKEY_SECRET_{}", name.to_uppercase().replace('-', "_"));
        let file_env_name = format!("{env_name}_FILE");

        if let Ok(file_path) = std::env::var(&file_env_name) {
            let contents = std::fs::read_to_string(&file_path).map_err(|e| {
                SecurityError::InvalidKey(format!("failed to read secret file {file_path}: {e}"))
            })?;
            return Ok(SecretString::new(contents.trim().to_string()));
        }

        let value = std::env::var(&env_name).map_err(|_| {
            SecurityError::InvalidKey(format!(
                "secret '{name}' not found (set {env_name} or {file_env_name})"
            ))
        })?;

        Ok(SecretString::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn env_secrets_provider_returns_value() {
        // SAFETY: single-threaded test; no other thread races on this env var.
        unsafe { std::env::set_var("FERRISKEY_SECRET_TEST_KEY_42", "my-secret-value") };
        let provider = EnvSecretsProvider::new();
        let secret = provider.get_secret("test_key_42").expect("should find key");
        assert_eq!(secret.expose_secret(), "my-secret-value");
        // SAFETY: same as above.
        unsafe { std::env::remove_var("FERRISKEY_SECRET_TEST_KEY_42") };
    }

    #[test]
    fn env_secrets_provider_missing_key_fails_closed() {
        // SAFETY: single-threaded test.
        unsafe { std::env::remove_var("FERRISKEY_SECRET_NONEXISTENT_KEY_XYZ") };
        let provider = EnvSecretsProvider::new();
        let result = provider.get_secret("nonexistent_key_xyz");
        assert!(result.is_err(), "missing key must fail closed");
    }

    #[test]
    fn env_secrets_provider_hyphen_normalized() {
        // SAFETY: single-threaded test.
        unsafe { std::env::set_var("FERRISKEY_SECRET_MY_KEY", "hyphen-ok") };
        let provider = EnvSecretsProvider::new();
        let secret = provider
            .get_secret("my-key")
            .expect("hyphen should normalize");
        assert_eq!(secret.expose_secret(), "hyphen-ok");
        // SAFETY: same as above.
        unsafe { std::env::remove_var("FERRISKEY_SECRET_MY_KEY") };
    }
}
