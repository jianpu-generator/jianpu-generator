# Label Directive Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `label="..."` directive parseable inside parenthesised directive lines, rendered as italic text above the row group where declared.

**Architecture:** Follow the existing bpm/key/time pipeline — parse to `ScoreEvent::LabelChange`, propagate through `GroupedMeasure` → `MultiPartMeasure`, emit `GridContent::SectionLabel` in the layout, and render as italic SVG text in row +0 (bar-number row) left-aligned above the notes.

**Tech Stack:** Rust, existing codebase (no new dependencies)

---

## File Map

| File | Change |
|------|--------|
| `src/ast/parsed.rs` | Add `ScoreEvent::LabelChange(String)` |
| `src/parser/score/interleaved_parser.rs` | Add quote-aware directive tokenizer; parse `label="..."` |
| `src/ast/grouped.rs` | Add `label: Option<String>` to `GroupedMeasure` and `MultiPartMeasure` |
| `src/grouper.rs` | Handle `LabelChange` event; set `pending_label`; flush into measure |
| `src/combiner.rs` | Propagate `label` from first part's measure |
| `src/layout/types.rs` | Add `GridContent::SectionLabel { text: String }` |
| `src/layout/mod.rs` | Emit `SectionLabel` at row +0 when `measure.label` is `Some` |
| `src/renderer.rs` | Render `SectionLabel` as italic SVG text |

---

### Task 1: Parse `label="..."` directive → `ScoreEvent::LabelChange`

**Files:**
- Modify: `src/ast/parsed.rs`
- Modify: `src/parser/score/interleaved_parser.rs`

- [ ] **Step 1: Add `LabelChange` variant to `ScoreEvent` in `src/ast/parsed.rs`**

Find the `ScoreEvent` enum and add the new variant after `TimeSignatureChange`:

```rust
pub enum ScoreEvent {
    Note(ParsedNote),
    Rest(ParsedRest),
    BpmChange(u32),
    KeyChange(KeyChange),
    TimeSignatureChange { numerator: u8, denominator: u8 },
    /// The `-` token: extends the previous note/rest by one full beat (4 quarter-beats).
    Extension,
    LabelChange(String),
}
```

- [ ] **Step 2: Write the failing test for `label=` parsing**

Add to the `#[cfg(test)]` block at the bottom of `src/parser/score/interleaved_parser.rs`:

```rust
#[test]
fn label_directive_parsed() {
    let content = "(time=4/4 key=C4 bpm=120 label=\"Verse 1\")\n1 2 3 4\n";
    let parts = vec![notes_col("")];
    let result = parse(content, &parts).unwrap();
    use crate::ast::parsed::ScoreEvent;
    let label_event = result[0].score.events.iter()
        .find(|e| matches!(&e.value, ScoreEvent::LabelChange(_)));
    assert!(label_event.is_some(), "expected a LabelChange event");
    if let ScoreEvent::LabelChange(text) = &label_event.unwrap().value {
        assert_eq!(text, "Verse 1");
    }
}

#[test]
fn label_directive_rejects_unclosed_quote() {
    let content = "(label=\"Verse 1)\n1 2 3 4\n";
    let parts = vec![notes_col("")];
    assert!(parse(content, &parts).is_err());
}
```

- [ ] **Step 3: Run the tests to verify they fail**

```bash
cargo test label_directive_parsed label_directive_rejects_unclosed_quote 2>&1 | grep -E "FAILED|error"
```

Expected: compile error (unknown variant `LabelChange`) or test failure.

- [ ] **Step 4: Replace the whitespace-only tokenizer with a quote-aware one and handle `label=`**

In `src/parser/score/interleaved_parser.rs`, replace the `parse_directive_line` function and add a new `tokenize_directive_tokens` helper:

```rust
fn tokenize_directive_tokens(inner: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;

    for ch in inner.chars() {
        if in_quote {
            current.push(ch);
            if ch == '"' {
                in_quote = false;
            }
        } else if ch == '"' {
            current.push(ch);
            in_quote = true;
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    if in_quote {
        return Err("unclosed quote in directive line".to_string());
    }
    Ok(tokens)
}

fn parse_directive_line(line: &str) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
    let inner = &line[1..line.len() - 1];
    let tokens = tokenize_directive_tokens(inner).map_err(|msg| {
        JianPuError::new(Span::new(0, line.len()), msg)
    })?;
    let mut events = Vec::new();

    for token in &tokens {
        let span = Span::new(0, token.len());

        let event = if let Some(rest) = token.strip_prefix("bpm=") {
            let bpm = rest.parse::<u32>().map_err(|_| {
                JianPuError::new(span.clone(), format!("invalid bpm value: {}", rest))
            })?;
            ScoreEvent::BpmChange(bpm)
        } else if let Some(rest) = token.strip_prefix("key=") {
            parse_key_value(rest, span.clone())?
        } else if let Some(rest) = token.strip_prefix("time=") {
            parse_time_value(rest, span.clone())?
        } else if let Some(rest) = token.strip_prefix("label=") {
            if rest.len() < 2 || !rest.starts_with('"') || !rest.ends_with('"') {
                return Err(JianPuError::new(
                    span,
                    format!("label value must be a quoted string, got: {}", rest),
                ));
            }
            let text = rest[1..rest.len() - 1].to_string();
            ScoreEvent::LabelChange(text)
        } else {
            return Err(JianPuError::new(span, format!("unknown directive: '{}'", token)));
        };

        events.push(Spanned::new(event, span));
    }

    Ok(events)
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test label_directive_parsed label_directive_rejects_unclosed_quote 2>&1 | grep -E "test.*ok|FAILED|error"
```

