//! The `# score` directive-line vocabulary (see `syntax.md`): `key=value`
//! assignments like `bpm=92`, plus the bare `break`.

keyword_enum! {
    /// The key of a `key=value` directive-line token.
    pub enum DirectiveKey {
        Bpm => "bpm",
        Key => "key",
        Time => "time",
        Label => "label",
        MergeDuplicateMeasuresAcrossParts => "merge_duplicate_measures_across_parts",
        HideRestingParts => "hide_resting_parts",
    }
}

/// The bare directive-line token that forces a system break.
pub const BREAK_KEYWORD: &str = "break";

/// One recognized directive-line token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveToken<'a> {
    Assignment { key: DirectiveKey, value: &'a str },
    Break,
}

impl DirectiveToken<'_> {
    /// Byte length of the token's leading keyword (`bpm` in `bpm=92`).
    pub fn keyword_len(&self) -> usize {
        match self {
            Self::Assignment { key, .. } => key.keyword().len(),
            Self::Break => BREAK_KEYWORD.len(),
        }
    }
}

/// Classifies one whitespace-separated directive-line token, or `None` when
/// it is not a directive.
pub fn classify_directive_token(token: &str) -> Option<DirectiveToken<'_>> {
    if token == BREAK_KEYWORD {
        return Some(DirectiveToken::Break);
    }
    let (key, value) = token.split_once('=')?;
    DirectiveKey::from_keyword(key).map(|key| DirectiveToken::Assignment { key, value })
}
