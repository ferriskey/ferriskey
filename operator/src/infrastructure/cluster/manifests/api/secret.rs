use std::collections::BTreeMap;

use k8s_openapi::{ByteString, api::core::v1::Secret};
use kube::api::ObjectMeta;
use rand::seq::{IndexedRandom, SliceRandom};

use crate::domain::cluster::ClusterSpec;

fn generate_password(length: usize) -> String {
    let length = length.max(12);

    // Character sets
    let lowercase = b"abcdefghijklmnopqrstuvwxyz";
    let uppercase = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let digits = b"0123456789";
    let symbols = b"!@$?";

    let mut rng = rand::rng();

    // Ensure at least one from each category
    let mut password: Vec<u8> = vec![
        *lowercase.choose(&mut rng).unwrap(),
        *uppercase.choose(&mut rng).unwrap(),
        *digits.choose(&mut rng).unwrap(),
        //*symbols.choose(&mut rng).unwrap(),
    ];

    // Pool of all allowed characters
    let all: Vec<u8> = lowercase
        .iter()
        .chain(uppercase.iter())
        .chain(digits.iter())
        .chain(symbols.iter())
        .copied()
        .collect();

    // Fill the rest
    for _ in password.len()..length {
        password.push(*all.choose(&mut rng).unwrap());
    }

    // Shuffle so guaranteed chars are not predictable
    password.shuffle(&mut rng);

    String::from_utf8(password).unwrap()
}

pub fn make_admin_secret(spec: &ClusterSpec, namespace: &str) -> Secret {
    let secret_name = format!("ferriskey-admin-{}", spec.name);

    let mut data = BTreeMap::new();

    let random_password = generate_password(16);

    data.insert(
        "password".to_string(),
        ByteString(random_password.as_bytes().to_vec()),
    );

    Secret {
        metadata: ObjectMeta {
            name: Some(secret_name),
            namespace: Some(namespace.to_string()),
            labels: Some(BTreeMap::from([
                ("app".to_string(), format!("ferriskey-{}", spec.name)),
                ("component".to_string(), "admin-secret".to_string()),
            ])),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    }
}
