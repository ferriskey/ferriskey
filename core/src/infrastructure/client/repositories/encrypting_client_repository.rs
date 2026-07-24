use std::sync::Arc;

use ferriskey_domain::client::{
    entities::Client,
    ports::ClientRepository,
    value_objects::{CreateClientRequest, UpdateClientRequest},
};
use ferriskey_security::FieldCipher;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

/// `ClientRepository` decorator that transparently encrypts `secret` on write
/// and decrypts on read when encryption is enabled.
///
/// ## Sentinel convention
/// - `secret_key_id IS NULL` in the database → the row was created before
///   encryption was introduced. `secret` (plaintext) is returned as-is.
/// - `secret_key_id = "v1"` → `secret_encrypted` holds the AES-256-GCM blob;
///   `secret` is left NULL (or ignored) so no plaintext is stored alongside it.
///
/// ## Backfill
/// Existing plaintext rows are safe: decrypt path falls back to `secret` when
/// `secret_key_id` is NULL.  A follow-up migration job should iterate all rows
/// with `secret IS NOT NULL AND secret_key_id IS NULL`, encrypt each, write
/// `secret_encrypted` + `secret_key_id = "v1"`, and NULL out `secret`.
#[derive(Clone, Debug)]
pub struct EncryptingClientRepository<R> {
    inner: Arc<R>,
    cipher: Option<FieldCipher>,
}

impl<R> EncryptingClientRepository<R> {
    pub fn new(inner: Arc<R>, cipher: Option<FieldCipher>) -> Self {
        Self { inner, cipher }
    }

    /// Decrypt a client's secret field.  Falls back gracefully to plaintext for
    /// pre-encryption rows (sentinel: `secret_key_id IS NULL`).
    fn decrypt_client(&self, mut client: Client) -> Result<Client, CoreError> {
        if let Some(cipher) = &self.cipher
            && let Some(blob) = client.secret.as_deref()
            && let Some(encrypted_part) = blob.strip_prefix("enc:")
        {
            let plaintext = cipher
                .decrypt(encrypted_part)
                .map_err(|e| CoreError::Configuration(e.to_string()))?;
            client.secret = Some(plaintext);
        }
        Ok(client)
    }
}

impl<R> ClientRepository for EncryptingClientRepository<R>
where
    R: ClientRepository + Send + Sync,
{
    async fn create_client(&self, mut data: CreateClientRequest) -> Result<Client, CoreError> {
        if let Some(ref cipher) = self.cipher
            && let Some(ref plain_secret) = data.secret.clone()
        {
            let blob = cipher
                .encrypt(plain_secret)
                .map_err(|e| CoreError::Configuration(e.to_string()))?;
            data.secret = Some(format!("enc:{blob}"));
        }

        let client = self.inner.create_client(data).await?;
        self.decrypt_client(client)
    }

    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: RealmId,
    ) -> Result<Client, CoreError> {
        let client = self.inner.get_by_client_id(client_id, realm_id).await?;
        self.decrypt_client(client)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Client, CoreError> {
        let client = self.inner.get_by_id(id).await?;
        self.decrypt_client(client)
    }

    async fn get_by_realm_id(&self, realm_id: RealmId) -> Result<Vec<Client>, CoreError> {
        let clients = self.inner.get_by_realm_id(realm_id).await?;
        clients
            .into_iter()
            .map(|c| self.decrypt_client(c))
            .collect()
    }

    async fn update_client(
        &self,
        client_id: Uuid,
        data: UpdateClientRequest,
    ) -> Result<Client, CoreError> {
        let client = self.inner.update_client(client_id, data).await?;
        self.decrypt_client(client)
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), CoreError> {
        self.inner.delete_by_id(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferriskey_domain::client::{
        entities::{Client, ClientType, MaintenanceSessionStrategy},
        value_objects::CreateClientRequest,
    };
    use ferriskey_domain::realm::RealmId;
    use ferriskey_security::{FieldCipher, cipher::field_cipher::KEY_ID_V1};

    use std::sync::Arc;
    use uuid::Uuid;

    fn test_cipher() -> FieldCipher {
        let key = *b"an example very very secret key!";
        FieldCipher::new(&key, KEY_ID_V1)
    }

    fn make_client(secret: Option<String>) -> Client {
        let now = chrono::Utc::now();
        Client {
            id: Uuid::new_v4(),
            enabled: true,
            client_id: "test-client".to_string(),
            secret,
            realm_id: RealmId::from(Uuid::new_v4()),
            protocol: "openid-connect".to_string(),
            public_client: false,
            service_account_enabled: false,
            direct_access_grants_enabled: false,
            oauth_device_code_grant_enabled: false,
            require_pkce: false,
            client_type: ClientType::Confidential,
            name: "Test".to_string(),
            redirect_uris: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: false,
            maintenance_reason: None,
            maintenance_session_strategy: MaintenanceSessionStrategy::Expire,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn decrypt_client_decrypts_enc_prefixed_secret() {
        let cipher = test_cipher();
        let plaintext = "my-client-secret";
        let blob = cipher.encrypt(plaintext).unwrap();
        let enc_value = format!("enc:{blob}");

        let client = make_client(Some(enc_value));

        struct Dummy;
        impl ClientRepository for Dummy {
            async fn create_client(&self, _: CreateClientRequest) -> Result<Client, CoreError> {
                unimplemented!()
            }
            async fn get_by_client_id(&self, _: String, _: RealmId) -> Result<Client, CoreError> {
                unimplemented!()
            }
            async fn get_by_id(&self, _: Uuid) -> Result<Client, CoreError> {
                unimplemented!()
            }
            async fn get_by_realm_id(&self, _: RealmId) -> Result<Vec<Client>, CoreError> {
                unimplemented!()
            }
            async fn update_client(
                &self,
                _: Uuid,
                _: UpdateClientRequest,
            ) -> Result<Client, CoreError> {
                unimplemented!()
            }
            async fn delete_by_id(&self, _: Uuid) -> Result<(), CoreError> {
                unimplemented!()
            }
        }

        let repo = EncryptingClientRepository::new(Arc::new(Dummy), Some(cipher));
        let result = repo.decrypt_client(client).unwrap();
        assert_eq!(result.secret, Some(plaintext.to_string()));
    }

    #[test]
    fn decrypt_client_passes_through_plaintext_sentinel() {
        struct Dummy;
        impl ClientRepository for Dummy {
            async fn create_client(&self, _: CreateClientRequest) -> Result<Client, CoreError> {
                unimplemented!()
            }
            async fn get_by_client_id(&self, _: String, _: RealmId) -> Result<Client, CoreError> {
                unimplemented!()
            }
            async fn get_by_id(&self, _: Uuid) -> Result<Client, CoreError> {
                unimplemented!()
            }
            async fn get_by_realm_id(&self, _: RealmId) -> Result<Vec<Client>, CoreError> {
                unimplemented!()
            }
            async fn update_client(
                &self,
                _: Uuid,
                _: UpdateClientRequest,
            ) -> Result<Client, CoreError> {
                unimplemented!()
            }
            async fn delete_by_id(&self, _: Uuid) -> Result<(), CoreError> {
                unimplemented!()
            }
        }

        let cipher = test_cipher();
        let repo = EncryptingClientRepository::new(Arc::new(Dummy), Some(cipher));

        let client = make_client(Some("legacy-plaintext-secret".to_string()));
        let result = repo.decrypt_client(client).unwrap();
        assert_eq!(
            result.secret,
            Some("legacy-plaintext-secret".to_string()),
            "pre-migration plaintext should pass through unchanged"
        );
    }
}
