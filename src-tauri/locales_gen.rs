//! Build-time list of the locale files, shared by `build.rs` and the tests.
//!
//! Every `*.json` file in `/locales` becomes a language of the Rust side. A
//! new language is one new file, with no code change.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Writes `locales.rs` into `out_dir`: a `LOCALES` list of (code, JSON text).
pub fn generate(locales_dir: &Path, out_dir: &Path) {
    println!("cargo:rerun-if-changed={}", locales_dir.display());
    let mut codes: Vec<String> = fs::read_dir(locales_dir)
        .expect("the locales folder exists")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .filter_map(|path| path.file_stem()?.to_str().map(str::to_string))
        .collect();
    codes.sort();

    let mut source = String::from("/// (language code, locale file text), from /locales.\n");
    source.push_str("pub const LOCALES: &[(&str, &str)] = &[\n");
    for code in &codes {
        let path = locales_dir.join(format!("{code}.json"));
        println!("cargo:rerun-if-changed={}", path.display());
        let path = path.to_string_lossy().replace('\\', "/");
        writeln!(source, "    ({code:?}, include_str!({path:?})),").expect("writing to a string");
    }
    source.push_str("];\n");
    fs::write(out_dir.join("locales.rs"), source).expect("the output folder is writable");
}
