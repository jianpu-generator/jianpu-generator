# Multi-Part Score Support — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add multi-part (choral) score support where named `[score:X]`/`[lyrics:X]` sections are stacked vertically with aligned bar lines, each part having its own notes and optional lyrics rows.

**Architecture:** Part-aware end-to-end pipeline — section splitter extracts names, parser pairs sections into `ParsedPart`s, grouper produces `GroupedPart`s per part, combiner zips into `Vec<MultiPartMeasure>`, layout stacks parts vertically with reserved label columns, renderer is unchanged except for a new `PartLabel` grid content variant.

**Tech Stack:** Rust, `cargo test` to run tests, no new dependencies.

---

## File Map

| File | Change |
|---|---|
| `src/parser/section_splitter.rs` | `SectionKind::Score { name }` / `Lyrics { name }` record variants |
| `src/parser/metadata_parser.rs` | Rename `cell_size` → `cell size`, add `label width` |
| `src/parser/mod.rs` | Multi-part pairing, orphan/duplicate detection |
| `src/ast/parsed.rs` | Add `ParsedScore`, `ParsedLyrics`, `ParsedPart`; update `ParsedDocument`, `ParsedMetadata` |
| `src/ast/grouped.rs` | Add `Notes`, `Lyrics`, `PartSlice`, `MultiPartMeasure`, `GroupedPart`, `GroupedMeasure`; update `Score`, `Metadata`; remove old `Measure` |
| `src/grouper.rs` | Refactor to `group_part()` helper producing `GroupedPart` |
| `src/combiner.rs` | **New** — zip `Vec<GroupedPart>` → `Vec<MultiPartMeasure>`, distribute lyrics per measure |
| `src/main.rs` | Add `mod combiner`, wire into pipeline |
| `src/layout/types.rs` | Add `PartLabel { text: String }` to `GridContent`; `BarLine { height_in_rows: u32 }` |
| `src/layout/mod.rs` | Multi-part layout: per-measure max-width, stacked rows, label columns, directive duplication |
| `src/renderer.rs` | Handle `PartLabel` and `BarLine { height_in_rows }` |
| `demo.jianpu` | Rename `cell_size` → `cell size`, add multi-part example |

---

## Task 1: Named section headers in section splitter

**Files:**
- Modify: `src/parser/section_splitter.rs`

- [ ] **Step 1: Write failing tests**

Add to the `#[cfg(test)]` block in `src/parser/section_splitter.rs`:

```rust
#[test]
fn parses_named_score_section() {
    let input = "[score:Soprano]\n1 2 3\n";
    let sections = split_sections(input).unwrap();
    assert_eq!(sections[0].kind, SectionKind::Score { name: Some("Soprano".to_string()) });
}

#[test]
fn parses_unnamed_score_section_remains_compatible() {
    let input = "[score]\n1 2 3\n";
    let sections = split_sections(input).unwrap();
    assert_eq!(sections[0].kind, SectionKind::Score { name: None });
}

#[test]
fn parses_named_lyrics_section() {
    let input = "[lyrics:Alto]\ndo re mi\n";
    let sections = split_sections(input).unwrap();
    assert_eq!(sections[0].kind, SectionKind::Lyrics { name: Some("Alto".to_string()) });
}
```

- [ ] **Step 2: Run tests — expect compile error** (old `SectionKind::Score` tuple syntax)

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test section_splitter 2>&1 | head -30
```

- [ ] **Step 3: Update `SectionKind` to record variants**

Replace the `SectionKind` enum and the section header parsing in `src/parser/section_splitter.rs`:

```rust
#[derive(Debug, PartialEq)]
pub enum SectionKind {
    Metadata,
    Score { name: Option<String> },
    Lyrics { name: Option<String> },
}
```

Replace the match arm that parses the `kind_str`:

```rust
current_kind = Some(match kind_str.split_once(':') {
    Some(("metadata", _)) | None if kind_str == "metadata" => SectionKind::Metadata,
    Some(("score", name)) => SectionKind::Score { name: Some(name.to_string()) },
    None if kind_str == "score" => SectionKind::Score { name: None },
    Some(("lyrics", name)) => SectionKind::Lyrics { name: Some(name.to_string()) },
    None if kind_str == "lyrics" => SectionKind::Lyrics { name: None },
    _ => {
        return Err(JianPuError::new(
            Span::new(byte_offset, byte_offset + line.len()),
            format!("unknown section: [{}]", kind_str),
        ))
    }
});
```

Update existing tests in the same file that pattern-match on the old variants:

```rust
// splits_three_sections: update assertions
assert_eq!(sections[1].kind, SectionKind::Score { name: None });
assert_eq!(sections[2].kind, SectionKind::Lyrics { name: None });

// handles_consecutive_headers: update assertion
assert_eq!(sections[1].kind, SectionKind::Score { name: None });
```

- [ ] **Step 4: Run tests — expect pass**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test section_splitter 2>&1
```

Expected: all `section_splitter` tests pass.

- [ ] **Step 5: Commit**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && git add src/parser/section_splitter.rs && git commit -m "feat: add named section headers to section splitter"
```

---

## Task 2: Space-case metadata keys and label width

**Files:**
- Modify: `src/parser/metadata_parser.rs`
- Modify: `src/ast/parsed.rs`

- [ ] **Step 1: Write failing tests**

Add to `src/parser/metadata_parser.rs` tests:

```rust
#[test]
fn parses_cell_size_with_space_case() {
    let content = "title = \"t\"\nauthor = \"a\"\ncell size = 20\n";
    let meta = parse_metadata(content, 0).unwrap();
    assert_eq!(meta.cell_size, Some(20));
}

#[test]
fn rejects_cell_size_with_underscore() {
    let content = "title = \"t\"\nauthor = \"a\"\ncell_size = 20\n";
    assert!(parse_metadata(content, 0).is_err());
}

#[test]
fn parses_label_width() {
    let content = "title = \"t\"\nauthor = \"a\"\nlabel width = 60\n";
    let meta = parse_metadata(content, 0).unwrap();
    assert_eq!(meta.label_width, Some(60));
}

#[test]
fn label_width_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let meta = parse_metadata(content, 0).unwrap();
    assert_eq!(meta.label_width, None);
}
```

- [ ] **Step 2: Run tests — expect compile error** (`label_width` field not defined)

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test metadata 2>&1 | head -30
```

- [ ] **Step 3: Add `label_width` to `ParsedMetadata` in `src/ast/parsed.rs`**

```rust
pub struct ParsedMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub cell_size: Option<u32>,
    pub label_width: Option<u32>,
}
```

- [ ] **Step 4: Update `parse_metadata` in `src/parser/metadata_parser.rs`**

Add `let mut label_width: Option<u32> = None;` with the other `let mut` declarations.

Replace the `"cell_size"` match arm with `"cell size"` and add `"label width"`:

```rust
"cell size" => {
    let parsed = value.parse::<u32>().map_err(|_| {
        JianPuError::new(
            line_span.clone(),
            format!("cell size must be a positive integer, got: {}", value),
        )
    })?;
    if parsed == 0 {
        return Err(JianPuError::new(
            line_span.clone(),
            "cell size must be greater than zero".to_string(),
        ));
    }
    cell_size = Some(parsed);
}
"label width" => {
    let parsed = value.parse::<u32>().map_err(|_| {
        JianPuError::new(
            line_span.clone(),
            format!("label width must be a positive integer, got: {}", value),
        )
    })?;
    if parsed == 0 {
        return Err(JianPuError::new(
            line_span.clone(),
            "label width must be greater than zero".to_string(),
        ));
    }
    label_width = Some(parsed);
}
```

Add `label_width` to the returned `ParsedMetadata`:

```rust
Ok(ParsedMetadata {
    title: title.ok_or_else(|| JianPuError::new(zero_span.clone(), "missing required field: title"))?,
    subtitle,
    author: author.ok_or_else(|| JianPuError::new(zero_span, "missing required field: author"))?,
    cell_size,
    label_width,
})
```

Update existing metadata parser tests that use `cell_size`:

```rust
// parses_optional_cell_size
let content = "title = \"t\"\nauthor = \"a\"\ncell size = 16\n";

// rejects_invalid_cell_size
let content = "title = \"t\"\nauthor = \"a\"\ncell size = abc\n";
```

- [ ] **Step 5: Fix grouper compile error**

In `src/grouper.rs`, `doc.metadata.cell_size` is still `Option<u32>` — no change needed. But `Metadata` in `grouped.rs` will eventually get `label_width`. For now the grouper still compiles since we haven't changed `grouped.rs` yet.