Expected:
```
test parser::score::interleaved_parser::tests::label_directive_parsed ... ok
test parser::score::interleaved_parser::tests::label_directive_rejects_unclosed_quote ... ok
```

- [ ] **Step 6: Add placeholder arm to grouper to keep the match exhaustive**

The new `LabelChange` variant makes the `match spanned.value` in `src/grouper.rs` non-exhaustive. Add a no-op arm after the `Rest` arm (it will be replaced by the real handler in Task 2):

```rust
ScoreEvent::LabelChange(_) => {}
```

- [ ] **Step 7: Run full test suite to verify no regressions**

```bash
cargo test 2>&1 | tail -5
```

Expected: `test result: ok.`

- [ ] **Step 8: Commit**

```bash
git add src/ast/parsed.rs src/parser/score/interleaved_parser.rs src/grouper.rs
git commit -m "feat: parse label directive into ScoreEvent::LabelChange"
```

---

### Task 2: Propagate `label` through grouped AST, grouper, and combiner

**Files:**
- Modify: `src/ast/grouped.rs`
- Modify: `src/grouper.rs`
- Modify: `src/combiner.rs`

- [ ] **Step 1: Write the failing tests in `src/grouper.rs`**

Add to the `#[cfg(test)]` block in `src/grouper.rs`:

```rust
#[test]
fn label_directive_propagates_to_measure() {
    let score = parse_and_group(concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\nparts = notes:\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120 label=\"Verse 1\")\n1 2 3 4\n",
    ));
    assert_eq!(score.measures[0].label, Some("Verse 1".to_string()));
}

#[test]
fn label_is_none_when_not_declared() {
    let score = parse_and_group(concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\nparts = notes:\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120)\n1 2 3 4\n",
    ));
    assert_eq!(score.measures[0].label, None);
}

#[test]
fn label_does_not_persist_to_next_measure() {
    let score = parse_and_group(concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\nparts = notes:\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120 label=\"Verse 1\")\n1 2 3 4\n\n5 6 7 1\n",
    ));
    assert_eq!(score.measures[0].label, Some("Verse 1".to_string()));
    assert_eq!(score.measures[1].label, None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test label_directive_propagates_to_measure label_is_none_when_not_declared label_does_not_persist_to_next_measure 2>&1 | grep -E "FAILED|error"
```

Expected: compile error — `label` field doesn't exist on `MultiPartMeasure`.

- [ ] **Step 3: Add `label` field to `GroupedMeasure` and `MultiPartMeasure` in `src/ast/grouped.rs`**

Add `pub label: Option<String>` to both structs:

```rust
pub struct MultiPartMeasure {
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<u32>,
    // TODO: key-change rendering (1=X label) is not yet implemented in layout/renderer
    pub key: Option<KeyChange>,
    pub label: Option<String>,
    pub parts: Vec<PartSlice>,
}
```

```rust
pub(crate) struct GroupedMeasure {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) label: Option<String>,
    pub(crate) notes: Notes,
}
```

- [ ] **Step 4: Handle `LabelChange` in `src/grouper.rs`**

In `group_part`, make the following changes:

1. Add `pending_label: Option<String> = None;` after the other state variables (around line 61):

```rust
let mut current_beat: u32 = 0;
let mut capacity = measure_capacity(&current_time_sig);
let mut pending_label: Option<String> = None;
```

2. Add `label: pending_label.take(),` to the `GroupedMeasure` construction inside `flush_measure!()`:

```rust
macro_rules! flush_measure {
    () => {
        if !current_notes.is_empty() {
            measures.push(GroupedMeasure {
                time_signature: if time_sig_changed {
                    Some(TimeSignature {
                        numerator: current_time_sig.numerator,
                        denominator: current_time_sig.denominator,
                    })
                } else {
                    None
                },
                bpm: if bpm_changed { Some(current_bpm) } else { None },
                key: if key_changed {
                    Some(current_key.clone())
                } else {
                    None
                },
                label: pending_label.take(),
                notes: Notes {
                    events: std::mem::take(&mut current_notes),
                },
            });
            current_beat = 0;
            bpm_changed = false;
            key_changed = false;
            time_sig_changed = false;
        }
    };
}
```

