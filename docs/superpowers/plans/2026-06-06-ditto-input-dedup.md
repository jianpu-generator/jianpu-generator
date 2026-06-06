# Ditto Input Deduplication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `"` ditto token to score lines that resolves to the content of the closest preceding line of the same column type, eliminating manual repetition of notes/lyrics across voice parts.

**Architecture:** A new `desugar` module performs raw-line substitution on the groups produced by `collect_groups` before the interleaved parser tokenizes them. The interleaved parser, grouper, and all downstream stages are unchanged — they never see a ditto token. The desugar pass takes `(groups, parts_declaration)` and returns resolved groups or an error.

**Tech Stack:** Rust, existing `JianPuError`/`Span` error types, `PartColumn` enum from `src/ast/parsed.rs`.

---

### Task 1: Create `src/desugar.rs` — notes ditto

**Files:**
- Create: `src/desugar.rs`
- Modify: `src/main.rs` (add `mod desugar;`)

- [ ] **Step 1: Add `mod desugar;` to `src/main.rs`**

Open `src/main.rs` and add after the existing `mod` declarations (e.g. after `mod combiner;`):

```rust
mod desugar;
```

- [ ] **Step 2: Create `src/desugar.rs` with a failing test for notes ditto**

```rust
use crate::ast::parsed::PartColumn;
use crate::error::{JianPuError, Span};

/// Resolves `"` ditto lines within each measure group.
///
/// A `"` on a data line means "same content as the closest preceding line of
/// the same column type in this group." The directive line (starts with `(`)
/// is never a ditto source or target.
///
/// `parts` maps each data-line position to its column type.
pub fn desugar_groups(
    groups: Vec<Vec<(String, usize)>>,
    parts: &[PartColumn],
) -> Result<Vec<Vec<(String, usize)>>, JianPuError> {
    groups
        .into_iter()
        .map(|group| desugar_group(group, parts))
        .collect()
}

fn desugar_group(
    group: Vec<(String, usize)>,
    parts: &[PartColumn],
) -> Result<Vec<(String, usize)>, JianPuError> {
    // Directive line (starts with `(`) is never a ditto target — pass it through.
    let directive_count = if group.first().map(|(l, _)| l.starts_with('(')).unwrap_or(false) {
        1
    } else {
        0
    };

    let directive_lines = group[..directive_count].to_vec();
    let data_lines = &group[directive_count..];

    let mut resolved: Vec<(String, usize)> = Vec::with_capacity(data_lines.len());

    for (i, (line, offset)) in data_lines.iter().enumerate() {
        if line.as_str() == "\"" {
            // Guard: if i >= parts.len() the interleaved parser will emit a better error.
            if i >= parts.len() {
                resolved.push((line.clone(), *offset));
                continue;
            }
            let col_type = column_type(&parts[i]);
            let source = (0..resolved.len())
                .rev()
                .find(|&j| j < parts.len() && column_type(&parts[j]) == col_type)
                .map(|j| resolved[j].0.clone());

            match source {
                Some(src_content) => {
                    // Keep the ditto's own byte offset so error spans point here.
                    resolved.push((src_content, *offset));
                }
                None => {
                    return Err(JianPuError::new(
                        Span::new(*offset, *offset + 1),
                        format!(
                            "ditto '\"' has no preceding {} line in this measure group",
                            col_type_name(&parts[i])
                        ),
                    ));
                }
            }
        } else {
            resolved.push((line.clone(), *offset));
        }
    }

    let mut result = directive_lines;
    result.extend(resolved);
    Ok(result)
}

#[derive(PartialEq)]
enum ColType {
    Notes,
    Lyrics,
    Chord,
}

fn column_type(col: &PartColumn) -> ColType {
    match col {
        PartColumn::Notes { .. } => ColType::Notes,
        PartColumn::Lyrics { .. } => ColType::Lyrics,
        PartColumn::Chord { .. } => ColType::Chord,
    }
}

