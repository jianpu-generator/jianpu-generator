//! Loads and caches the parsed `ttf_parser::Face`s that back
//! `font_metrics`'s glyph-advance measurements — one implementation for
//! non-wasm builds (fonts embedded at compile time) and one for wasm (fonts
//! fetched and pushed in at runtime). See `super`'s own doc comment for which
//! font backs which role.

// Non-wasm builds (the CLI, `cargo test`) embed the fonts at compile time,
// exactly as before.
#[cfg(not(target_arch = "wasm32"))]
mod imp {
    /// The font pinned for directive-line text (see
    /// `DIRECTIVE_LINE_FONT_FAMILY` in `src/serializer/mod.rs`), parsed once
    /// so its real glyph advance widths can be used instead of a
    /// character-bucket heuristic. `None` only if the embedded font fails to
    /// parse, which shouldn't happen for a file fixed at compile time.
    /// A character missing from it measures via
    /// `FALLBACK_ADVANCE_WIDTH_RATIO` instead of a fallback font's real
    /// advance, since this single-face measurement has no fallback chain of
    /// its own. `FontFamily::Serif` (see `lyric_font` below) isn't measured
    /// here — see its doc comment in `src/compositor/types.rs`.
    static DIRECTIVE_LINE_FONT: std::sync::LazyLock<Option<ttf_parser::Face<'static>>> =
        std::sync::LazyLock::new(|| {
            ttf_parser::Face::parse(crate::fonts::SANS_SERIF_FONT_BYTES, 0).ok()
        });

    /// The font pinned for lyric syllables/lines (see `SERIF_FONT_FAMILY` in
    /// `src/serializer/mod.rs` — lyrics share the song title's font), parsed
    /// once for the same real-glyph-advance reason as `DIRECTIVE_LINE_FONT`.
    /// The song title itself isn't measured here — see its doc comment in
    /// `src/compositor/types.rs`.
    static LYRIC_FONT: std::sync::LazyLock<Option<ttf_parser::Face<'static>>> =
        std::sync::LazyLock::new(|| {
            ttf_parser::Face::parse(crate::fonts::SERIF_FONT_BYTES, 0).ok()
        });

    /// The font pinned for monospace glyphs (notehead digits, rests, chord
    /// symbols, note dashes, Latin lyric syllables — see
    /// `FontFamily::Monospace` resolving to `"Noto Sans Mono", monospace` in
    /// `src/serializer/mod.rs`), parsed once so layout weights can be
    /// measured against the same font that actually renders.
    static MONOSPACE_FONT: std::sync::LazyLock<Option<ttf_parser::Face<'static>>> =
        std::sync::LazyLock::new(|| {
            ttf_parser::Face::parse(crate::fonts::MONOSPACE_FONT_BYTES, 0).ok()
        });

    pub(crate) fn directive_line_font() -> Option<&'static ttf_parser::Face<'static>> {
        DIRECTIVE_LINE_FONT.as_ref()
    }

    pub(crate) fn lyric_font() -> Option<&'static ttf_parser::Face<'static>> {
        LYRIC_FONT.as_ref()
    }

    pub(crate) fn monospace_font() -> Option<&'static ttf_parser::Face<'static>> {
        MONOSPACE_FONT.as_ref()
    }

    /// No-op on non-wasm builds: the font is already embedded at compile
    /// time, so there's nothing to receive at runtime. Exists so callers
    /// (e.g. `crates/jianpu-wasm`, which is also built for the host arch as
    /// a workspace member) don't need their own `cfg` gate.
    pub(crate) fn set_directive_line_font_bytes(_bytes: Vec<u8>) {}

    /// No-op on non-wasm builds — see `set_directive_line_font_bytes`.
    pub(crate) fn set_lyric_font_bytes(_bytes: Vec<u8>) {}

    /// No-op on non-wasm builds — see `set_directive_line_font_bytes`.
    pub(crate) fn set_monospace_font_bytes(_bytes: Vec<u8>) {}
}

// The wasm build has no compile-time font bytes: `set_directive_line_font_bytes`/
// `set_monospace_font_bytes` are called at runtime (from `crates/jianpu-wasm`)
// once the app has fetched the same font bytes it already needs for PDF
// export. Two `OnceLock`s per font (rather than a `LazyLock`) so a render
// that races ahead of the fetch just falls back to
// `FALLBACK_ADVANCE_WIDTH_RATIO` for that one call, instead of a `LazyLock`
// permanently caching `None` if it happened to be evaluated before the bytes
// arrived.
#[cfg(target_arch = "wasm32")]
mod imp {
    use std::sync::OnceLock;

    static DIRECTIVE_LINE_FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    static DIRECTIVE_LINE_FONT_FACE: OnceLock<ttf_parser::Face<'static>> = OnceLock::new();
    static LYRIC_FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    static LYRIC_FONT_FACE: OnceLock<ttf_parser::Face<'static>> = OnceLock::new();
    static MONOSPACE_FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    static MONOSPACE_FONT_FACE: OnceLock<ttf_parser::Face<'static>> = OnceLock::new();

    pub(crate) fn set_directive_line_font_bytes(bytes: Vec<u8>) {
        DIRECTIVE_LINE_FONT_BYTES.set(bytes).ok();
    }

    pub(crate) fn set_lyric_font_bytes(bytes: Vec<u8>) {
        LYRIC_FONT_BYTES.set(bytes).ok();
    }

    pub(crate) fn set_monospace_font_bytes(bytes: Vec<u8>) {
        MONOSPACE_FONT_BYTES.set(bytes).ok();
    }

    fn face_from(
        bytes_cell: &'static OnceLock<Vec<u8>>,
        face_cell: &'static OnceLock<ttf_parser::Face<'static>>,
    ) -> Option<&'static ttf_parser::Face<'static>> {
        if let Some(face) = face_cell.get() {
            return Some(face);
        }
        let bytes = bytes_cell.get()?;
        let face = ttf_parser::Face::parse(bytes, 0).ok()?;
        Some(face_cell.get_or_init(|| face))
    }

    pub(crate) fn directive_line_font() -> Option<&'static ttf_parser::Face<'static>> {
        face_from(&DIRECTIVE_LINE_FONT_BYTES, &DIRECTIVE_LINE_FONT_FACE)
    }

    pub(crate) fn lyric_font() -> Option<&'static ttf_parser::Face<'static>> {
        face_from(&LYRIC_FONT_BYTES, &LYRIC_FONT_FACE)
    }

    pub(crate) fn monospace_font() -> Option<&'static ttf_parser::Face<'static>> {
        face_from(&MONOSPACE_FONT_BYTES, &MONOSPACE_FONT_FACE)
    }
}

pub(crate) use imp::{
    directive_line_font, lyric_font, monospace_font, set_directive_line_font_bytes,
    set_lyric_font_bytes, set_monospace_font_bytes,
};