- [ ] **Step 6: Run tests — expect pass**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && git add src/parser/metadata_parser.rs src/ast/parsed.rs && git commit -m "feat: rename cell_size to 'cell size', add 'label width' metadata field"
```

---

## Task 3: Multi-part parsed AST + parser pairing

This task changes `ParsedDocument` and updates the parser. The grouper will break and is fixed in Task 4.

**Files:**
- Modify: `src/ast/parsed.rs`
- Modify: `src/parser/mod.rs`

- [ ] **Step 1: Write failing parser tests**

Add to `src/parser/mod.rs` tests:

```rust
#[test]
fn parses_two_named_parts() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
        "[score:Soprano]\n4/4 1 2 3 4\n",
        "[lyrics:Soprano]\na b c d\n",
        "[score:Alto]\n5 6 7 1\n",
    );
    let doc = parse(input, "test.jianpu").unwrap();
    assert_eq!(doc.parts.len(), 2);
    assert_eq!(doc.parts[0].name, Some("Soprano".to_string()));
    assert_eq!(doc.parts[1].name, Some("Alto".to_string()));
    assert!(doc.parts[0].lyrics.is_some());
    assert!(doc.parts[1].lyrics.is_none());
}

#[test]
fn single_unnamed_part_remains_compatible() {
    let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4\n\n[lyrics]\na b c d\n";
    let doc = parse(input, "test.jianpu").unwrap();
    assert_eq!(doc.parts.len(), 1);
    assert_eq!(doc.parts[0].name, None);
    assert!(doc.parts[0].lyrics.is_some());
}

#[test]
fn rejects_orphan_lyrics_section() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
        "[score]\n4/4 1 2 3 4\n",
        "[lyrics:Alto]\na b c d\n",
    );
    let err = parse(input, "test.jianpu").unwrap_err();
    assert!(err.message.contains("orphan"), "expected orphan error, got: {}", err.message);
}

#[test]
fn rejects_duplicate_score_section_by_name() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
        "[score:S]\n4/4 1 2 3 4\n",
        "[score:S]\n4/4 5 6 7 1\n",
    );
    assert!(parse(input, "test.jianpu").is_err());
}

#[test]
fn rejects_duplicate_lyrics_section_by_name() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
        "[score:S]\n4/4 1 2 3 4\n",
        "[lyrics:S]\na b c d\n",
        "[lyrics:S]\ne f g h\n",
    );
    assert!(parse(input, "test.jianpu").is_err());
}

#[test]
fn rejects_directive_in_non_first_part() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
        "[score:Soprano]\n4/4 1 2 3 4\n",
        "[score:Alto]\nbpm=90 5 6 7 1\n",
    );
    assert!(parse(input, "test.jianpu").is_err());
}
```

- [ ] **Step 2: Run tests — expect compile errors**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test parser 2>&1 | head -30
```

- [ ] **Step 3: Update `src/ast/parsed.rs` with multi-part types**

Replace the file content (keep all existing types, add new ones):

```rust
use crate::error::Spanned;

pub struct ParsedScore {
    pub events: Vec<Spanned<ScoreEvent>>,
}

pub struct ParsedLyrics {
    pub syllables: Vec<Syllable>,
}

pub struct ParsedPart {
    pub name: Option<String>,
    pub score: ParsedScore,
    pub lyrics: Option<ParsedLyrics>,
}

pub struct ParsedDocument {
    pub filename: String,
    pub metadata: ParsedMetadata,
    pub parts: Vec<ParsedPart>,
}

pub struct ParsedMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub cell_size: Option<u32>,
    pub label_width: Option<u32>,
}

pub enum ScoreEvent {
    Note(ParsedNote),
    Rest(ParsedRest),
    BpmChange(u32),
    KeyChange(KeyChange),
    TimeSignatureChange { numerator: u8, denominator: u8 },
    Extension,
}

pub struct ParsedNote {
    pub pitch: JianPuPitch,
    pub octave: i8,
    pub duration: u32,
    pub tie: bool,
}

pub struct ParsedRest {
    pub duration: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JianPuPitch {
    One, Two, Three, Four, Five, Six, Seven,
}

#[derive(Clone)]
pub struct KeyChange {
    pub note: Note,
}

#[derive(Clone)]
pub struct Note {
    pub name: NoteName,
    pub octave: u8,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteName { A, B, C, D, E, F, G }

#[derive(Debug, Clone, PartialEq)]
pub enum Accidental { Flat, Sharp, Natural }

#[derive(Debug, Clone, PartialEq)]
pub struct Syllable {
    pub text: String,
    pub held: bool,
}
```

- [ ] **Step 4: Rewrite `src/parser/mod.rs`**

Replace the full file:

```rust
use crate::ast::parsed::{ParsedDocument, ParsedLyrics, ParsedPart, ParsedScore};
use crate::error::{JianPuError, Span};

pub mod lyrics;
pub mod metadata_parser;
pub mod score;
pub mod section_splitter;

pub fn parse(input: &str, filename: &str) -> Result<ParsedDocument, JianPuError> {
    use section_splitter::{split_sections, SectionKind};

    let sections = split_sections(input)?;

    let mut raw_metadata: Option<(String, usize)> = None;
    let mut raw_scores: Vec<(Option<String>, String, usize)> = Vec::new();
    let mut raw_lyrics: Vec<(Option<String>, String)> = Vec::new();

    let doc_span = Span::new(0, input.len());

    for section in sections {
        match section.kind {
            SectionKind::Metadata => {
                if raw_metadata.is_some() {
                    return Err(JianPuError::new(doc_span.clone(), "duplicate [metadata] section"));
                }
                raw_metadata = Some((section.content, section.content_offset));
            }
            SectionKind::Score { name } => {
                if raw_scores.iter().any(|(n, _, _)| n == &name) {
                    return Err(JianPuError::new(
                        doc_span.clone(),
                        format!("duplicate [score{}] section", name.as_deref().map(|n| format!(":{}", n)).unwrap_or_default()),
                    ));
                }
                raw_scores.push((name, section.content, section.content_offset));
            }
            SectionKind::Lyrics { name } => {
                if raw_lyrics.iter().any(|(n, _)| n == &name) {
                    return Err(JianPuError::new(
                        doc_span.clone(),
                        format!("duplicate [lyrics{}] section", name.as_deref().map(|n| format!(":{}", n)).unwrap_or_default()),
                    ));
                }
                // Orphan check: lyrics name must match a score name
                if !raw_scores.iter().any(|(n, _, _)| n == &name) {
                    return Err(JianPuError::new(
                        doc_span.clone(),
                        format!(
                            "orphan [lyrics{}] section: no matching [score{}] found",
                            name.as_deref().map(|n| format!(":{}", n)).unwrap_or_default(),
                            name.as_deref().map(|n| format!(":{}", n)).unwrap_or_default(),
                        ),
                    ));
                }
                raw_lyrics.push((name, section.content));
            }
        }
    }

    let (meta_content, meta_offset) = raw_metadata
        .ok_or_else(|| JianPuError::new(doc_span.clone(), "missing [metadata] section"))?;

    if raw_scores.is_empty() {
        return Err(JianPuError::new(doc_span, "missing [score] section"));
    }

    let metadata = metadata_parser::parse_metadata(&meta_content, meta_offset)?;

    let mut parts = Vec::new();
    for (i, (name, score_content, score_offset)) in raw_scores.into_iter().enumerate() {
        let tokens = score::tokenizer::tokenize(&score_content, score_offset);
        let events = score::token_parser::parse_tokens(tokens)?;

        // Directives are only allowed in the first part
        if i > 0 {
            use crate::ast::parsed::ScoreEvent;
            for spanned in &events {
                match &spanned.value {
                    ScoreEvent::BpmChange(_)
                    | ScoreEvent::KeyChange(_)
                    | ScoreEvent::TimeSignatureChange { .. } => {
                        return Err(JianPuError::new(
                            spanned.span.clone(),
                            "directives (bpm, key, time signature) are only allowed in the first part's score section".to_string(),
                        ));
                    }
                    _ => {}
                }
            }
        }

        let lyrics = raw_lyrics
            .iter()
            .find(|(n, _)| n == &name)
            .map(|(_, content)| ParsedLyrics {
                syllables: lyrics::tokenizer::tokenize_lyrics(content),
            });

        parts.push(ParsedPart {
            name,
            score: ParsedScore { events },
            lyrics,
        });
    }

    Ok(ParsedDocument {
        filename: filename.to_string(),
        metadata,
        parts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = concat!(
        "[metadata]\ntitle = \"hello world\"\nauthor = \"foo\"\n\n",
        "[score]\nbpm=120 1=C4 4/4 1 2 _3 _4\n\n",
        "[lyrics]\n你好wo rld\n"
    );

    #[test]
    fn parses_full_document() {
        let doc = parse(SAMPLE, "test.jianpu").unwrap();
        assert_eq!(doc.metadata.title, "hello world");
        assert_eq!(doc.metadata.author, "foo");
        assert_eq!(doc.parts.len(), 1);
        // 4/4 time sig + bpm + key + 4 notes = 7 events
        assert_eq!(doc.parts[0].score.events.len(), 7);
        // 4 syllables: 你 好 wo rld
        assert_eq!(doc.parts[0].lyrics.as_ref().unwrap().syllables.len(), 4);
    }

    #[test]
    fn rejects_unknown_section() {
        let input = "[unknown]\nfoo\n";
        assert!(parse(input, "test.jianpu").is_err());
    }

    #[test]
    fn rejects_duplicate_score_section() {
        let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n[score]\n1 2 3 4\n[score]\n5 6 7 1\n[lyrics]\na\n";
        assert!(parse(input, "test.jianpu").is_err());
    }

    #[test]
    fn rejects_missing_metadata_section() {
        let input = "[score]\n4/4 1 2 3 4\n[lyrics]\na b c d\n";
        assert!(parse(input, "test.jianpu").is_err());
    }

    #[test]
    fn parses_two_named_parts() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
            "[score:Soprano]\n4/4 1 2 3 4\n",
            "[lyrics:Soprano]\na b c d\n",
            "[score:Alto]\n5 6 7 1\n",
        );
        let doc = parse(input, "test.jianpu").unwrap();
        assert_eq!(doc.parts.len(), 2);
        assert_eq!(doc.parts[0].name, Some("Soprano".to_string()));
        assert_eq!(doc.parts[1].name, Some("Alto".to_string()));
        assert!(doc.parts[0].lyrics.is_some());
        assert!(doc.parts[1].lyrics.is_none());
    }

    #[test]
    fn single_unnamed_part_remains_compatible() {
        let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4\n\n[lyrics]\na b c d\n";
        let doc = parse(input, "test.jianpu").unwrap();
        assert_eq!(doc.parts.len(), 1);
        assert_eq!(doc.parts[0].name, None);
        assert!(doc.parts[0].lyrics.is_some());
    }

    #[test]
    fn rejects_orphan_lyrics_section() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
            "[score]\n4/4 1 2 3 4\n",
            "[lyrics:Alto]\na b c d\n",
        );
        let err = parse(input, "test.jianpu").unwrap_err();
        assert!(err.message.contains("orphan"), "expected orphan error, got: {}", err.message);
    }

    #[test]
    fn rejects_duplicate_score_section_by_name() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
            "[score:S]\n4/4 1 2 3 4\n",
            "[score:S]\n4/4 5 6 7 1\n",
        );
        assert!(parse(input, "test.jianpu").is_err());
    }

    #[test]
    fn rejects_duplicate_lyrics_section_by_name() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
            "[score:S]\n4/4 1 2 3 4\n",
            "[lyrics:S]\na b c d\n",
            "[lyrics:S]\ne f g h\n",
        );
        assert!(parse(input, "test.jianpu").is_err());
    }

    #[test]
    fn rejects_directive_in_non_first_part() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
            "[score:Soprano]\n4/4 1 2 3 4\n",
            "[score:Alto]\nbpm=90 5 6 7 1\n",
        );
        assert!(parse(input, "test.jianpu").is_err());
    }
}
```

