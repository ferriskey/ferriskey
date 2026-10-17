#[cfg_attr(
    not(test),
    expect(dead_code, reason = "wired with the evaluation endpoint")
)]
pub mod principal_slice;
pub mod repository;
