use wasm_bindgen::prelude::*;

/// Compress a share-link payload with brotli (quality 11).
///
/// The caller is responsible for base64url-encoding the result for use in a URL.
#[wasm_bindgen]
pub fn compress_share_payload(payload: &str) -> Vec<u8> {
    let params = brotli::enc::BrotliEncoderParams {
        quality: 11,
        ..Default::default()
    };
    let mut output = Vec::new();
    // Writing to an in-memory `Vec<u8>` cannot produce an I/O error, so any
    // `Err` here is unreachable in practice; ignore it rather than panicking.
    if brotli::BrotliCompress(&mut payload.as_bytes(), &mut output, &params).is_err() {
        return Vec::new();
    }
    output
}

/// Decompress a brotli-compressed share-link payload back into a UTF-8 string.
///
/// Returns `None` if `bytes` is not valid brotli, or decompresses to invalid UTF-8.
#[wasm_bindgen]
pub fn decompress_share_payload(bytes: &[u8]) -> Option<String> {
    let mut output = Vec::new();
    brotli::BrotliDecompress(&mut &bytes[..], &mut output).ok()?;
    String::from_utf8(output).ok()
}
