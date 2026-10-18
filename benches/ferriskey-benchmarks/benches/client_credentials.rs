use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ferriskey_core::domain::authentication::client_authentication::{
    ClientGrant, authenticate_client,
};
use ferriskey_core::domain::authentication::entities::AuthProtocol;
use ferriskey_core::domain::client::entities::{Client, ClientConfig, ClientType};
use ferriskey_domain::realm::RealmId;
use uuid::Uuid;

const SECRET: &str = "4c1f2d8e9b7a6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b";

fn confidential_client() -> Client {
    Client::new(ClientConfig {
        realm_id: RealmId::from(Uuid::new_v4()),
        name: "orders-api".to_string(),
        client_id: "orders-api".to_string(),
        secret: Some(SECRET.to_string()),
        enabled: true,
        protocol: AuthProtocol::OpenIdConnect,
        public_client: false,
        service_account_enabled: true,
        client_type: ClientType::Confidential,
        direct_access_grants_enabled: None,
        oauth_device_code_grant_enabled: None,
        token_exchange_enabled: None,
        access_token_lifetime: None,
        refresh_token_lifetime: None,
        id_token_lifetime: None,
        temporary_token_lifetime: None,
    })
}

fn client_credentials(c: &mut Criterion) {
    let client = confidential_client();

    c.bench_function(
        "client_credentials: authenticate a confidential client",
        |b| {
            b.iter(|| {
                authenticate_client(
                    black_box(&client),
                    Some(black_box(SECRET)),
                    ClientGrant::ClientCredentials,
                )
                .expect("the secret matches")
            })
        },
    );

    c.bench_function("client_credentials: refuse a wrong client secret", |b| {
        b.iter(|| {
            authenticate_client(
                black_box(&client),
                Some(black_box("not-the-secret")),
                ClientGrant::ClientCredentials,
            )
            .is_err()
        })
    });
}

criterion_group!(benches, client_credentials);
criterion_main!(benches);