- [ ] **Step 5: Run parser tests — expect pass, but grouper tests will fail (expected)**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test parser 2>&1 | tail -20
```

Expected: all `parser::tests` pass. `grouper` and `renderer` tests will fail to compile — that's expected and fixed in Task 4.

- [ ] **Step 6: Commit parser changes (build may not fully compile yet)**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && git add src/ast/parsed.rs src/parser/mod.rs && git commit -m "feat: multi-part parsed AST types and parser pairing logic"
```

---

## Task 4: Multi-part grouped AST, grouper, combiner, and main pipeline

This is the largest task. It replaces `Measure`-based `Score` with `MultiPartMeasure`-based `Score` and wires everything together. All tests should pass after this task.

**Files:**
- Modify: `src/ast/grouped.rs`
- Modify: `src/grouper.rs`
- Create: `src/combiner.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Replace `src/ast/grouped.rs`**

```rust
use crate::ast::parsed::{JianPuPitch, KeyChange, Syllable};

// ── Public final types ────────────────────────────────────────────────────────

pub struct Metadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    /// Grid cell size in points. Default: 24.
    pub cell_size: u32,
    /// Left margin reserved for part labels in points. Default: 40.
    pub label_width: u32,
}

pub struct Notes {
    pub events: Vec<NoteEvent>,
}

pub struct Lyrics {
    pub syllables: Vec<Syllable>,
}

pub struct PartSlice {
    pub name: Option<String>,
    pub notes: Notes,
    pub lyrics: Option<Lyrics>,
}

pub struct MultiPartMeasure {
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<u32>,
    pub key: Option<KeyChange>,
    pub parts: Vec<PartSlice>,
}

pub struct Score {
    pub metadata: Metadata,
    pub measures: Vec<MultiPartMeasure>,
}

// ── Intermediate grouper types (not part of the public API) ─────────────────

pub(crate) struct GroupedMeasure {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) notes: Notes,
}

pub(crate) struct GroupedPart {
    pub(crate) name: Option<String>,
    pub(crate) measures: Vec<GroupedMeasure>,
    /// Flat lyrics list. `None` means no [lyrics] section was provided.
    pub(crate) lyrics: Option<Vec<Syllable>>,
}

// ── Shared note types ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

pub enum NoteEvent {
    Note(GroupedNote),
    Rest(GroupedRest),
}

pub struct GroupedNote {
    pub pitch: JianPuPitch,
    pub octave: i8,
    pub duration: u32,
    pub tie: bool,
}

pub struct GroupedRest {
    pub duration: u32,
}
```

- [ ] **Step 2: Rewrite `src/grouper.rs`**

```rust
use crate::ast::grouped::*;
use crate::ast::parsed::{Accidental, NoteName, ParsedDocument, ParsedPart};
use crate::combiner;
use crate::error::JianPuError;

pub fn group(doc: ParsedDocument) -> Result<Score, JianPuError> {
    let mut grouped_parts = Vec::new();
    for part in doc.parts {
        grouped_parts.push(group_part(part)?);
    }

    let measures = combiner::combine(grouped_parts)?;

    Ok(Score {
        metadata: Metadata {
            title: doc.metadata.title,
            subtitle: doc.metadata.subtitle,
            author: doc.metadata.author,
            cell_size: doc.metadata.cell_size.unwrap_or(24),
            label_width: doc.metadata.label_width.unwrap_or(40),
        },
        measures,
    })
}