fn col_type_name(col: &PartColumn) -> &'static str {
    match col {
        PartColumn::Notes { .. } => "notes",
        PartColumn::Lyrics { .. } => "lyrics",
        PartColumn::Chord { .. } => "chord",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notes(name: &str) -> PartColumn {
        PartColumn::Notes { name: name.to_string() }
    }
    fn lyrics(name: &str) -> PartColumn {
        PartColumn::Lyrics { name: name.to_string() }
    }
    fn chord(name: &str) -> PartColumn {
        PartColumn::Chord { name: name.to_string() }
    }

    fn group(lines: &[&str]) -> Vec<(String, usize)> {
        lines.iter().enumerate().map(|(i, l)| (l.to_string(), i * 10)).collect()
    }

    #[test]
    fn notes_ditto_copies_preceding_notes_line() {
        let groups = vec![group(&["1 2 3 4", "\""])];
        let parts = vec![notes("A"), notes("B")];
        let result = desugar_groups(groups, &parts).unwrap();
        assert_eq!(result[0][1].0, "1 2 3 4");
    }
}
```

- [ ] **Step 3: Run the test to verify it passes**

```bash
cargo test desugar -- --nocapture
```

Expected output: `test desugar::tests::notes_ditto_copies_preceding_notes_line ... ok`

- [ ] **Step 4: Commit**

```bash
git add src/desugar.rs src/main.rs
git commit -m "feat: add desugar module with notes ditto support"
```

---

### Task 2: Add lyrics and chord ditto tests

**Files:**
- Modify: `src/desugar.rs` (add tests only — implementation already covers these)

- [ ] **Step 1: Add lyrics and chord ditto tests**

In `src/desugar.rs`, inside the `tests` module, add:

```rust
    #[test]
    fn lyrics_ditto_copies_preceding_lyrics_line() {
        let groups = vec![group(&["1 2 3 4", "hello world", "5 6 7 1", "\""])];
        let parts = vec![notes("A"), lyrics("A"), notes("B"), lyrics("B")];
        let result = desugar_groups(groups, &parts).unwrap();
        assert_eq!(result[0][3].0, "hello world");
    }

    #[test]
    fn chord_ditto_copies_preceding_chord_line() {
        let groups = vec![group(&["1 - - -", "1 2 3 4", "\"", "5 6 7 1"])];
        let parts = vec![chord("main"), notes("A"), chord("main2"), notes("B")];
        let result = desugar_groups(groups, &parts).unwrap();
        assert_eq!(result[0][2].0, "1 - - -");
    }

    #[test]
    fn notes_ditto_does_not_copy_lyrics_line() {
        // A `"` on a notes line must NOT match a preceding lyrics line.
        // Only the notes line before it counts.
        let groups = vec![group(&["1 2 3 4", "hello world", "\""])];
        let parts = vec![notes("A"), lyrics("A"), notes("B")];
        let result = desugar_groups(groups, &parts).unwrap();
        assert_eq!(result[0][2].0, "1 2 3 4");
    }
```

- [ ] **Step 2: Run the new tests**

```bash
cargo test desugar -- --nocapture
```

Expected: all three new tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/desugar.rs
git commit -m "test: verify lyrics and chord ditto resolution"
```

---

### Task 3: Ditto chains and error cases

**Files:**
- Modify: `src/desugar.rs` (add tests only)

- [ ] **Step 1: Add chained ditto and error-case tests**

In `src/desugar.rs`, inside the `tests` module, add:

