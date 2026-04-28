use ferriskey_security::jwt::entities::{
    ClaimsTyp, DEFAULT_ACCESS_TOKEN_LIFETIME, DEFAULT_REFRESH_TOKEN_LIFETIME, JwtClaim, JwtKeyPair,
};
use jsonwebtoken::{Algorithm, Header, Validation, decode, encode};
use uuid::Uuid;

fn main() {
    divan::main();
}

fn setup_key_pair() -> JwtKeyPair {
    let (private_pem, public_pem) = JwtKeyPair::generate().expect("RSA key generation failed");
    JwtKeyPair::from_pem(&private_pem, &public_pem, Uuid::new_v4(), Uuid::new_v4())
        .expect("key pair load failed")
}

fn make_access_claims() -> JwtClaim {
    JwtClaim::new(
        Uuid::new_v4(),
        "bench_user".to_string(),
        "https://ferriskey.example.com/realms/master".to_string(),
        vec!["bench-client".to_string()],
        ClaimsTyp::Bearer,
        "bench-client".to_string(),
        Some("bench@example.com".to_string()),
        Some("openid profile email".to_string()),
        DEFAULT_ACCESS_TOKEN_LIFETIME,
    )
}

/// Measures RS256 signing of an access token.
/// Key pair setup is outside the hot path.
#[divan::bench]
fn sign_rs256(bencher: divan::Bencher) {
    let key_pair = setup_key_pair();

    bencher
        .with_inputs(make_access_claims)
        .bench_values(|claims| {
            encode(
                &Header::new(Algorithm::RS256),
                &claims,
                &key_pair.encoding_key,
            )
            .expect("sign failed")
        });
}

/// Measures RS256 verification of a pre-signed access token.
#[divan::bench]
fn verify_rs256(bencher: divan::Bencher) {
    let key_pair = setup_key_pair();
    let token = encode(
        &Header::new(Algorithm::RS256),
        &make_access_claims(),
        &key_pair.encoding_key,
    )
    .expect("sign failed");

    bencher.bench(|| {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&["bench-client"]);
        decode::<JwtClaim>(&token, &key_pair.decoding_key, &validation).expect("verify failed")
    });
}

/// Measures the full sign → verify roundtrip.
#[divan::bench]
fn roundtrip(bencher: divan::Bencher) {
    let key_pair = setup_key_pair();

    bencher
        .with_inputs(make_access_claims)
        .bench_values(|claims| {
            let token = encode(
                &Header::new(Algorithm::RS256),
                &claims,
                &key_pair.encoding_key,
            )
            .expect("sign failed");
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_audience(&["bench-client"]);
            decode::<JwtClaim>(&token, &key_pair.decoding_key, &validation).expect("verify failed")
        });
}

/// Measures signing of a refresh token (no identity claims).
#[divan::bench]
fn sign_refresh_token(bencher: divan::Bencher) {
    let key_pair = setup_key_pair();

    bencher
        .with_inputs(|| {
            JwtClaim::new_refresh_token(
                Uuid::new_v4(),
                "https://ferriskey.example.com/realms/master".to_string(),
                vec!["bench-client".to_string()],
                "bench-client".to_string(),
                Some("openid".to_string()),
                DEFAULT_REFRESH_TOKEN_LIFETIME,
            )
        })
        .bench_values(|claims| {
            encode(
                &Header::new(Algorithm::RS256),
                &claims,
                &key_pair.encoding_key,
            )
            .expect("sign failed")
        });
}