fn group_part(part: ParsedPart) -> Result<GroupedPart, JianPuError> {
    use crate::ast::parsed::{KeyChange, Note, ScoreEvent};

    let default_key = KeyChange {
        note: Note { name: NoteName::C, octave: 4, accidental: Accidental::Natural },
    };

    let mut current_bpm: u32 = 120;
    let mut current_key = default_key;
    let mut current_time_sig = TimeSignature { numerator: 4, denominator: 4 };

    let measure_capacity = |ts: &TimeSignature| -> u32 {
        (ts.numerator as u32) * 16 / (ts.denominator as u32)
    };

    // Track whether each directive was explicitly set since the last measure boundary.
    // All start as true so the first measure always gets Some(_) for all directives.
    let mut bpm_changed = true;
    let mut key_changed = true;
    let mut time_sig_changed = true;

    let mut measures: Vec<GroupedMeasure> = Vec::new();
    let mut current_notes: Vec<NoteEvent> = Vec::new();
    let mut current_beat: u32 = 0;
    let mut capacity = measure_capacity(&current_time_sig);

    let flush_measure = |measures: &mut Vec<GroupedMeasure>,
                         current_notes: &mut Vec<NoteEvent>,
                         current_beat: &mut u32,
                         current_bpm: u32,
                         bpm_changed: &mut bool,
                         current_key: &KeyChange,
                         key_changed: &mut bool,
                         current_time_sig: &TimeSignature,
                         time_sig_changed: &mut bool| {
        if !current_notes.is_empty() {
            measures.push(GroupedMeasure {
                time_signature: if *time_sig_changed {
                    Some(TimeSignature {
                        numerator: current_time_sig.numerator,
                        denominator: current_time_sig.denominator,
                    })
                } else {
                    None
                },
                bpm: if *bpm_changed { Some(current_bpm) } else { None },
                key: if *key_changed { Some(current_key.clone()) } else { None },
                notes: Notes { events: std::mem::take(current_notes) },
            });
            *current_beat = 0;
            *bpm_changed = false;
            *key_changed = false;
            *time_sig_changed = false;
        }
    };

    for spanned in part.score.events {
        match spanned.value {
            ScoreEvent::BpmChange(bpm) => {
                flush_measure(
                    &mut measures, &mut current_notes, &mut current_beat,
                    current_bpm, &mut bpm_changed,
                    &current_key, &mut key_changed,
                    &current_time_sig, &mut time_sig_changed,
                );
                current_bpm = bpm;
                bpm_changed = true;
            }
            ScoreEvent::KeyChange(kc) => {
                flush_measure(
                    &mut measures, &mut current_notes, &mut current_beat,
                    current_bpm, &mut bpm_changed,
                    &current_key, &mut key_changed,
                    &current_time_sig, &mut time_sig_changed,
                );
                current_key = kc;
                key_changed = true;
            }
            ScoreEvent::TimeSignatureChange { numerator, denominator } => {
                flush_measure(
                    &mut measures, &mut current_notes, &mut current_beat,
                    current_bpm, &mut bpm_changed,
                    &current_key, &mut key_changed,
                    &current_time_sig, &mut time_sig_changed,
                );
                current_time_sig = TimeSignature { numerator, denominator };
                capacity = measure_capacity(&current_time_sig);
                time_sig_changed = true;
            }
            ScoreEvent::Extension => {
                match current_notes.last_mut() {
                    Some(NoteEvent::Note(n)) => {
                        n.duration += 4;
                        current_beat += 4;
                    }
                    Some(NoteEvent::Rest(r)) => {
                        r.duration += 4;
                        current_beat += 4;
                    }
                    None => {
                        return Err(JianPuError::new(
                            spanned.span,
                            "extension `-` without a preceding note or rest; if it follows a measure boundary, cross-measure extension is not supported".to_string(),
                        ));
                    }
                }
                if current_beat >= capacity {
                    flush_measure(
                        &mut measures, &mut current_notes, &mut current_beat,
                        current_bpm, &mut bpm_changed,
                        &current_key, &mut key_changed,
                        &current_time_sig, &mut time_sig_changed,
                    );
                }
            }
            ScoreEvent::Note(pn) => {
                if current_beat >= capacity {
                    flush_measure(
                        &mut measures, &mut current_notes, &mut current_beat,
                        current_bpm, &mut bpm_changed,
                        &current_key, &mut key_changed,
                        &current_time_sig, &mut time_sig_changed,
                    );
                }
                let note_duration = pn.duration;
                current_notes.push(NoteEvent::Note(GroupedNote {
                    pitch: pn.pitch,
                    octave: pn.octave,
                    duration: pn.duration,
                    tie: pn.tie,
                }));
                current_beat += note_duration;
                if current_beat > capacity {
                    return Err(JianPuError::new(
                        spanned.span,
                        format!(
                            "note duration {} overflows the current measure (capacity {} quarter-beats, {} used)",
                            note_duration, capacity, current_beat
                        ),
                    ));
                }
                if current_beat == capacity {
                    flush_measure(
                        &mut measures, &mut current_notes, &mut current_beat,
                        current_bpm, &mut bpm_changed,
                        &current_key, &mut key_changed,
                        &current_time_sig, &mut time_sig_changed,
                    );
                }
            }
            ScoreEvent::Rest(pr) => {
                if current_beat >= capacity {
                    flush_measure(
                        &mut measures, &mut current_notes, &mut current_beat,
                        current_bpm, &mut bpm_changed,
                        &current_key, &mut key_changed,
                        &current_time_sig, &mut time_sig_changed,
                    );
                }
                let rest_duration = pr.duration;
                current_notes.push(NoteEvent::Rest(GroupedRest { duration: pr.duration }));
                current_beat += rest_duration;
                if current_beat > capacity {
                    return Err(JianPuError::new(
                        spanned.span,
                        format!(
                            "rest duration {} overflows the current measure (capacity {} quarter-beats, {} used)",
                            rest_duration, capacity, current_beat
                        ),
                    ));
                }
                if current_beat == capacity {
                    flush_measure(
                        &mut measures, &mut current_notes, &mut current_beat,
                        current_bpm, &mut bpm_changed,
                        &current_key, &mut key_changed,
                        &current_time_sig, &mut time_sig_changed,
                    );
                }
            }
        }
    }

    if !current_notes.is_empty() {
        flush_measure(
            &mut measures, &mut current_notes, &mut current_beat,
            current_bpm, &mut bpm_changed,
            &current_key, &mut key_changed,
            &current_time_sig, &mut time_sig_changed,
        );
    }

    Ok(GroupedPart {
        name: part.name,
        measures,
        lyrics: part.lyrics.map(|l| l.syllables),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::ast::parsed::NoteName;

    fn parse_and_group(input: &str) -> Score {
        let doc = parser::parse(input, "test.jianpu").unwrap();
        group(doc).unwrap()
    }

    fn parse_and_group_err(input: &str) -> JianPuError {
        let doc = parser::parse(input, "test.jianpu").unwrap();
        match group(doc) {
            Err(e) => e,
            Ok(_) => panic!("expected group() to return Err, but it returned Ok"),
        }
    }

    fn first_part_notes(score: &Score, measure_idx: usize) -> &Vec<NoteEvent> {
        &score.measures[measure_idx].parts[0].notes.events
    }

    #[test]
    fn groups_four_four_into_single_measure() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4\n\n[lyrics]\na b c d\n",
        );
        assert_eq!(score.measures.len(), 1);
        assert_eq!(first_part_notes(&score, 0).len(), 4);
    }

    #[test]
    fn splits_into_two_measures_at_bar_boundary() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4 5 6 7 1\n\n[lyrics]\na b c d e f g h\n",
        );
        assert_eq!(score.measures.len(), 2);
    }

    #[test]
    fn extension_adds_to_previous_note_duration() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 - 3 4\n\n[lyrics]\na - b c\n",
        );
        match &first_part_notes(&score, 0)[0] {
            NoteEvent::Note(n) => assert_eq!(n.duration, 8),
            _ => panic!("expected Note"),
        }
    }

    #[test]
    fn first_measure_has_bpm_some() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4\n\n[lyrics]\na b c d\n",
        );
        assert_eq!(score.measures[0].bpm, Some(120));
    }

    #[test]
    fn bpm_change_sets_some_on_next_measure() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4 bpm=90 5 6 7 1\n\n[lyrics]\na b c d e f g h\n",
        );
        assert_eq!(score.measures[0].bpm, Some(120));
        assert_eq!(score.measures[1].bpm, Some(90));
    }

    #[test]
    fn unchanged_bpm_is_none_on_second_measure() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4 5 6 7 1\n\n[lyrics]\na b c d e f g h\n",
        );
        assert_eq!(score.measures[0].bpm, Some(120));
        assert_eq!(score.measures[1].bpm, None);
    }

    #[test]
    fn key_change_propagates() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1=G4 1 2 3 4\n\n[lyrics]\na b c d\n",
        );
        assert_eq!(score.measures[0].key.as_ref().unwrap().note.name, NoteName::G);
    }

    #[test]
    fn cell_size_defaults_to_24() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4\n\n[lyrics]\na b c d\n",
        );
        assert_eq!(score.metadata.cell_size, 24);
    }

    #[test]
    fn half_beat_notes_accumulate_correctly() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 _1 _2 _3 _4 _5 _6 _7 _1\n\n[lyrics]\na b c d e f g h\n",
        );
        assert_eq!(score.measures.len(), 1);
    }

    #[test]
    fn overflow_note_errors() {
        let err = parse_and_group_err(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 _1 _1 _1 _1 _1 _1 _1 1\n\n[lyrics]\na b c d e f g h\n",
        );
        assert!(err.message.contains("overflows"), "expected overflow error, got: {}", err.message);
    }

    #[test]
    fn bpm_change_creates_new_measure() {
        let score = parse_and_group(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 bpm=90 3 4\n\n[lyrics]\na b c d\n",
        );
        assert_eq!(score.measures.len(), 2);
        assert_eq!(score.measures[0].bpm, Some(120));
        assert_eq!(first_part_notes(&score, 0).len(), 2);
        assert_eq!(score.measures[1].bpm, Some(90));
        assert_eq!(first_part_notes(&score, 1).len(), 2);
    }

    #[test]
    fn two_part_score_has_two_part_slices_per_measure() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
            "[score:Soprano]\n4/4 1 2 3 4\n",
            "[score:Alto]\n5 6 7 1\n",
        );
        let doc = parser::parse(input, "test.jianpu").unwrap();
        let score = group(doc).unwrap();
        assert_eq!(score.measures.len(), 1);
        assert_eq!(score.measures[0].parts.len(), 2);
        assert_eq!(score.measures[0].parts[0].name, Some("Soprano".to_string()));
        assert_eq!(score.measures[0].parts[1].name, Some("Alto".to_string()));
    }

    #[test]
    fn lyrics_distributed_per_measure() {
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
            "[score]\n4/4 1 2 3 4 5 6 7 1\n",
            "[lyrics]\na b c d e f g h\n",
        );
        let doc = parser::parse(input, "test.jianpu").unwrap();
        let score = group(doc).unwrap();
        assert_eq!(score.measures.len(), 2);
        let m0_lyrics = score.measures[0].parts[0].lyrics.as_ref().unwrap();
        let m1_lyrics = score.measures[1].parts[0].lyrics.as_ref().unwrap();
        assert_eq!(m0_lyrics.syllables.len(), 4);
        assert_eq!(m1_lyrics.syllables.len(), 4);
    }
}
```

- [ ] **Step 3: Create `src/combiner.rs`**

```rust
use crate::ast::grouped::*;
use crate::ast::parsed::{JianPuPitch, Syllable};
use crate::error::{JianPuError, Span};

