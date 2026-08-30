//! Generates `src/fonts.rs`'s actual constants from `fonts/fonts.json` (the
//! single source of truth for which font file backs each `FontFamily`
//! role — see that file's own comments for the full picture, including the
//! Serif-vs-alias asymmetry). Output lands in `OUT_DIR/fonts_generated.rs`,
//! `include!`-d by `src/fonts.rs`.

use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct FontEntry {
    filename: String,
    name: Option<String>,
    #[serde(rename = "familyCss")]
    family_css: String,
}

#[derive(Deserialize)]
struct FontsManifest {
    serif: FontEntry,
    #[serde(rename = "sansSerif")]
    sans_serif: FontEntry,
    monospace: FontEntry,
}

/// Emits one role's constants into `out`. `bytes_const_doc` documents the
/// `_FONT_BYTES` const; `name_const` is `None` for the `serif` role, which
/// has no `fontdb` generic-alias name (see the manifest's own comment).
fn emit_role(
    out: &mut String,
    manifest_dir: &str,
    role_prefix: &str,
    entry: &FontEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(name) = &entry.name {
        out.push_str(&format!(
            "pub const {role_prefix}_FONT_NAME: &str = {name:?};\n"
        ));
    }
    out.push_str(&format!(
        "pub const {role_prefix}_FONT_FAMILY_CSS: &str = {family_css:?};\n",
        family_css = entry.family_css,
    ));
    let absolute_font_path = PathBuf::from(manifest_dir)
        .join("fonts")
        .join(&entry.filename);
    let absolute_font_path_str = absolute_font_path
        .to_str()
        .ok_or("font path is not valid UTF-8")?;
    out.push_str("#[cfg(not(target_arch = \"wasm32\"))]\n");
    out.push_str(&format!(
        "pub const {role_prefix}_FONT_BYTES: &[u8] = include_bytes!({absolute_font_path_str:?});\n"
    ));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=fonts/fonts.json");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let manifest_json_path = PathBuf::from(&manifest_dir).join("fonts/fonts.json");
    let manifest_json = fs::read_to_string(&manifest_json_path)?;
    let manifest: FontsManifest = serde_json::from_str(&manifest_json)?;

    let mut out = String::new();
    emit_role(&mut out, &manifest_dir, "SERIF", &manifest.serif)?;
    emit_role(&mut out, &manifest_dir, "SANS_SERIF", &manifest.sans_serif)?;
    emit_role(&mut out, &manifest_dir, "MONOSPACE", &manifest.monospace)?;

    let out_dir = env::var("OUT_DIR")?;
    let out_path = PathBuf::from(out_dir).join("fonts_generated.rs");
    fs::write(out_path, out)?;

    Ok(())
}
