#[path = "locales_gen.rs"]
mod locales_gen;

fn main() {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    locales_gen::generate(&manifest.join("../locales"), &out);
    tauri_build::build()
}