pub fn combine(parts: Vec<GroupedPart>) -> Result<Vec<MultiPartMeasure>, JianPuError> {
    if parts.is_empty() {
        return Ok(Vec::new());
    }

    let expected_len = parts[0].measures.len();
    for part in &parts[1..] {
        if part.measures.len() != expected_len {
            return Err(JianPuError::new(
                Span::new(0, 0),
                format!(
                    "part {:?} has {} measures but the first part has {}; all parts must have the same number of measures",
                    part.name, part.measures.len(), expected_len
                ),
            ));
        }
    }

    let lyrics_per_part: Vec<Vec<Vec<Syllable>>> = parts
        .iter()
        .map(|p| {
            p.lyrics
                .as_deref()
                .map(|lyrics| distribute_lyrics(&p.measures, lyrics))
                .unwrap_or_else(|| vec![vec![]; p.measures.len()])
        })
        .collect();

    let num_measures = expected_len;
    let mut combined = Vec::with_capacity(num_measures);

    for measure_idx in 0..num_measures {
        let first = &parts[0].measures[measure_idx];

        let part_slices = parts
            .iter()
            .enumerate()
            .map(|(part_idx, part)| {
                let measure = &part.measures[measure_idx];
                let syllables = lyrics_per_part[part_idx][measure_idx].clone();
                let lyrics = if part.lyrics.is_some() {
                    Some(Lyrics { syllables })
                } else {
                    None
                };
                PartSlice {
                    name: part.name.clone(),
                    notes: Notes { events: measure.notes.events.iter().map(clone_note_event).collect() },
                    lyrics,
                }
            })
            .collect();

        combined.push(MultiPartMeasure {
            time_signature: first.time_signature.clone(),
            bpm: first.bpm,
            key: first.key.clone(),
            parts: part_slices,
        });
    }

    Ok(combined)
}

fn clone_note_event(event: &NoteEvent) -> NoteEvent {
    match event {
        NoteEvent::Note(n) => NoteEvent::Note(GroupedNote {
            pitch: n.pitch.clone(),
            octave: n.octave,
            duration: n.duration,
            tie: n.tie,
        }),
        NoteEvent::Rest(r) => NoteEvent::Rest(GroupedRest { duration: r.duration }),
    }
}

fn distribute_lyrics(measures: &[GroupedMeasure], lyrics: &[Syllable]) -> Vec<Vec<Syllable>> {
    let mut syllable_idx = 0;
    let mut prev_tie = false;
    let mut prev_pitch: Option<JianPuPitch> = None;

    let mut result = Vec::with_capacity(measures.len());
    for measure in measures {
        let mut measure_syllables = Vec::new();
        for event in &measure.notes.events {
            match event {
                NoteEvent::Note(note) => {
                    let is_continuation = prev_tie && prev_pitch.as_ref() == Some(&note.pitch);
                    if !is_continuation && syllable_idx < lyrics.len() {
                        measure_syllables.push(lyrics[syllable_idx].clone());
                        syllable_idx += 1;
                    }
                    prev_tie = note.tie;
                    prev_pitch = Some(note.pitch.clone());
                }
                NoteEvent::Rest(_) => {
                    prev_tie = false;
                }
            }
        }
        result.push(measure_syllables);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grouper, parser};

    fn make_two_part_score(soprano: &str, alto: &str) -> Vec<MultiPartMeasure> {
        let input = format!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n[score:Soprano]\n4/4 {}\n[score:Alto]\n{}\n",
            soprano, alto
        );
        let doc = parser::parse(&input, "test.jianpu").unwrap();
        grouper::group(doc).unwrap().measures
    }

    #[test]
    fn combines_two_parts_into_measures() {
        let measures = make_two_part_score("1 2 3 4", "5 6 7 1");
        assert_eq!(measures.len(), 1);
        assert_eq!(measures[0].parts.len(), 2);
    }

    #[test]
    fn directives_come_from_first_part() {
        let measures = make_two_part_score("1 2 3 4", "5 6 7 1");
        assert_eq!(measures[0].bpm, Some(120));
        assert!(measures[0].time_signature.is_some());
    }

    #[test]
    fn rejects_parts_with_different_measure_counts() {
        // Alto has 2 measures, Soprano has 1
        let input = concat!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
            "[score:Soprano]\n4/4 1 2 3 4\n",
            "[score:Alto]\n5 6 7 1 5 6 7 1\n",
        );
        let doc = parser::parse(input, "test.jianpu").unwrap();
        assert!(grouper::group(doc).is_err());
    }
}
```

- [ ] **Step 4: Add `mod combiner` to `src/main.rs`**

```rust
mod ast;
mod combiner;
mod error;
mod grouper;
mod layout;
mod parser;
mod pdf;
mod renderer;
mod utils;
```

- [ ] **Step 5: Run all tests — expect pass**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test 2>&1 | tail -30
```

Expected: all tests pass. The layout and renderer tests will fail because `score.measures` now has a different shape — fix them in Step 6.

- [ ] **Step 6: Fix layout tests that reference old `Measure` fields**

The layout (`src/layout/mod.rs`) still references `score.measures` and `score.lyrics`. Update the layout module to compile against the new types (full layout rewrite is in Task 6; for now just make it compile):

The key failing references are:
- `score.lyrics` → remove (lyrics now in `PartSlice`)
- `measure.notes` → `measure.parts[0].notes.events`
- `measure.time_signature` → `measure.time_signature.as_ref()` (now `Option`)
- `measure.bpm` → `measure.bpm` (now `Option<u32>`)
- `measure.key` → `measure.key.as_ref()` (now `Option`)
- `previous_time_signature` / `previous_bpm` comparisons → no longer needed (grouper now handles this)

Full replacement of `src/layout/mod.rs` is in Task 6 below.

- [ ] **Step 7: Commit**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && git add src/ast/grouped.rs src/grouper.rs src/combiner.rs src/main.rs && git commit -m "feat: multi-part grouped AST, grouper, and combiner"
```

---

## Task 5: Layout types — PartLabel and BarLine height

**Files:**
- Modify: `src/layout/types.rs`

- [ ] **Step 1: Write failing test**

Add to `src/layout/mod.rs` tests (or create a new test):

```rust
#[test]
fn part_label_content_exists() {
    // Just verify the type exists and can be constructed
    let _ = GridContent::PartLabel { text: "Soprano".to_string() };
    let _ = GridContent::BarLine { height_in_rows: 1 };
}
```

- [ ] **Step 2: Run — expect compile error**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test part_label_content_exists 2>&1 | head -20
```

- [ ] **Step 3: Update `src/layout/types.rs`**

Add `PartLabel` and update `BarLine`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum GridContent {
    NoteHead { pitch: JianPuPitch, octave: i8 },
    Rest,
    Lyric { text: String, is_cjk: bool },
    TieOrSlurCurve { from_column: u32, to_column: u32 },
    DurationUnderlines { levels: Vec<UnderlineSpan> },
    LowerOctaveDots { count: u32 },
    BarLine { height_in_rows: u32 },
    Extension,
    TimeSignatureLabel { numerator: u8, denominator: u8 },
    BpmLabel { bpm: u32 },
    PartLabel { text: String },
}
```

- [ ] **Step 4: Fix all places that construct `GridContent::BarLine`**

In `src/layout/mod.rs`, update every `GridContent::BarLine` construction to `GridContent::BarLine { height_in_rows: 1 }`.

In `src/renderer.rs`, update the `GridContent::BarLine` match arm (add `height_in_rows` field):

```rust
GridContent::BarLine { height_in_rows } => {
    let line_x = base_x;
    let line_y1 = base_y;
    let line_y2 = base_y + *height_in_rows as f32 * cell;
    elements.push_str(&format!(
        r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="black" stroke-width="1.5"/>"#,
        line_x, line_y1, line_x, line_y2
    ));
}
```

- [ ] **Step 5: Add PartLabel stub to renderer** (so it compiles)

In `src/renderer.rs`, add to the match in `render_page`:

```rust
GridContent::PartLabel { text } => {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        x, y, base_font_size * 0.8, escape_xml(text)
    ));
}
```

- [ ] **Step 6: Run all tests — expect pass**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test 2>&1 | tail -20
```