3. Replace the placeholder `ScoreEvent::LabelChange(_) => {}` arm added in Task 1 with the real handler:

```rust
ScoreEvent::LabelChange(text) => {
    flush_measure!();
    pending_label = Some(text);
}
```

4. Add `label: pending_label.take(),` to the final inline flush at the bottom of `group_part` (the `if !current_notes.is_empty()` block, around line 190):

```rust
if !current_notes.is_empty() {
    measures.push(GroupedMeasure {
        time_signature: if time_sig_changed {
            Some(TimeSignature {
                numerator: current_time_sig.numerator,
                denominator: current_time_sig.denominator,
            })
        } else {
            None
        },
        bpm: if bpm_changed { Some(current_bpm) } else { None },
        key: if key_changed {
            Some(current_key.clone())
        } else {
            None
        },
        label: pending_label.take(),
        notes: Notes {
            events: std::mem::take(&mut current_notes),
        },
    });
}
```

- [ ] **Step 5: Propagate `label` in `src/combiner.rs`**

In the `combined.push(MultiPartMeasure { ... })` block (around line 58), add `label: first.label.clone()`:

```rust
combined.push(MultiPartMeasure {
    time_signature: first.time_signature.clone(),
    bpm: first.bpm,
    key: first.key.clone(),
    label: first.label.clone(),
    parts: part_slices,
});
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cargo test label_directive_propagates_to_measure label_is_none_when_not_declared label_does_not_persist_to_next_measure 2>&1 | grep -E "test.*ok|FAILED|error"
```

Expected:
```
test grouper::tests::label_directive_propagates_to_measure ... ok
test grouper::tests::label_is_none_when_not_declared ... ok
test grouper::tests::label_does_not_persist_to_next_measure ... ok
```

- [ ] **Step 7: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: `test result: ok.`

- [ ] **Step 8: Commit**

```bash
git add src/ast/grouped.rs src/grouper.rs src/combiner.rs
git commit -m "feat: propagate label through grouped AST, grouper, and combiner"
```

---

### Task 3: Emit and render `GridContent::SectionLabel`

**Files:**
- Modify: `src/layout/types.rs`
- Modify: `src/layout/mod.rs`
- Modify: `src/renderer.rs`

- [ ] **Step 1: Write the failing renderer test**

Add to the `#[cfg(test)]` block in `src/renderer.rs`:

```rust
#[test]
fn section_label_renders_in_svg() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\nparts = notes: lyrics:\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120 label=\"Verse 1\")\n1 2 3 4\na b c d\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(&pages, score.metadata.row_height);
    assert!(svgs[0].contains("Verse 1"), "expected section label 'Verse 1' in SVG");
    assert!(svgs[0].contains("font-style=\"italic\""), "expected italic style on section label");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test section_label_renders_in_svg 2>&1 | grep -E "FAILED|error"
```

Expected: compile error — `SectionLabel` variant doesn't exist.

- [ ] **Step 3: Add `SectionLabel` variant to `GridContent` in `src/layout/types.rs`**

Add the variant to the `GridContent` enum after `BarNumber`:

```rust
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
    HorizontalBar { from_column: u32, to_column: u32 },
    BarNumber { number: u32 },
    SectionLabel { text: String },
}
```

- [ ] **Step 4: Emit `SectionLabel` in `src/layout/mod.rs`**

In the measure loop, after `is_line_start = false;` and before `let directive_col_start = current_col;`, add:

```rust
// Emit section label above the row group (row +0) if present for this measure
if let Some(label_text) = &measure.label {
    current_elements.push(GridElement {
        position: GridPosition { column: current_col, row: current_row_offset },
        horizontal_alignment: HorizontalAlignment::Left,
        vertical_alignment: VerticalAlignment::Bottom,
        content: GridContent::SectionLabel { text: label_text.clone() },
    });
}
```

- [ ] **Step 5: Render `SectionLabel` in `src/renderer.rs`**

Add a match arm in the `match &element.content` block, after the `BarNumber` arm:

```rust
GridContent::SectionLabel { text } => {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="ideographic" font-style="italic" font-family="sans-serif">{}</text>"#,
        x, y, base_font_size * 0.7, escape_xml(text)
    ));
}
```

- [ ] **Step 6: Run the test to verify it passes**

```bash
cargo test section_label_renders_in_svg 2>&1 | grep -E "test.*ok|FAILED|error"
```

Expected:
```
test renderer::tests::section_label_renders_in_svg ... ok
```

- [ ] **Step 7: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: `test result: ok.`

- [ ] **Step 8: Commit**

```bash
git add src/layout/types.rs src/layout/mod.rs src/renderer.rs
git commit -m "feat: render label directive as SectionLabel above row group"
```
