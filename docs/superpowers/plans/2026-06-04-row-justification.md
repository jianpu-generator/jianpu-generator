# Row Justification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Justify every row to the full page width so all rows share the same left/right edges, fixing the misalignment caused by directive prefix columns on the first row.

**Architecture:** Split the single `cell` variable into `row_height` (global, for vertical/font sizing) and `column_width` (per row group, computed as `usable_width / width_in_columns`). Replace `cell_size` metadata with `row_height` + `max_columns`. Enforce `RowGroup.elements` non-emptiness via the `nonempty` crate.

**Tech Stack:** Rust, `nonempty` crate (new dependency)

---

## File Map

| File | Change |
|---|---|
| `Cargo.toml` | Add `nonempty` dependency |
| `src/ast/parsed.rs` | Replace `cell_size` with `row_height` + `max_columns` in `ParsedMetadata` |
| `src/ast/grouped.rs` | Same in `Metadata`; `nonempty` import for `RowGroup` (in layout/types.rs) |
| `src/layout/types.rs` | `RowGroup.elements: Vec<GridElement>` → `NonEmpty<GridElement>` |
| `src/parser/metadata_parser.rs` | Parse `row height` and `max columns` instead of `cell size` |
| `src/grouper.rs` | Populate `row_height` (default 24) and `max_columns` (default 28) |
| `src/layout/mod.rs` | Use `row_height` for vertical calc, `max_columns` for wrap; construct `NonEmpty` at push sites |
| `src/renderer.rs` | Accept `row_height`; compute `column_width` per row group; use each appropriately |
| `src/main.rs` | Pass `row_height` to `renderer::render` |
| `src/pdf.rs` | Same |
| `demo.jianpu` | `cell size = 20` → `row height = 20` |

---

## Task 1: Add `nonempty` dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add the dependency**