- [ ] **Step 7: Commit**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && git add src/layout/types.rs src/renderer.rs src/layout/mod.rs && git commit -m "feat: add PartLabel and BarLine height_in_rows to layout types"
```

---

## Task 6: Multi-part layout

**Files:**
- Modify: `src/layout/mod.rs`

This is the core layout rewrite. The layout must: reserve label columns, stack parts vertically, emit directives on every part's row, emit `PartLabel` at line starts, compute per-measure max column width, use `BarLine { height_in_rows }` spanning all parts.

- [ ] **Step 1: Write failing tests**

Add to `src/layout/mod.rs` tests:

```rust
fn make_two_part_score_raw(s_notes: &str, a_notes: &str) -> Score {
    let input = format!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n[score:Soprano]\n4/4 {}\n[score:Alto]\n{}\n",
        s_notes, a_notes
    );
    let doc = parser::parse(&input, "test.jianpu").unwrap();
    grouper::group(doc).unwrap()
}

#[test]
fn two_part_layout_emits_part_labels_for_named_parts() {
    let score = make_two_part_score_raw("1 2 3 4", "5 6 7 1");
    let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
    let labels: Vec<_> = pages.iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .filter(|e| matches!(&e.content, GridContent::PartLabel { .. }))
        .collect();
    assert_eq!(labels.len(), 2, "expected one PartLabel per named part on the first system");
}

#[test]
fn two_part_layout_has_note_heads_for_both_parts() {
    let score = make_two_part_score_raw("1 2 3 4", "5 6 7 1");
    let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
    let note_heads: Vec<_> = pages.iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .filter(|e| matches!(e.content, GridContent::NoteHead { .. }))
        .collect();
    assert_eq!(note_heads.len(), 8, "expected 4 notes per part × 2 parts");
}

#[test]
fn two_part_layout_emits_directives_on_both_parts_rows() {
    let score = make_two_part_score_raw("1 2 3 4", "5 6 7 1");
    let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
    let time_sig_labels: Vec<_> = pages.iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .filter(|e| matches!(e.content, GridContent::TimeSignatureLabel { .. }))
        .collect();
    assert_eq!(time_sig_labels.len(), 2, "time signature label should appear on both parts' rows");
}

#[test]
fn single_unnamed_part_produces_no_part_labels() {
    let score = make_score("1 2 3 4", "a b c d");
    let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
    let labels: Vec<_> = pages.iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .filter(|e| matches!(e.content, GridContent::PartLabel { .. }))
        .collect();
    assert_eq!(labels.len(), 0);
}
```

- [ ] **Step 2: Run tests — expect failures**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test layout 2>&1 | tail -30
```

- [ ] **Step 3: Rewrite `src/layout/mod.rs`**

Replace the entire file with the multi-part implementation:

