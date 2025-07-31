use std::{os::unix::process::CommandExt, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-env-changed=FORCE_REBUILD");
    println!("cargo:rerun-if-changed=migrations");

    let _ = Command::new("typeshare")
        .args([
            "./",
            "--lang=typescript",
            "--output-file=../front/src/api/api.interface.ts",
        ])
        .exec();
}