In `Cargo.toml`, add to `[dependencies]`:
```toml
nonempty = "0.10"
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1 | head -5
```
Expected: compiles (possibly with unused-import warnings, which is fine).

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add nonempty dependency"
```

---

## Task 2: Rename `cell_size` → `row_height` and add `max_columns` in AST types

**Files:**
- Modify: `src/ast/parsed.rs`
- Modify: `src/ast/grouped.rs`

- [ ] **Step 1: Update `ParsedMetadata` in `src/ast/parsed.rs`**

Replace:
```rust
pub struct ParsedMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub cell_size: Option<u32>,
    pub label_width: Option<u32>,
}
```
With:
```rust
pub struct ParsedMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub row_height: Option<u32>,
    pub max_columns: Option<u32>,
    pub label_width: Option<u32>,
}
```

- [ ] **Step 2: Update `Metadata` in `src/ast/grouped.rs`**

Replace:
```rust
pub struct Metadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    /// Grid cell size in points. Default: 24.
    pub cell_size: u32,
    /// Left margin reserved for part labels in points. Default: 40.
    pub label_width: u32,
}
```
With:
```rust
pub struct Metadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    /// Row height in points. Controls font sizes, dot radii, and all vertical spacing. Default: 24.
    pub row_height: u32,
    /// Maximum logical columns per row before wrapping. Default: 28.
    pub max_columns: u32,
    /// Left margin reserved for part labels in points. Default: 40.
    pub label_width: u32,
}
```

- [ ] **Step 3: Verify compilation fails with useful errors (expected)**

```bash
cargo build 2>&1 | grep "error\[" | head -20
```
Expected: errors pointing to `cell_size` usages in `grouper.rs`, `layout/mod.rs`, `renderer.rs`, `main.rs`, `pdf.rs`. These are fixed in later tasks.

---

## Task 3: Update metadata parser

**Files:**
- Modify: `src/parser/metadata_parser.rs`

- [ ] **Step 1: Update the existing tests — rename `cell_size` tests and add `max_columns` tests**

Replace the entire `#[cfg(test)]` block at the bottom of `src/parser/metadata_parser.rs` with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_title_and_author() {
        let content = "title = \"hello world\"\nauthor = \"foo\"\n";
        let meta = parse_metadata(content, 0).unwrap();
        assert_eq!(meta.title, "hello world");
        assert_eq!(meta.author, "foo");
        assert_eq!(meta.row_height, None);
        assert_eq!(meta.max_columns, None);
        assert_eq!(meta.label_width, None);
    }

    #[test]
    fn parses_optional_row_height() {
        let content = "title = \"t\"\nauthor = \"a\"\nrow height = 16\n";
        let meta = parse_metadata(content, 0).unwrap();
        assert_eq!(meta.row_height, Some(16));
    }

    #[test]
    fn parses_optional_max_columns() {
        let content = "title = \"t\"\nauthor = \"a\"\nmax columns = 32\n";
        let meta = parse_metadata(content, 0).unwrap();
        assert_eq!(meta.max_columns, Some(32));
    }

    #[test]
    fn rejects_missing_title() {
        let content = "author = \"foo\"\n";
        assert!(parse_metadata(content, 0).is_err());
    }

    #[test]
    fn rejects_missing_author() {
        let content = "title = \"foo\"\n";
        assert!(parse_metadata(content, 0).is_err());
    }

    #[test]
    fn rejects_unknown_field() {
        let content = "title = \"t\"\nauthor = \"a\"\nfoo = \"bar\"\n";
        assert!(parse_metadata(content, 0).is_err());
    }

    #[test]
    fn rejects_invalid_row_height() {
        let content = "title = \"t\"\nauthor = \"a\"\nrow height = abc\n";
        assert!(parse_metadata(content, 0).is_err());
    }

    #[test]
    fn rejects_invalid_max_columns() {
        let content = "title = \"t\"\nauthor = \"a\"\nmax columns = 0\n";
        assert!(parse_metadata(content, 0).is_err());
    }

    #[test]
    fn parses_optional_subtitle() {
        let content = "title = \"hello\"\nauthor = \"foo\"\nsubtitle = \"sub\"\n";
        let meta = parse_metadata(content, 0).unwrap();
        assert_eq!(meta.subtitle, Some("sub".to_string()));
    }

    #[test]
    fn subtitle_defaults_to_none() {
        let content = "title = \"t\"\nauthor = \"a\"\n";
        let meta = parse_metadata(content, 0).unwrap();
        assert_eq!(meta.subtitle, None);
    }

    #[test]
    fn rejects_row_height_with_underscore() {
        let content = "title = \"t\"\nauthor = \"a\"\nrow_height = 20\n";
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
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test --lib parser::metadata_parser 2>&1 | tail -20
```
Expected: compile errors or test failures on `cell_size` / missing `row_height` / `max_columns`.

- [ ] **Step 3: Update the parser implementation**

Replace the entire body of `parse_metadata` in `src/parser/metadata_parser.rs` with:

```rust
pub fn parse_metadata(
    content: &str,
    base_offset: usize,
) -> Result<ParsedMetadata, JianPuError> {
    let mut title: Option<String> = None;
    let mut subtitle: Option<String> = None;
    let mut author: Option<String> = None;
    let mut row_height: Option<u32> = None;
    let mut max_columns: Option<u32> = None;
    let mut label_width: Option<u32> = None;
    let mut byte_offset = base_offset;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            byte_offset += line.len() + 1;
            continue;
        }

        let line_span = Span::new(byte_offset, byte_offset + line.len());

        let (key_raw, value_raw) = trimmed.split_once('=').ok_or_else(|| {
            JianPuError::new(line_span.clone(), format!("expected key = value, got: {}", trimmed))
        })?;

        let key = key_raw.trim();
        let value = value_raw.trim().trim_matches('"');

        match key {
            "title" => title = Some(value.to_string()),
            "subtitle" => subtitle = Some(value.to_string()),
            "author" => author = Some(value.to_string()),
            "row height" => {
                row_height = Some(parse_positive_u32("row height", value, &line_span)?);
            }
            "max columns" => {
                max_columns = Some(parse_positive_u32("max columns", value, &line_span)?);
            }
            "label width" => {
                label_width = Some(parse_positive_u32("label width", value, &line_span)?);
            }
            _ => {
                return Err(JianPuError::new(
                    line_span,
                    format!("unknown metadata field: {}", key),
                ))
            }
        }

        byte_offset += line.len() + 1;
    }

    let zero_span = Span::new(base_offset, base_offset);

    Ok(ParsedMetadata {
        title: title
            .ok_or_else(|| JianPuError::new(zero_span.clone(), "missing required field: title"))?,
        subtitle,
        author: author
            .ok_or_else(|| JianPuError::new(zero_span, "missing required field: author"))?,
        row_height,
        max_columns,
        label_width,
    })
}
```

- [ ] **Step 4: Run parser tests**

```bash
cargo test --lib parser::metadata_parser 2>&1 | tail -10
```
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/ast/parsed.rs src/ast/grouped.rs src/parser/metadata_parser.rs
git commit -m "feat: replace cell_size with row_height and max_columns in AST and parser"
```

