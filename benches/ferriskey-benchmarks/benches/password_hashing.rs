use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ferriskey_core::{hash_argon2_password, verify_argon2_password};
use ferriskey_domain::crypto::PasswordHashingCost;

const PASSWORD: &str = "Correct-horse-battery-7";

fn password_hashing(c: &mut Criterion) {
    let cost = PasswordHashingCost::default();
    let stored = hash_argon2_password(PASSWORD, cost).expect("hash the password");

    c.bench_function("argon2 hash a password", |b| {
        b.iter(|| hash_argon2_password(black_box(PASSWORD), cost).expect("hash the password"))
    });

    c.bench_function("argon2 verify a password", |b| {
        b.iter(|| {
            verify_argon2_password(black_box(PASSWORD), &stored.hash, &stored.algorithm)
                .expect("verify the password")
        })
    });
}

criterion_group!(benches, password_hashing);
criterion_main!(benches);