```rust
use crate::ast::grouped::{MultiPartMeasure, NoteEvent, Score};
use crate::ast::parsed::JianPuPitch;
use crate::layout::types::*;
use crate::utils::is_cjk_char;

pub mod types;

struct BeamBufferEntry {
    column: u32,
    underline_count: u32,
    duration: u32,
}

fn flush_beam_buffer(
    buffer: &mut Vec<BeamBufferEntry>,
    row_offset: u32,
    elements: &mut Vec<GridElement>,
) {
    if buffer.is_empty() { return; }
    let levels = compute_underline_levels(buffer);
    elements.push(GridElement {
        position: GridPosition { column: buffer[0].column, row: row_offset + 2 },
        horizontal_alignment: HorizontalAlignment::Left,
        vertical_alignment: VerticalAlignment::Top,
        content: GridContent::DurationUnderlines { levels },
    });
    buffer.clear();
}

fn compute_underline_levels(buffer: &[BeamBufferEntry]) -> Vec<UnderlineSpan> {
    let first = &buffer[0];
    let last = &buffer[buffer.len() - 1];
    let mut levels = vec![UnderlineSpan {
        from_column: first.column,
        to_column: last.column + last.duration,
    }];
    let mut run_start: Option<u32> = None;
    let mut run_end: u32 = 0;
    for entry in buffer {
        if entry.underline_count >= 2 {
            if run_start.is_none() { run_start = Some(entry.column); }
            run_end = entry.column + entry.duration;
        } else if let Some(start) = run_start.take() {
            levels.push(UnderlineSpan { from_column: start, to_column: run_end });
        }
    }
    if let Some(start) = run_start {
        levels.push(UnderlineSpan { from_column: start, to_column: run_end });
    }
    levels
}

fn measure_column_width(measure: &MultiPartMeasure) -> u32 {
    let max_notes: u32 = measure.parts.iter().map(|part| {
        part.notes.events.iter().map(|n| match n {
            NoteEvent::Note(note) => note.duration,
            NoteEvent::Rest(rest) => rest.duration,
        }).sum::<u32>()
    }).max().unwrap_or(0);
    max_notes + 1 // +1 for bar line
}

fn compute_prefix_width(measure: &MultiPartMeasure) -> u32 {
    let mut width = 0;
    if measure.time_signature.is_some() { width += 2; }
    if measure.bpm.is_some() { width += 2; }
    // key (1=X) has no visual label in the current renderer — not counted here
    width
}

const PAGE_MARGIN: f32 = 25.0;

pub fn layout(score: &Score, page_width_pt: f32, page_height_pt: f32) -> Vec<Page> {
    let cell = score.metadata.cell_size as f32;
    let usable_width = page_width_pt - 2.0 * PAGE_MARGIN;
    let columns_per_page = (usable_width / cell) as u32;

    let num_parts = score.measures.first().map(|m| m.parts.len()).unwrap_or(1).max(1);
    let row_group_height: u32 = 4 * num_parts as u32;

    let has_named_parts = score.measures.first()
        .map(|m| m.parts.iter().any(|p| p.name.is_some()))
        .unwrap_or(false);
    let label_cols: u32 = if has_named_parts {
        ((score.metadata.label_width as f32 / cell).ceil()) as u32
    } else {
        0
    };

    let header_rows: u32 = if score.metadata.subtitle.is_some() { 3 } else { 2 };
    let footer_rows: u32 = 1;
    let reserved_rows = header_rows + footer_rows;
    let usable_height = page_height_pt - 2.0 * PAGE_MARGIN;
    let row_groups_per_page = ((usable_height / cell) as u32 - reserved_rows) / row_group_height;

    let make_header = || Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
    };

    let mut pages: Vec<Page> = Vec::new();
    let mut current_page_row_groups: Vec<RowGroup> = Vec::new();
    let mut current_elements: Vec<GridElement> = Vec::new();
    // current_col starts at label_cols (label columns are reserved at col 0..label_cols)
    let mut current_col: u32 = label_cols;
    let mut current_row_offset: u32 = header_rows;
    let mut is_line_start: bool = true;

    let first_measure_parts: Vec<Option<String>> = score.measures.first()
        .map(|m| m.parts.iter().map(|p| p.name.clone()).collect())
        .unwrap_or_default();

    let emit_part_labels = |elements: &mut Vec<GridElement>, row_offset: u32| {
        if !has_named_parts { return; }
        for (part_idx, name_opt) in first_measure_parts.iter().enumerate() {
            if let Some(name) = name_opt {
                let part_row = row_offset + part_idx as u32 * 4;
                elements.push(GridElement {
                    position: GridPosition { column: 0, row: part_row + 1 },
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Center,
                    content: GridContent::PartLabel { text: name.clone() },
                });
            }
        }
    };

    for measure in &score.measures {
        let prefix_width = compute_prefix_width(measure);
        let measure_width = measure_column_width(measure);

        if current_col + prefix_width + measure_width > columns_per_page {
            // Flush current line
            for part_idx in 0..num_parts {
                let part_row = current_row_offset + part_idx as u32 * 4;
                // Flush any open beam buffer (we keep per-part buffers below;
                // here we just close the line — beam buffers flushed per-part in note processing)
                let _ = part_row; // beam buffers handled per-part below
            }
            let width = current_col;
            if !current_elements.is_empty() {
                current_page_row_groups.push(RowGroup {
                    elements: std::mem::take(&mut current_elements),
                    height_in_rows: row_group_height,
                    width_in_columns: width,
                });
            }
            current_col = label_cols;
            current_row_offset += row_group_height;
            is_line_start = true;

            if current_page_row_groups.len() >= row_groups_per_page as usize {
                if !current_page_row_groups.is_empty() {
                    pages.push(Page {
                        header: make_header(),
                        footer: Footer { page: pages.len() as u32 + 1, total: 0 },
                        row_groups: std::mem::take(&mut current_page_row_groups),
                        page_width_pt,
                    });
                }
                current_row_offset = header_rows;
            }
        }

        if is_line_start {
            emit_part_labels(&mut current_elements, current_row_offset);
            is_line_start = false;
        }

        // Compute directive start column and notes start column
        let directive_col_start = current_col;
        let mut directive_advance = 0u32;

        // Emit directives for every part
        for part_idx in 0..num_parts {
            let part_row = current_row_offset + part_idx as u32 * 4;
            let mut dc = directive_col_start;

            if let Some(ts) = &measure.time_signature {
                current_elements.push(GridElement {
                    position: GridPosition { column: dc, row: part_row + 1 },
                    horizontal_alignment: HorizontalAlignment::Center,
                    vertical_alignment: VerticalAlignment::Center,
                    content: GridContent::TimeSignatureLabel {
                        numerator: ts.numerator,
                        denominator: ts.denominator,
                    },
                });
                if part_idx == 0 { directive_advance += 2; }
                dc += 2;
            }

            if let Some(bpm) = &measure.bpm {
                current_elements.push(GridElement {
                    position: GridPosition { column: dc, row: part_row + 1 },
                    horizontal_alignment: HorizontalAlignment::Center,
                    vertical_alignment: VerticalAlignment::Center,
                    content: GridContent::BpmLabel { bpm: *bpm },
                });
                if part_idx == 0 { directive_advance += 2; }
            }
        }

        let note_col_start = directive_col_start + directive_advance;

        // Compute max notes width for bar line placement
        let max_notes_width: u32 = measure.parts.iter().map(|part| {
            part.notes.events.iter().map(|n| match n {
                NoteEvent::Note(note) => note.duration,
                NoteEvent::Rest(rest) => rest.duration,
            }).sum::<u32>()
        }).max().unwrap_or(0);

        // Emit notes for each part
        for (part_idx, part_slice) in measure.parts.iter().enumerate() {
            let part_row = current_row_offset + part_idx as u32 * 4;
            let mut col = note_col_start;
            let measure_col_start = note_col_start;

            let mut beam_buffer: Vec<BeamBufferEntry> = Vec::new();
            let mut pending_chain: Vec<(u32, JianPuPitch)> = Vec::new();
            let mut chain_row: u32 = part_row + 1;
            let mut prev_tie = false;
            let mut prev_pitch: Option<JianPuPitch> = None;

            let mut lyrics_iter = part_slice.lyrics.as_ref()
                .map(|l| l.syllables.iter())
                .into_iter()
                .flatten();

            for note_event in &part_slice.notes.events {
                match note_event {
                    NoteEvent::Note(note) => {
                        // Note head (row +1)
                        current_elements.push(GridElement {
                            position: GridPosition { column: col, row: part_row + 1 },
                            horizontal_alignment: HorizontalAlignment::Center,
                            vertical_alignment: VerticalAlignment::Center,
                            content: GridContent::NoteHead { pitch: note.pitch.clone(), octave: note.octave },
                        });

                        // Lower octave dots (row +2)
                        if note.octave < 0 {
                            current_elements.push(GridElement {
                                position: GridPosition { column: col, row: part_row + 2 },
                                horizontal_alignment: HorizontalAlignment::Center,
                                vertical_alignment: VerticalAlignment::Bottom,
                                content: GridContent::LowerOctaveDots { count: (-note.octave) as u32 },
                            });
                        }

                        // Extension dashes (row +1)
                        if note.duration > 4 {
                            let extra_beats = (note.duration - 4) / 4;
                            for i in 0..extra_beats {
                                current_elements.push(GridElement {
                                    position: GridPosition { column: col + 4 + i * 4, row: part_row + 1 },
                                    horizontal_alignment: HorizontalAlignment::Center,
                                    vertical_alignment: VerticalAlignment::Center,
                                    content: GridContent::Extension,
                                });
                            }
                        }

                        let underline_count = match note.duration {
                            1 => 2,
                            2 => 1,
                            _ => 0,
                        };

                        if underline_count == 0 {
                            flush_beam_buffer(&mut beam_buffer, part_row, &mut current_elements);
                        }

                        if pending_chain.is_empty() { chain_row = part_row + 1; }
                        pending_chain.push((col, note.pitch.clone()));

                        // Lyric (row +3)
                        let is_tie_continuation = prev_tie && prev_pitch.as_ref() == Some(&note.pitch);
                        if !is_tie_continuation {
                            if let Some(syllable) = lyrics_iter.next() {
                                let is_cjk = syllable.text.chars().next().map(|c| is_cjk_char(c)).unwrap_or(false);
                                current_elements.push(GridElement {
                                    position: GridPosition { column: col, row: part_row + 3 },
                                    horizontal_alignment: HorizontalAlignment::Center,
                                    vertical_alignment: VerticalAlignment::Top,
                                    content: GridContent::Lyric { text: syllable.text.clone(), is_cjk },
                                });
                            }
                        }
                        prev_tie = note.tie;
                        prev_pitch = Some(note.pitch.clone());

                        if underline_count > 0 {
                            beam_buffer.push(BeamBufferEntry {
                                column: col,
                                underline_count,
                                duration: note.duration,
                            });
                        }

                        col += note.duration;

                        let beat_position = col - measure_col_start;
                        if underline_count > 0 && beat_position % 4 == 0 {
                            flush_beam_buffer(&mut beam_buffer, part_row, &mut current_elements);
                        }

                        if !note.tie {
                            flush_chain(&pending_chain, chain_row, &mut current_elements);
                            pending_chain.clear();
                        }
                    }
                    NoteEvent::Rest(rest) => {
                        flush_beam_buffer(&mut beam_buffer, part_row, &mut current_elements);
                        current_elements.push(GridElement {
                            position: GridPosition { column: col, row: part_row + 1 },
                            horizontal_alignment: HorizontalAlignment::Center,
                            vertical_alignment: VerticalAlignment::Center,
                            content: GridContent::Rest,
                        });
                        col += rest.duration;
                        prev_tie = false;
                    }
                }
            }

            flush_beam_buffer(&mut beam_buffer, part_row, &mut current_elements);
        }

        // Bar line spanning all parts (placed at first part's note row)
        let bar_col = note_col_start + max_notes_width;
        let bar_height_in_rows = 1 + (num_parts as u32 - 1) * 4;
        current_elements.push(GridElement {
            position: GridPosition { column: bar_col, row: current_row_offset + 1 },
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
            content: GridContent::BarLine { height_in_rows: bar_height_in_rows },
        });

        current_col = bar_col + 1;
    }

    // Flush remaining
    if !current_elements.is_empty() {
        current_page_row_groups.push(RowGroup {
            elements: std::mem::take(&mut current_elements),
            height_in_rows: row_group_height,
            width_in_columns: current_col,
        });
    }
    if !current_page_row_groups.is_empty() {
        pages.push(Page {
            header: make_header(),
            footer: Footer { page: pages.len() as u32 + 1, total: 0 },
            row_groups: std::mem::take(&mut current_page_row_groups),
            page_width_pt,
        });
    }

    if pages.is_empty() {
        pages.push(Page {
            header: make_header(),
            footer: Footer { page: 1, total: 1 },
            row_groups: Vec::new(),
            page_width_pt,
        });
    }

    let total = pages.len() as u32;
    for page in &mut pages {
        page.footer.total = total;
    }

    pages
}

fn flush_chain(chain: &[(u32, JianPuPitch)], chain_row: u32, elements: &mut Vec<GridElement>) {
    if chain.len() <= 1 { return; }
    let has_pitch_change = chain.windows(2).any(|w| w[0].1 != w[1].1);
    if has_pitch_change {
        elements.push(GridElement {
            position: GridPosition { column: chain[0].0, row: chain_row },
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
            content: GridContent::TieOrSlurCurve {
                from_column: chain[0].0,
                to_column: chain.last().unwrap().0,
            },
        });
    }
    for w in chain.windows(2) {
        if w[0].1 == w[1].1 {
            elements.push(GridElement {
                position: GridPosition { column: w[0].0, row: chain_row },
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
                content: GridContent::TieOrSlurCurve {
                    from_column: w[0].0,
                    to_column: w[1].0,
                },
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::grouper;

    const A4_WIDTH: f32 = 595.0;
    const A4_HEIGHT: f32 = 842.0;

    fn make_score(score_str: &str, lyrics_str: &str) -> Score {
        let input = format!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 {}\n\n[lyrics]\n{}\n",
            score_str, lyrics_str
        );
        let doc = parser::parse(&input, "test.jianpu").unwrap();
        grouper::group(doc).unwrap()
    }

    fn make_score_raw(score_section: &str, lyrics_str: &str) -> Score {
        let input = format!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n{}\n\n[lyrics]\n{}\n",
            score_section, lyrics_str
        );
        let doc = parser::parse(&input, "test.jianpu").unwrap();
        grouper::group(doc).unwrap()
    }

    fn make_two_part_score_raw(s_notes: &str, a_notes: &str) -> Score {
        let input = format!(
            "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n[score:Soprano]\n4/4 {}\n[score:Alto]\n{}\n",
            s_notes, a_notes
        );
        let doc = parser::parse(&input, "test.jianpu").unwrap();
        grouper::group(doc).unwrap()
    }

    fn collect_curves(pages: &[Page]) -> Vec<(u32, u32)> {
        pages.iter().flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter_map(|e| match &e.content {
                GridContent::TieOrSlurCurve { from_column, to_column } => Some((*from_column, *to_column)),
                _ => None,
            }).collect()
    }

    fn collect_lyric_positions(pages: &[Page]) -> Vec<(u32, String)> {
        pages.iter().flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter_map(|e| match &e.content {
                GridContent::Lyric { text, .. } => Some((e.position.column, text.clone())),
                _ => None,
            }).collect()
    }

    fn collect_underline_levels(pages: &[Page]) -> Vec<Vec<UnderlineSpan>> {
        pages.iter().flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter_map(|e| match &e.content {
                GridContent::DurationUnderlines { levels } => Some(levels.clone()),
                _ => None,
            }).collect()
    }

    #[test]
    fn first_measure_emits_time_signature_label() {
        let score = make_score("1 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let labels: Vec<_> = pages[0].row_groups.iter()
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::TimeSignatureLabel { .. }))
            .collect();
        assert_eq!(labels.len(), 1);
        if let GridContent::TimeSignatureLabel { numerator, denominator } = &labels[0].content {
            assert_eq!(*numerator, 4);
            assert_eq!(*denominator, 4);
        }
    }

    #[test]
    fn first_measure_emits_bpm_label() {
        let score = make_score("1 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let labels: Vec<_> = pages[0].row_groups.iter()
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::BpmLabel { .. }))
            .collect();
        assert_eq!(labels.len(), 1);
        if let GridContent::BpmLabel { bpm } = &labels[0].content {
            assert_eq!(*bpm, 120);
        }
    }

    #[test]
    fn unchanged_time_signature_emits_no_second_label() {
        // Both measures have same time sig → grouper sets None on second → layout emits only 1
        let score = make_score("1 2 3 4 5 6 7 1", "a b c d e f g h");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let labels: Vec<_> = pages.iter()
            .flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::TimeSignatureLabel { .. }))
            .collect();
        assert_eq!(labels.len(), 1);
    }

    #[test]
    fn time_signature_change_emits_second_label() {
        let score = make_score_raw("4/4 1 2 3 4 3/4 1 2 3", "a b c d e f g");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let labels: Vec<_> = pages.iter()
            .flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::TimeSignatureLabel { .. }))
            .collect();
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn bpm_change_emits_second_label() {
        let score = make_score_raw("4/4 bpm=120 1 2 3 4 bpm=90 5 6 7 1", "a b c d e f g h");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let labels: Vec<_> = pages.iter()
            .flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::BpmLabel { .. }))
            .collect();
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn header_is_populated_on_every_page() {
        let score = make_score("1 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        for page in &pages {
            assert_eq!(page.header.title, "t");
            assert_eq!(page.header.author, "a");
        }
    }

    #[test]
    fn footer_page_numbers_are_correct() {
        let score = make_score("1 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let total = pages.len() as u32;
        for (i, page) in pages.iter().enumerate() {
            assert_eq!(page.footer.page, i as u32 + 1);
            assert_eq!(page.footer.total, total);
        }
    }

    #[test]
    fn produces_at_least_one_page() {
        let score = make_score("1 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        assert!(!pages.is_empty());
    }

    #[test]
    fn note_heads_are_present() {
        let score = make_score("1 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let note_heads: Vec<_> = pages[0].row_groups.iter()
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::NoteHead { .. }))
            .collect();
        assert_eq!(note_heads.len(), 4);
    }

    #[test]
    fn lyrics_are_present() {
        let score = make_score("1 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let lyrics: Vec<_> = pages[0].row_groups.iter()
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::Lyric { .. }))
            .collect();
        assert_eq!(lyrics.len(), 4);
    }

    #[test]
    fn two_different_notes_emit_one_slur() {
        let score = make_score("1~ 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let curves = collect_curves(&pages);
        assert_eq!(curves.len(), 1);
    }

    #[test]
    fn tied_notes_share_one_lyric_syllable() {
        let score = make_score("3~3 1 2", "a b c");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let lyric_positions = collect_lyric_positions(&pages);
        assert_eq!(lyric_positions.len(), 3);
    }

    #[test]
    fn consecutive_eighth_notes_share_one_underline() {
        let score = make_score("_2 _2 0 0 0", "a b");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let groups = collect_underline_levels(&pages);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 1);
    }

    #[test]
    fn lower_octave_note_emits_lower_octave_dots_element() {
        let score = make_score("1. 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let lower_dots: Vec<_> = pages[0].row_groups.iter()
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::LowerOctaveDots { .. }))
            .collect();
        assert_eq!(lower_dots.len(), 1);
    }

    // ── Multi-part tests ─────────────────────────────────────────────────────

    #[test]
    fn two_part_layout_emits_part_labels_for_named_parts() {
        let score = make_two_part_score_raw("1 2 3 4", "5 6 7 1");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let labels: Vec<_> = pages.iter()
            .flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(&e.content, GridContent::PartLabel { .. }))
            .collect();
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn two_part_layout_has_note_heads_for_both_parts() {
        let score = make_two_part_score_raw("1 2 3 4", "5 6 7 1");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let note_heads: Vec<_> = pages.iter()
            .flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::NoteHead { .. }))
            .collect();
        assert_eq!(note_heads.len(), 8);
    }

    #[test]
    fn two_part_layout_emits_directives_on_both_parts_rows() {
        let score = make_two_part_score_raw("1 2 3 4", "5 6 7 1");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let time_sig_labels: Vec<_> = pages.iter()
            .flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::TimeSignatureLabel { .. }))
            .collect();
        assert_eq!(time_sig_labels.len(), 2);
    }

    #[test]
    fn single_unnamed_part_produces_no_part_labels() {
        let score = make_score("1 2 3 4", "a b c d");
        let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
        let labels: Vec<_> = pages.iter()
            .flat_map(|p| p.row_groups.iter())
            .flat_map(|rg| rg.elements.iter())
            .filter(|e| matches!(e.content, GridContent::PartLabel { .. }))
            .collect();
        assert_eq!(labels.len(), 0);
    }
}
```