---

## Task 4: Update grouper defaults

**Files:**
- Modify: `src/grouper.rs`

- [ ] **Step 1: Update the existing defaults test and add a `max_columns` test**

In `src/grouper.rs`, find and replace the `cell_size_defaults_to_24` test:

```rust
#[test]
fn row_height_defaults_to_24() {
    let score = parse_and_group(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4\n\n[lyrics]\na b c d\n",
    );
    assert_eq!(score.metadata.row_height, 24);
}

#[test]
fn max_columns_defaults_to_28() {
    let score = parse_and_group(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 1 2 3 4\n\n[lyrics]\na b c d\n",
    );
    assert_eq!(score.metadata.max_columns, 28);
}
```

- [ ] **Step 2: Run to verify the tests fail**

```bash
cargo test --lib grouper::tests::row_height_defaults_to_24 2>&1 | tail -5
cargo test --lib grouper::tests::max_columns_defaults_to_28 2>&1 | tail -5
```
Expected: compile error (field `cell_size` not found, or similar).

- [ ] **Step 3: Update the `group` function in `src/grouper.rs`**

Replace:
```rust
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
```
With:
```rust
Ok(Score {
    metadata: Metadata {
        title: doc.metadata.title,
        subtitle: doc.metadata.subtitle,
        author: doc.metadata.author,
        row_height: doc.metadata.row_height.unwrap_or(24),
        max_columns: doc.metadata.max_columns.unwrap_or(28),
        label_width: doc.metadata.label_width.unwrap_or(40),
    },
    measures,
})
```

- [ ] **Step 4: Run grouper tests**

