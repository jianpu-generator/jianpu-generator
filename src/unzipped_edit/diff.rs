//! Domain-free token-level LCS diff powering
//! [`super::repack::repack_lyrics_via_diff`]. Kept independent of any
//! Lyrics/measure vocabulary so it can be tested purely as an algorithm (see
//! `tests_diff`).

/// One token of the edited sequence, tagged by how it relates to the
/// original sequence: either an exact match carrying the original measure it
/// came from (`Equal`), or new content with no original counterpart
/// (`Insert`). Deleted original tokens (present in `original_tokens` but
/// dropped from `edited_tokens`) contribute nothing to the walk and are
/// simply absent from the returned list — callers only ever need to know
/// what ends up in the edited output, not what was removed.
pub(super) enum DiffToken {
    Equal {
        edited_index: usize,
        owner_measure: usize,
    },
    Insert {
        edited_index: usize,
    },
}

/// Token-level LCS diff of `original_tokens` against `edited_tokens`,
/// returned in `edited_tokens` order. `original_owner_measure[i]` is which
/// original measure `original_tokens[i]` came from (same length as
/// `original_tokens`).
///
/// Plain O(n*m) dynamic-programming LCS + backtrack (not Myers): verse sizes
/// here are tens/low-hundreds of tokens, and a DP table is simpler to keep
/// under this repo's complexity/length clippy limits than an O(ND) Myers
/// implementation. The backtrack always prefers the diagonal (match) move
/// when one is available, so a fully unedited round trip (`original_tokens
/// == edited_tokens`) deterministically produces an all-`Equal` walk with
/// strictly increasing `owner_measure` — this is what lets
/// `repack_lyrics_via_diff` reproduce untouched content byte-for-byte
/// without ever consulting capacity.
pub(super) fn diff_tokens(
    original_tokens: &[&str],
    original_owner_measure: &[usize],
    edited_tokens: &[&str],
) -> Vec<DiffToken> {
    let table = build_lcs_table(original_tokens, edited_tokens);
    backtrack_edit_script(
        original_tokens,
        original_owner_measure,
        edited_tokens,
        &table,
    )
}

/// LCS length table as a flat `(n+1)*(m+1)` grid, `table[i * (m+1) + j]` =
/// LCS length of `original_tokens[..i]` against `edited_tokens[..j]`.
/// Accessed only via `.get()`/`.get_mut()` (never `table[i][j]`) to satisfy
/// `clippy::indexing_slicing`.
fn build_lcs_table(original_tokens: &[&str], edited_tokens: &[&str]) -> Vec<u32> {
    let n = original_tokens.len();
    let m = edited_tokens.len();
    let width = m + 1;
    let mut table = vec![0u32; (n + 1) * width];

    for i in 1..=n {
        for j in 1..=m {
            let is_match = original_tokens.get(i - 1) == edited_tokens.get(j - 1);
            let value = if is_match {
                table.get((i - 1) * width + (j - 1)).copied().unwrap_or(0) + 1
            } else {
                let up = table.get((i - 1) * width + j).copied().unwrap_or(0);
                let left = table.get(i * width + (j - 1)).copied().unwrap_or(0);
                up.max(left)
            };
            if let Some(cell) = table.get_mut(i * width + j) {
                *cell = value;
            }
        }
    }
    table
}

/// Walks `table` from `(n, m)` back to `(0, 0)`, preferring the diagonal
/// (match) move whenever one is available at a cell, and returns the
/// resulting edit script in forward (`edited_tokens`-increasing) order.
fn backtrack_edit_script(
    original_tokens: &[&str],
    original_owner_measure: &[usize],
    edited_tokens: &[&str],
    table: &[u32],
) -> Vec<DiffToken> {
    let n = original_tokens.len();
    let m = edited_tokens.len();
    let width = m + 1;
    let cell = |i: usize, j: usize| table.get(i * width + j).copied().unwrap_or(0);

    let mut script = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 || j > 0 {
        let is_match = i > 0 && j > 0 && original_tokens.get(i - 1) == edited_tokens.get(j - 1);
        if is_match && cell(i, j) == cell(i - 1, j - 1) + 1 {
            i -= 1;
            j -= 1;
            let owner_measure = original_owner_measure.get(i).copied().unwrap_or(0);
            script.push(DiffToken::Equal {
                edited_index: j,
                owner_measure,
            });
        } else if j > 0 && (i == 0 || cell(i, j - 1) >= cell(i - 1, j)) {
            j -= 1;
            script.push(DiffToken::Insert { edited_index: j });
        } else if i > 0 {
            // Deleted original token: contributes nothing to the edited walk.
            i -= 1;
        } else {
            break;
        }
    }
    script.reverse();
    script
}

#[cfg(test)]
mod tests_diff;
