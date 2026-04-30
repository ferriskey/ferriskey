use divan::{self, black_box};
use maskass::{EmailMask, FullMask, HashMask, MaskStrategy, PartialMask};

fn main() {
    divan::main();
}

#[divan::bench]
fn full_mask() -> String {
    FullMask::mask(black_box("super-secret-password-value"))
}

#[divan::bench]
fn email_mask() -> String {
    EmailMask::mask(black_box("user@example.com"))
}

#[divan::bench]
fn email_mask_no_at() -> String {
    EmailMask::mask(black_box("not-an-email"))
}

#[divan::bench]
fn partial_mask_short() -> String {
    PartialMask::<2, 2>::mask(black_box("123456789"))
}

#[divan::bench]
fn partial_mask_long() -> String {
    PartialMask::<2, 2>::mask(black_box("a]3kL9#mQw!xR7zT&vBn2^pYf*hD5jS"))
}

#[divan::bench]
fn hash_mask() -> String {
    HashMask::mask(black_box("user@example.com"))
}