```bash
cargo test --lib grouper 2>&1 | tail -10
```
Expected: all grouper tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/grouper.rs
git commit -m "feat: populate row_height and max_columns defaults in grouper"
```

---

## Task 5: Change `RowGroup.elements` to `NonEmpty<GridElement>`

**Files:**
- Modify: `src/layout/types.rs`
- Modify: `src/layout/mod.rs`

- [ ] **Step 1: Update `RowGroup` in `src/layout/types.rs`**

Add the import at the top of the file:
```rust
use nonempty::NonEmpty;
```

Replace:
```rust
pub struct RowGroup {
    pub elements: Vec<GridElement>,
    pub height_in_rows: u32,
    /// Number of grid columns actually used by this row group.
    /// Used by the renderer to center each row individually.
    pub width_in_columns: u32,
}
```
With:
```rust
pub struct RowGroup {
    pub elements: NonEmpty<GridElement>,
    pub height_in_rows: u32,
    /// Number of grid columns actually used by this row group.
    pub width_in_columns: u32,
}
```

- [ ] **Step 2: Update the two `RowGroup` push sites in `src/layout/mod.rs`**

There are exactly two places where a `RowGroup` is constructed. Both have a `if !current_elements.is_empty()` guard. Replace both with `NonEmpty::from_vec`:

First site (inside the wrap block, around line 147):

Replace:
```rust
if !current_elements.is_empty() {
    current_page_row_groups.push(RowGroup {
        elements: std::mem::take(&mut current_elements),
        height_in_rows: row_group_height,
        width_in_columns: current_col,
    });
}
```
With:
```rust
if let Some(elements) = nonempty::NonEmpty::from_vec(std::mem::take(&mut current_elements)) {
    current_page_row_groups.push(RowGroup {
        elements,
        height_in_rows: row_group_height,
        width_in_columns: current_col,
    });
}
```

Second site (end-of-score flush, around line 360):

Replace:
```rust
if !current_elements.is_empty() {
    current_page_row_groups.push(RowGroup {
        elements: std::mem::take(&mut current_elements),
        height_in_rows: row_group_height,
        width_in_columns: current_col,
    });
}
```
With:
```rust
if let Some(elements) = nonempty::NonEmpty::from_vec(std::mem::take(&mut current_elements)) {
    current_page_row_groups.push(RowGroup {
        elements,
        height_in_rows: row_group_height,
        width_in_columns: current_col,
    });
}
```

- [ ] **Step 3: Run layout tests**

```bash
cargo test --lib layout 2>&1 | tail -15
```
Expected: layout tests pass (the renderer tests may still fail due to the `cell_size` rename — that's fine here).

- [ ] **Step 4: Commit**

```bash
git add src/layout/types.rs src/layout/mod.rs
git commit -m "refactor: enforce NonEmpty<GridElement> for RowGroup.elements"
```

---

## Task 6: Update layout to use `row_height` and `max_columns`

**Files:**
- Modify: `src/layout/mod.rs`

- [ ] **Step 1: Replace `cell` with `row_height` and `columns_per_page` with `max_columns`**

In `src/layout/mod.rs`, in the `layout` function body, replace:
```rust
let cell = score.metadata.cell_size as f32;
let usable_width = page_width_pt - 2.0 * PAGE_MARGIN;
let columns_per_page = (usable_width / cell) as u32;
```
With:
```rust
let row_height = score.metadata.row_height as f32;
let columns_per_row = score.metadata.max_columns;
```

Replace:
```rust
let label_cols: u32 = if has_named_parts {
    ((score.metadata.label_width as f32 / cell).ceil()) as u32
} else {
    0
};
```
With:
```rust
let label_cols: u32 = if has_named_parts {
    ((score.metadata.label_width as f32 / row_height).ceil()) as u32
} else {
    0
};
```

Replace:
```rust
let usable_height = page_height_pt - 2.0 * PAGE_MARGIN;
let row_groups_per_page = ((usable_height / cell) as u32 - reserved_rows) / row_group_height;
```
With:
```rust
let usable_height = page_height_pt - 2.0 * PAGE_MARGIN;
let row_groups_per_page = ((usable_height / row_height) as u32 - reserved_rows) / row_group_height;
```

Replace the wrap check:
```rust
if current_col + prefix_width + measure_width > columns_per_page {
```
With:
```rust
if current_col + prefix_width + measure_width > columns_per_row {
```

Also remove the now-unused comment on line 80:
```rust
/// Column width = cell_size, row height = cell_size.
```
Replace it with:
```rust
/// Row height in points = score.metadata.row_height. Column width varies per row (justified).
```

- [ ] **Step 2: Update the stale comment in `unchanged_labels_do_not_repeat_after_line_wrap` test**

Find this comment block in the test:
```rust
// Use a narrow page so measures wrap across multiple row groups.
// With cell_size=24 and page_width=300: columns_per_page = 12.
// First measure: 2+2+16+1 = 21 > 12 → wraps before placing notes.
// After wrap the first measure is placed (still same time sig, same BPM).
// Second measure: same time sig, same BPM → no prefix labels.
// Total TimeSignatureLabel count across the whole score should be exactly 1.
```
Replace with:
```rust
// Wrapping is controlled by max_columns (default 28), not page width.
// First measure: 4 (directives) + 16 (notes) + 1 (bar) = 21 cols — fits in 28.
// Second measure: 0 + 16 + 1 = 17 cols — 21 + 17 = 38 > 28 → wraps after first measure.
// Same time sig and BPM on second measure → no repeat labels.
// Total TimeSignatureLabel count across the whole score should be exactly 1.
```

- [ ] **Step 3: Run layout tests**

```bash
cargo test --lib layout 2>&1 | tail -15
```
Expected: all layout tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/layout/mod.rs
git commit -m "feat: use row_height and max_columns in layout, remove page-width-based column calculation"
```

---

## Task 7: Update renderer — split `cell` into `row_height` and `column_width`

**Files:**
- Modify: `src/renderer.rs`

- [ ] **Step 1: Update the tests in `src/renderer.rs`**

The `render_score` helper passes `score.metadata.cell_size` — update it to `score.metadata.row_height`. Also update the font-size comment in `cjk_lyric_has_larger_font`:

Find and replace:
```rust
fn render_score(score_str: &str, lyrics_str: &str) -> Vec<String> {
    let input = format!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 {}\n\n[lyrics]\n{}\n",
        score_str, lyrics_str
    );
    let doc = parser::parse(&input, "test.jianpu").unwrap();
    let score = grouper::group(doc).unwrap();
    let pages = layout::layout(&score, A4_W, A4_H);
    render(&pages, score.metadata.cell_size)
}
```
With:
```rust
fn render_score(score_str: &str, lyrics_str: &str) -> Vec<String> {
    let input = format!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[score]\n4/4 {}\n\n[lyrics]\n{}\n",
        score_str, lyrics_str
    );
    let doc = parser::parse(&input, "test.jianpu").unwrap();
    let score = grouper::group(doc).unwrap();
    let pages = layout::layout(&score, A4_W, A4_H);
    render(&pages, score.metadata.row_height)
}
```

Find and replace the comment in `cjk_lyric_has_larger_font`:
```rust
// CJK font = base * 1.2, non-CJK = base
// With default cell_size=24: base = 24*0.6 = 14.4, cjk = 14.4*1.2 = 17.3
```
With:
```rust
// CJK font = base * 1.2, non-CJK = base
// With default row_height=24: base = 24*0.6 = 14.4, cjk = 14.4*1.2 = 17.3
```

Also update the two `render` call sites in tests that use `score.metadata.cell_size` directly (in the `bpm_label_renders_beats_per_minute_text` and `time_signature_label_renders_numerator_and_denominator_text` tests). Search for `score.metadata.cell_size` in the test block and replace each with `score.metadata.row_height`.

- [ ] **Step 2: Run renderer tests to verify they fail**

```bash
cargo test --lib renderer 2>&1 | tail -10
```
Expected: compile errors on `cell_size` field.

- [ ] **Step 3: Update the public `render` function signature**

Replace:
```rust
pub fn render(pages: &[Page], cell_size: u32) -> Vec<String> {
    pages.iter().map(|page| render_page(page, cell_size)).collect()
}
```
With:
```rust
pub fn render(pages: &[Page], row_height: u32) -> Vec<String> {
    pages.iter().map(|page| render_page(page, row_height)).collect()
}
```

- [ ] **Step 4: Update `render_page` — replace `cell` with `row_height` and `column_width`**

Replace the entire `render_page` function with:

```rust
fn render_page(page: &Page, row_height: u32) -> String {
    let row_height = row_height as f32;
    let base_font_size = row_height * 0.6;
    let cjk_font_size = base_font_size * 1.2;
    let page_width = page.page_width_pt;
    let page_height = 842.0_f32; // A4 height in points (matches SVG viewBox)
    let usable_width = page_width - 2.0 * PAGE_MARGIN;

    let mut elements = String::new();

    // --- Header ---
    let title_y = PAGE_MARGIN + row_height * 0.75;
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        page_width / 2.0,
        title_y,
        row_height * 1.5,
        escape_xml(&page.header.title)
    ));

    let subtitle_author_y = PAGE_MARGIN + row_height * 1.5;
    if let Some(subtitle) = &page.header.subtitle {
        elements.push_str(&format!(
            r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
            page_width / 2.0,
            subtitle_author_y,
            base_font_size,
            escape_xml(subtitle)
        ));
    }
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="end" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        page_width - PAGE_MARGIN,
        subtitle_author_y,
        base_font_size,
        escape_xml(&page.header.author)
    ));

    // --- Row groups ---
    for row_group in &page.row_groups {
        let column_width = usable_width / row_group.width_in_columns as f32;

        for element in row_group.elements.iter() {
            let col = element.position.column as f32;
            let row = element.position.row as f32;

            let base_x = col * column_width + PAGE_MARGIN;
            let base_y = PAGE_MARGIN + row * row_height;

            let x = match element.horizontal_alignment {
                HorizontalAlignment::Left => base_x,
                HorizontalAlignment::Center => base_x + column_width / 2.0,
                HorizontalAlignment::Right => base_x + column_width,
            };
            let y = match element.vertical_alignment {
                VerticalAlignment::Top => base_y,
                VerticalAlignment::Center => base_y + row_height / 2.0,
                VerticalAlignment::Bottom => base_y + row_height,
            };

            match &element.content {
                GridContent::NoteHead { pitch, octave } => {
                    let digit = pitch_to_digit(pitch);
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="monospace">{}</text>"#,
                        x, y, base_font_size, digit
                    ));
                    let dot_radius = row_height * 0.08;
                    let dot_spacing = dot_radius * 3.0;
                    for i in 0..*octave {
                        let dot_y = base_y - dot_radius - (i as f32) * dot_spacing;
                        elements.push_str(&format!(
                            r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="black"/>"#,
                            x, dot_y, dot_radius
                        ));
                    }
                }
                GridContent::Rest => {
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="monospace">0</text>"#,
                        x, y, base_font_size
                    ));
                }
                GridContent::DurationUnderlines { levels } => {
                    let _ = x;
                    for (i, span) in levels.iter().enumerate() {
                        let line_x1 = span.from_column as f32 * column_width + column_width * 0.1 + PAGE_MARGIN;
                        let line_x2 = span.to_column as f32 * column_width - column_width * 0.1 + PAGE_MARGIN;
                        let line_y = base_y + row_height * 0.1 + (i as f32) * (row_height * 0.15);
                        elements.push_str(&format!(
                            r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="black" stroke-width="1"/>"#,
                            line_x1, line_y, line_x2, line_y
                        ));
                    }
                }
                GridContent::LowerOctaveDots { count } => {
                    let dot_radius = row_height * 0.08;
                    let dot_spacing = dot_radius * 3.0;
                    for i in 0..*count {
                        let dot_y = base_y + dot_radius + (i as f32) * dot_spacing;
                        elements.push_str(&format!(
                            r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="black"/>"#,
                            x, dot_y, dot_radius
                        ));
                    }
                }
                GridContent::Lyric { text, is_cjk } => {
                    let font_size = if *is_cjk { cjk_font_size } else { base_font_size };
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="hanging" font-family="sans-serif">{}</text>"#,
                        x, y, font_size, escape_xml(text)
                    ));
                }
                GridContent::TieOrSlurCurve { from_column, to_column } => {
                    let _ = x;
                    let x1 = (*from_column as f32 + 0.5) * column_width + PAGE_MARGIN;
                    let x2 = (*to_column as f32 + 0.5) * column_width + PAGE_MARGIN;
                    let cy = base_y - row_height * 0.3;
                    elements.push_str(&format!(
                        r#"<path d="M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}" fill="none" stroke="black" stroke-width="1"/>"#,
                        x1, y, (x1 + x2) / 2.0, cy, x2, y
                    ));
                }
                GridContent::Extension => {
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="monospace">-</text>"#,
                        x, y, base_font_size
                    ));
                }
                GridContent::BarLine { height_in_rows } => {
                    let line_x = base_x;
                    let line_y1 = base_y;
                    let line_y2 = base_y + *height_in_rows as f32 * row_height;
                    elements.push_str(&format!(
                        r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="black" stroke-width="1.5"/>"#,
                        line_x, line_y1, line_x, line_y2
                    ));
                }
                GridContent::TimeSignatureLabel { numerator, denominator } => {
                    let slot_width = 2.0 * column_width;
                    let center_x = base_x + slot_width / 2.0;
                    let numerator_y = y - row_height * 0.25;
                    let rule_y = y;
                    let denominator_y = y + row_height * 0.25;
                    let rule_x1 = base_x + slot_width * 0.2;
                    let rule_x2 = base_x + slot_width * 0.8;
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
                        center_x, numerator_y, base_font_size, numerator
                    ));
                    elements.push_str(&format!(
                        r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="black" stroke-width="1"/>"#,
                        rule_x1, rule_y, rule_x2, rule_y
                    ));
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
                        center_x, denominator_y, base_font_size, denominator
                    ));
                }
                GridContent::BpmLabel { bpm } => {
                    let slot_width = 2.0 * column_width;
                    let center_x = base_x + slot_width / 2.0;
                    let small_font_size = base_font_size * 0.6;
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">♩={}</text>"#,
                        center_x, y, small_font_size, bpm
                    ));
                }
                GridContent::PartLabel { text } => {
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
                        x, y, base_font_size * 0.8, escape_xml(text)
                    ));
                }
            }
        }
    }

    // --- Footer ---
    let footer_y = page_height - PAGE_MARGIN - row_height * 0.5;
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif">{}/{}</text>"#,
        page_width / 2.0,
        footer_y,
        row_height * 0.75,
        page.footer.page,
        page.footer.total
    ));

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="210mm" height="297mm" viewBox="0 0 595 842">{}</svg>"#,
        elements
    )
}
```

- [ ] **Step 5: Run renderer tests**

```bash
cargo test --lib renderer 2>&1 | tail -15
```
Expected: all renderer tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/renderer.rs
git commit -m "feat: justify rows — compute column_width per row group, split from row_height"
```

---

## Task 8: Update callsites in `main.rs` and `pdf.rs`

**Files:**
- Modify: `src/main.rs`
- Modify: `src/pdf.rs`

- [ ] **Step 1: Update `src/main.rs`**

Replace:
```rust
let cell_size = score.metadata.cell_size;
let pages = layout::layout(&score, 595.0, 842.0);
let svgs = renderer::render(&pages, cell_size);
```
With:
```rust
let row_height = score.metadata.row_height;
let pages = layout::layout(&score, 595.0, 842.0);
let svgs = renderer::render(&pages, row_height);
```

- [ ] **Step 2: Update `src/pdf.rs`**

In the `make_pdf` test helper, replace:
```rust
let cell_size = score.metadata.cell_size;
let pages = layout::layout(&score, 595.0, 842.0);
let svgs = renderer::render(&pages, cell_size);
```
With:
```rust
let row_height = score.metadata.row_height;
let pages = layout::layout(&score, 595.0, 842.0);
let svgs = renderer::render(&pages, row_height);
```

- [ ] **Step 3: Run the full test suite**

```bash
cargo test 2>&1 | tail -20
```
Expected: all tests pass, zero compile errors.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs src/pdf.rs
git commit -m "chore: update main.rs and pdf.rs callsites to use row_height"
```

---

## Task 9: Update `demo.jianpu`

**Files:**
- Modify: `demo.jianpu`

- [ ] **Step 1: Replace `cell size` with `row height`**

In `demo.jianpu`, replace:
```
cell size = 20
```
With:
```
row height = 20
```

- [ ] **Step 2: Verify the demo still parses and renders**

```bash
cargo run -- demo.jianpu --svg 2>&1
```
Expected: no errors, SVG written.

- [ ] **Step 3: Commit**

```bash
git add demo.jianpu
git commit -m "chore: update demo.jianpu to use row height instead of cell size"
```

---

## Self-Review

**Spec coverage:**
- ✅ Remove `cell_size`, add `row_height` + `max_columns` — Tasks 2, 3, 4
- ✅ `nonempty` dependency — Task 1
- ✅ `RowGroup.elements: NonEmpty<GridElement>` — Task 5
- ✅ Layout uses `row_height` for vertical, `max_columns` for wrapping — Task 6
- ✅ Renderer computes `column_width` per row group, uses `row_height` for sizing — Task 7
- ✅ `margin_x = PAGE_MARGIN` (constant) — Task 7 (renderer sets `base_x = col * column_width + PAGE_MARGIN`)
- ✅ Callsites updated — Task 8
- ✅ `demo.jianpu` updated — Task 9

**Type consistency check:**
- `row_height` field name is consistent across all tasks
- `max_columns` field name is consistent
- `column_width` is a local variable in `render_page` — only used in Task 7, consistent
- `NonEmpty::from_vec` used at both push sites in Task 5 — consistent
- `render(pages, row_height)` signature matches usage in Tasks 7 and 8 — consistent