```rust
    #[test]
    fn chained_ditto_resolves_transitively() {
        // Third line dittos the second, which dittos the first.
        let groups = vec![group(&["1 2 3 4", "\"", "\""])];
        let parts = vec![notes("A"), notes("B"), notes("C")];
        let result = desugar_groups(groups, &parts).unwrap();
        assert_eq!(result[0][1].0, "1 2 3 4");
        assert_eq!(result[0][2].0, "1 2 3 4");
    }

    #[test]
    fn ditto_with_no_preceding_line_is_an_error() {
        let groups = vec![group(&["\""])];
        let parts = vec![notes("A")];
        let err = desugar_groups(groups, &parts).unwrap_err();
        assert!(
            err.message.contains("no preceding notes line"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn ditto_with_no_preceding_line_of_same_type_is_an_error() {
        // lyrics `"` with only a notes line before it — no preceding lyrics.
        let groups = vec![group(&["1 2 3 4", "\""])];
        let parts = vec![notes("A"), lyrics("A")];
        let err = desugar_groups(groups, &parts).unwrap_err();
        assert!(
            err.message.contains("no preceding lyrics line"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn directive_line_is_not_a_ditto_target() {
        // Group starts with a directive — ditto on the first data line
        // should still error (no preceding notes line in data lines).
        let groups = vec![group(&["(time=4/4)", "\""])];
        let parts = vec![notes("A")];
        let err = desugar_groups(groups, &parts).unwrap_err();
        assert!(err.message.contains("no preceding notes line"), "got: {}", err.message);
    }

    #[test]
    fn non_ditto_lines_are_passed_through_unchanged() {
        let groups = vec![group(&["1 2 3 4", "hello"])];
        let parts = vec![notes("A"), lyrics("A")];
        let result = desugar_groups(groups, &parts).unwrap();
        assert_eq!(result[0][0].0, "1 2 3 4");
        assert_eq!(result[0][1].0, "hello");
    }

    #[test]
    fn multiple_groups_are_desugared_independently() {
        // Ditto in group 2 must NOT copy from group 1.
        let groups = vec![
            group(&["1 2 3 4"]),
            group(&["\""])
        ];
        let parts = vec![notes("A")];
        let err = desugar_groups(groups, &parts).unwrap_err();
        assert!(err.message.contains("no preceding notes line"), "got: {}", err.message);
    }
```

- [ ] **Step 2: Run the tests**

```bash
cargo test desugar -- --nocapture
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/desugar.rs
git commit -m "test: add ditto chain and error-case tests"
```

---

### Task 4: Wire desugar into the interleaved parser

**Files:**
- Modify: `src/parser/score/interleaved_parser.rs`

- [ ] **Step 1: Write a failing integration test in `interleaved_parser.rs`**

In `src/parser/score/interleaved_parser.rs`, inside the `tests` module, add:

```rust
    #[test]
    fn notes_ditto_resolves_in_full_parse() {
        // Second part's notes line is `"` — should resolve to first part's notes.
        let content = concat!(
            "(time=4/4 key=C4 bpm=120)\n",
            "1 2 3 4\n",
            "\"\n",
        );
        let parts = vec![notes_col("S"), notes_col("A")];
        let (result, _) = parse(content, 0, &parts).unwrap();
        assert_eq!(result.len(), 2);
        // Both parts should have 7 events (3 directives + 4 notes).
        assert_eq!(result[0].score.events.len(), 7);
        assert_eq!(result[1].score.events.len(), 4, "Alto must have the same notes as Soprano");
    }

    #[test]
    fn lyrics_ditto_resolves_in_full_parse() {
        let content = concat!(
            "(time=4/4 key=C4 bpm=120)\n",
            "1 2 3 4\n",
            "do re mi fa\n",
            "\"\n",
            "\"\n",
        );
        let parts = vec![notes_col("S"), lyrics_col("S"), notes_col("A"), lyrics_col("A")];
        let (result, _) = parse(content, 0, &parts).unwrap();
        let s_lyrics = result[0].lyrics.as_ref().unwrap();
        let a_lyrics = result[1].lyrics.as_ref().unwrap();
        assert_eq!(s_lyrics.syllables.len(), 4);
        assert_eq!(a_lyrics.syllables.len(), 4);
        assert_eq!(s_lyrics.syllables[0].text, a_lyrics.syllables[0].text);
    }
```

- [ ] **Step 2: Run the new tests to confirm they fail**

```bash
cargo test interleaved_parser::tests::notes_ditto_resolves -- --nocapture
cargo test interleaved_parser::tests::lyrics_ditto_resolves -- --nocapture
```

Expected: both fail — `"` is not recognised and produces a parse/beat-count error.

- [ ] **Step 3: Call `desugar_groups` in `interleaved_parser::parse`**

In `src/parser/score/interleaved_parser.rs`, find the line:

```rust
    let groups = collect_groups(content);
```

Replace it with:

```rust
    let groups = collect_groups(content);
    let groups = crate::desugar::desugar_groups(groups, parts)?;
```

- [ ] **Step 4: Run the tests to confirm they pass**

```bash
cargo test -- --nocapture 2>&1 | tail -20
```

Expected: all 221+ tests pass with the two new tests now green.

- [ ] **Step 5: Commit**

```bash
git add src/parser/score/interleaved_parser.rs
git commit -m "feat: wire desugar_groups into interleaved parser"
```

---

### Task 5: Update `彌勒淨土鄉.jianpu` to use ditto

**Files:**
- Modify: `彌勒淨土鄉.jianpu`

- [ ] **Step 1: Replace duplicate lines with `"`**

For each measure group, the pattern is:
- Line 1: chord
- Line 2: A1&T notes  ← keep
- Line 3: A1&T lyrics ← keep
- Line 4: A2 notes    ← replace with `"` when identical to A1&T notes
- Line 5: A2 lyrics   ← replace with `"` (always same as A1&T)
- Line 6: S1 notes    ← replace with `"` when identical to A1&T notes
- Line 7: S1 lyrics   ← replace with `"` when identical to A1&T lyrics
- Line 8: S2 notes    ← replace with `"` (always same as S1)
- Line 9: S2 lyrics   ← replace with `"` (always same as S1)

For Verse 1 measure 1 (lines 13-22 in the file), the result is:

```
(bpm=92 key=C4 time=4/4 label="Verse 1")
1 - - -
_5 _5 _5 =5 =5 _5 _3 _2 _3~
白陽旗旛在大道盛宏
"
"
"
"
"
"
```

Apply this pattern to every measure group in the file where lines are identical. For Chorus measures where S1/S2 differ from A1&T, keep the actual S1/S2 content but still ditto S2 from S1.

- [ ] **Step 2: Verify the file generates without errors**

```bash
cargo run -- generate pdf 彌勒淨土鄉.jianpu
```

Expected: `written to "彌勒淨土鄉.pdf"` with no errors.

- [ ] **Step 3: Verify per-track output still works**

```bash
cargo run -- generate pdf 彌勒淨土鄉.jianpu --split-tracks
```

Expected: individual PDFs for A1&T, A2, S1, S2 all generated without errors.

- [ ] **Step 4: Commit**

```bash
git add 彌勒淨土鄉.jianpu
git commit -m "refactor: deduplicate score lines using ditto marker"
```
