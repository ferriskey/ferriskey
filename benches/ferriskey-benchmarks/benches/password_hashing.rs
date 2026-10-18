use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ferriskey_core::PasswordHasherRepository;
use ferriskey_security::crypto::ports::HasherRepository;

const PASSWORD: &str = "Correct-horse-battery-7";

fn password_hashing(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime");
    let hasher = PasswordHasherRepository::new();
    let stored = runtime
        .block_on(hasher.hash_password(PASSWORD))
        .expect("hash the password");

    c.bench_function("argon2 hash a password", |b| {
        b.iter(|| {
            runtime
                .block_on(hasher.hash_password(black_box(PASSWORD)))
                .expect("hash the password")
        })
    });

    c.bench_function("argon2 verify a password", |b| {
        b.iter(|| {
            runtime
                .block_on(hasher.verify_password(
                    black_box(PASSWORD),
                    &stored.hash,
                    stored.hash_iterations,
                    &stored.algorithm,
                    &stored.salt,
                ))
                .expect("verify the password")
        })
    });
}

criterion_group!(benches, password_hashing);
criterion_main!(benches);