- [ ] **Step 4: Run all tests — expect pass**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test 2>&1 | tail -30
```

- [ ] **Step 5: Commit**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && git add src/layout/mod.rs && git commit -m "feat: multi-part layout with stacked parts, label columns, directive duplication"
```

---

## Task 7: Renderer tests — update for new Score shape

**Files:**
- Modify: `src/renderer.rs`

The renderer tests call `grouper::group(doc)` and access `score.metadata.cell_size`. These still work. But they call `layout::layout(&score, ...)` which now uses the new `Score`. The renderer integration tests should still pass as-is after Task 6.

- [ ] **Step 1: Run renderer tests**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test renderer 2>&1 | tail -20
```

Expected: all renderer tests pass (they use the existing single-part API which remains compatible).

- [ ] **Step 2: Add a multi-part renderer test**

Add to `src/renderer.rs` tests:

```rust
#[test]
fn multi_part_svg_contains_both_part_labels() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n",
        "[score:Soprano]\n4/4 1 2 3 4\n",
        "[lyrics:Soprano]\na b c d\n",
        "[score:Alto]\n5 6 7 1\n",
        "[lyrics:Alto]\ne f g h\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(&pages, score.metadata.cell_size);
    assert!(svgs[0].contains("Soprano"), "expected 'Soprano' label in SVG");
    assert!(svgs[0].contains("Alto"), "expected 'Alto' label in SVG");
}
```

- [ ] **Step 3: Run tests — expect pass**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test renderer 2>&1 | tail -20
```

- [ ] **Step 4: Commit**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && git add src/renderer.rs && git commit -m "test: add multi-part renderer test for part labels"
```

---

## Task 8: Update demo.jianpu

**Files:**
- Modify: `demo.jianpu`

- [ ] **Step 1: Update `demo.jianpu`**

Replace `cell_size = 20` with `cell size = 20` and append a multi-part section after the existing content:

```
[metadata]
title = "Feature Demo"
author = "Jianpu Generator"
cell size = 20

[score]
bpm=100 1=C4 4/4

1 - 2 0 |
3~ 3 2 1 |
...rest of existing score...

[lyrics]
...existing lyrics...

[score:Soprano]
4/4 1 2 3 4 |
5 6 7 1 |

[lyrics:Soprano]
do re mi fa sol la ti do

[score:Alto]
5 6 7 1 |
3 4 5 6 |

[lyrics:Alto]
sol la ti do mi fa sol la
```

- [ ] **Step 2: Verify the demo compiles and runs**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo run -- demo.jianpu --svg -o /tmp/demo.svg 2>&1
```

Expected: exits 0, writes `/tmp/demo.svg`.

- [ ] **Step 3: Commit**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && git add demo.jianpu && git commit -m "feat: update demo.jianpu with cell size rename and multi-part example"
```

---

## Final Verification

- [ ] **Run full test suite**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo test 2>&1 | tail -10
```

Expected output: `test result: ok. N passed; 0 failed`.

- [ ] **Build release binary**

```bash
cd /Users/wongjiahau/personal-repos/jianpu-generator && cargo build --release 2>&1 | tail -5
```

Expected: `Finished release [optimized] target`.
