use ferriskey_security::crypto::secret_cipher::{SecretCipher, SecretCipherError};
use serde_json::Value;
use uuid::Uuid;

pub fn seal_field(
    cipher: &SecretCipher,
    purpose: &str,
    row: Uuid,
    document: &mut Value,
    pointer: &str,
) -> Result<(), SecretCipherError> {
    if let Some(Value::String(value)) = document.pointer_mut(pointer) {
        *value = cipher.seal(purpose, &row.to_string(), value)?;
    }

    Ok(())
}

pub fn open_field(
    cipher: &SecretCipher,
    purpose: &str,
    row: Uuid,
    document: &mut Value,
    pointer: &str,
) -> Result<(), SecretCipherError> {
    if let Some(Value::String(value)) = document.pointer_mut(pointer) {
        *value = cipher.open(purpose, &row.to_string(), value)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const KEY_A: &str = "kid-a:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";
    const POINTER: &str = "/bind/bind_password_encrypted";

    fn cipher() -> SecretCipher {
        SecretCipher::from_config(KEY_A).expect("a valid key ring")
    }

    #[test]
    fn only_the_pointed_string_is_sealed_and_it_opens_back() {
        let row = Uuid::new_v4();
        let mut document =
            json!({"bind": {"bind_dn": "cn=admin", "bind_password_encrypted": "s3cr3t"}});

        seal_field(&cipher(), "p", row, &mut document, POINTER).unwrap();

        let sealed = document.pointer(POINTER).and_then(Value::as_str).unwrap();
        assert!(SecretCipher::is_sealed(sealed), "{sealed}");
        assert_eq!(document["bind"]["bind_dn"], "cn=admin");

        open_field(&cipher(), "p", row, &mut document, POINTER).unwrap();
        assert_eq!(document["bind"]["bind_password_encrypted"], "s3cr3t");
    }

    #[test]
    fn a_missing_or_non_string_field_is_left_alone() {
        let mut absent = json!({"bind": {"bind_dn": "cn=admin"}});
        let mut null = json!({"bind": {"bind_password_encrypted": null}});

        seal_field(&cipher(), "p", Uuid::nil(), &mut absent, POINTER).unwrap();
        seal_field(&cipher(), "p", Uuid::nil(), &mut null, POINTER).unwrap();

        assert_eq!(absent, json!({"bind": {"bind_dn": "cn=admin"}}));
        assert_eq!(null, json!({"bind": {"bind_password_encrypted": null}}));
    }

    #[test]
    fn a_sealed_field_does_not_open_on_another_row() {
        let mut document = json!({"client_secret": "s3cr3t"});
        seal_field(
            &cipher(),
            "p",
            Uuid::new_v4(),
            &mut document,
            "/client_secret",
        )
        .unwrap();

        let result = open_field(
            &cipher(),
            "p",
            Uuid::new_v4(),
            &mut document,
            "/client_secret",
        );

        assert_eq!(result, Err(SecretCipherError::OpenFailed));
    }
}
