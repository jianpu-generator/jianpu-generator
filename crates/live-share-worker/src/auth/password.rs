//! Password rules and the stored hash's format. The hashing itself (PBKDF2
//! through WebCrypto) is `crate::crypto::pbkdf2_sha256`; this module only
//! decides what goes in and how the result is written down and compared.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

/// The only password rule: at least this many characters (Unicode scalar
/// values, so a non-Latin password isn't penalised for its byte length).
pub const MIN_PASSWORD_LENGTH: usize = 12;

/// PBKDF2-HMAC-SHA256 iterations for a new hash. A stored hash records its
/// own count, so raising this later only affects hashes written after.
pub const PBKDF2_ITERATIONS: u32 = 100_000;

/// Random salt bytes per hash.
pub const SALT_LENGTH: usize = 16;

/// Derived hash bytes (SHA-256's output size).
pub const HASH_LENGTH: usize = 32;

const SCHEME: &str = "pbkdf2-sha256";

/// Why a new password is refused, as shown to the person choosing it.
pub fn password_weakness(password: &str) -> Option<String> {
    (password.chars().count() < MIN_PASSWORD_LENGTH)
        .then(|| format!("Use at least {MIN_PASSWORD_LENGTH} characters."))
}

/// A stored password hash, in its self-describing text form
/// `pbkdf2-sha256$<iterations>$<salt base64>$<hash base64>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash {
    pub iterations: u32,
    pub salt: Vec<u8>,
    pub hash: Vec<u8>,
}

impl PasswordHash {
    pub fn encode(&self) -> String {
        format!(
            "{SCHEME}${}${}${}",
            self.iterations,
            STANDARD.encode(&self.salt),
            STANDARD.encode(&self.hash)
        )
    }

    /// `None` for anything `encode` couldn't have written.
    pub fn parse(encoded: &str) -> Option<PasswordHash> {
        let mut parts = encoded.split('$');
        let (Some(SCHEME), Some(iterations), Some(salt), Some(hash), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) else {
            return None;
        };
        Some(PasswordHash {
            iterations: iterations.parse().ok().filter(|count| *count > 0)?,
            salt: STANDARD.decode(salt).ok()?,
            hash: STANDARD.decode(hash).ok()?,
        })
    }

    /// Whether `derived` (the password being checked, hashed with this
    /// hash's salt and iterations) matches, in time that doesn't depend on
    /// where the bytes first differ.
    pub fn matches(&self, derived: &[u8]) -> bool {
        constant_time_eq(&self.hash, derived)
    }
}

/// Compares every byte regardless of where the first difference is, so the
/// time taken says nothing about how much of a guess was right. Only the
/// length (public: always `HASH_LENGTH`) short-circuits.
pub fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .fold(0u8, |difference, (a, b)| difference | (a ^ b))
            == 0
}
