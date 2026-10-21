//! What the worker accepts as an account's email. There is no verification
//! email, so this only catches typos that can't be an address at all; it
//! deliberately doesn't try to be RFC 5322.

/// The longest address SMTP can deliver to (RFC 5321's 254-octet path).
pub const MAX_EMAIL_LENGTH: usize = 254;

/// The form an email is stored and looked up in: trimmed and lowercased,
/// so `Me@Example.com ` and `me@example.com` are one account. `None` when
/// it isn't shaped like an address: exactly one `@`, something before it,
/// a dotted domain after it with no empty label, no whitespace, and at most
/// `MAX_EMAIL_LENGTH` bytes.
pub fn normalize_email(email: &str) -> Option<String> {
    let email = email.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;
    let well_formed = email.len() <= MAX_EMAIL_LENGTH
        && !email.chars().any(char::is_whitespace)
        && !local.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && domain.split('.').all(|label| !label.is_empty());
    well_formed.then_some(email)
}
