//! Copy shared branding into the crate tree so `include_*!` paths stay inside the package.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let src = manifest.join("../../assets/branding/Logo.png");
    let dst_dir = manifest.join("assets");
    let dst = dst_dir.join("logo.png");

    println!("cargo:rerun-if-changed={}", src.display());

    if src.exists() {
        fs::create_dir_all(&dst_dir).expect("create assets dir");
        fs::copy(&src, &dst).expect("copy Logo.png into nocode-app/assets");
    }
}
