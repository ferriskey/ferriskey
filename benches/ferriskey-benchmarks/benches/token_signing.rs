use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ferriskey_security::jwt::entities::{ClaimsTyp, JwtClaim, JwtKeyPair};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

fn access_token_claims() -> JwtClaim {
    JwtClaim::new(
        Uuid::new_v4(),
        "alice".to_string(),
        "https://auth.example/realms/master".to_string(),
        vec!["master-realm".to_string(), "account".to_string()],
        ClaimsTyp::Bearer,
        "security-admin-console".to_string(),
        Some("alice@example.com".to_string()),
        Some("openid profile email".to_string()),
        300,
    )
}

fn token_signing(c: &mut Criterion) {
    let (private_pem, public_pem) = JwtKeyPair::generate().expect("generate an RSA key pair");
    let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes()).expect("encoding key");
    let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes()).expect("decoding key");
    let header = Header::new(Algorithm::RS256);
    let claims = access_token_claims();
    let token = encode(&header, &claims, &encoding_key).expect("sign the access token");
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["master-realm", "account"]);

    c.bench_function("rs256 sign an access token", |b| {
        b.iter(|| {
            encode(&header, black_box(&claims), &encoding_key).expect("sign the access token")
        })
    });

    c.bench_function("rs256 verify an access token", |b| {
        b.iter(|| {
            decode::<JwtClaim>(black_box(&token), &decoding_key, &validation)
                .expect("verify the access token")
        })
    });
}

criterion_group!(benches, token_signing);
criterion_main!(benches);
