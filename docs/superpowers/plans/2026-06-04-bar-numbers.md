# Bar Numbers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Display a small sequential bar number above the left system bar at the start of every row group, counting from 1.

**Architecture:** Add a `BarNumber` variant to `GridContent`; layout emits it at each `is_line_start` using a counter that increments after every measure; the renderer draws it as small SVG text.

**Tech Stack:** Rust, SVG text rendering (no new dependencies)

---

### Task 1: Add `BarNumber` variant to `GridContent`

**Files:**
- Modify: `src/layout/types.rs:70-89`

- [ ] **Step 1: Write a failing compile-time test** — add this test to `src/layout/types.rs` to verify the variant compiles and pattern-matches:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_number_variant_exists() {
        let c = GridContent::BarNumber { number: 5 };
        match c {
            GridContent::BarNumber { number } => assert_eq!(number, 5),
            _ => panic!("unexpected variant"),
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test bar_number_variant_exists 2>&1 | head -20
```

Expected: compile error — `BarNumber` not found.

- [ ] **Step 3: Add the variant to `GridContent`**

In `src/layout/types.rs`, change line 88 (after `HorizontalBar`):

```rust
    HorizontalBar { from_column: u32, to_column: u32 },
    BarNumber { number: u32 },
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test bar_number_variant_exists 2>&1 | tail -5
```

Expected: `test layout::types::tests::bar_number_variant_exists ... ok`

- [ ] **Step 5: Fix exhaustive match in renderer** — the compiler will now warn about a non-exhaustive match in `src/renderer.rs`. Add a temporary stub arm at the end of the `match &element.content` block (around line 193, after the `HorizontalBar` arm):

```rust
                GridContent::BarNumber { .. } => {
                    // stub — rendered in Task 3
                }
```

- [ ] **Step 6: Verify the project compiles with no errors**

```bash
cargo build 2>&1 | tail -10
```

Expected: no errors (warnings about unused stub are fine).

- [ ] **Step 7: Commit**

```bash
git add src/layout/types.rs src/renderer.rs
git commit -m "feat: add BarNumber variant to GridContent"
```

---

### Task 2: Emit `BarNumber` elements in layout

**Files:**
- Modify: `src/layout/mod.rs`

- [ ] **Step 1: Write the failing test** — add this test inside the `#[cfg(test)]` block in `src/layout/mod.rs`:

```rust
#[test]
fn bar_number_emitted_at_start_of_each_row_group() {
    // First measure: 4 (directives) + 16 (notes) + 1 (bar) = 21 cols, fits in max_columns=28.
    // Second measure: 0 + 16 + 1 = 17 cols; 21+17=38 > 28 → wraps → two row groups.
    let score = make_score("1 2 3 4 5 6 7 1", "a b c d e f g h");
    let pages = layout(&score, A4_WIDTH, A4_HEIGHT);

    let bar_numbers: Vec<_> = pages.iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .filter(|e| matches!(e.content, GridContent::BarNumber { .. }))
        .collect();

    // One BarNumber per row group (2 row groups total)
    assert_eq!(bar_numbers.len(), 2, "expected one BarNumber per row group");

    // First row group: bar 1, at column 0 (label_cols=0), row = header_rows = 2
    if let GridContent::BarNumber { number } = bar_numbers[0].content {
        assert_eq!(number, 1, "first row group must start at bar 1");
    }
    assert_eq!(bar_numbers[0].position.column, 0);
    assert_eq!(bar_numbers[0].position.row, 2, "row = header_rows = 2");
    assert_eq!(bar_numbers[0].horizontal_alignment, HorizontalAlignment::Left);
    assert_eq!(bar_numbers[0].vertical_alignment, VerticalAlignment::Bottom);

    // Second row group: bar 2, at column 0, row = header_rows + row_group_height = 2 + 4 = 6
    if let GridContent::BarNumber { number } = bar_numbers[1].content {
        assert_eq!(number, 2, "second row group must start at bar 2");
    }
    assert_eq!(bar_numbers[1].position.column, 0);
    assert_eq!(bar_numbers[1].position.row, 6, "row = 2 + 4 = 6");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test bar_number_emitted_at_start_of_each_row_group 2>&1 | tail -10
```

Expected: FAIL — `expected one BarNumber per row group` (finds 0).

- [ ] **Step 3: Add the `bar_number` counter and emit `BarNumber` elements**

In `src/layout/mod.rs`, after line 121 (`let mut is_line_start = true;`), add:

```rust
    let mut bar_number: u32 = 1;
```

Then inside the `if is_line_start {` block (around lines 180–187), after emitting the left system `BarLine`, add the `BarNumber` emission:

```rust
        // Left system bar at start of each system line
        if is_line_start {
            current_elements.push(GridElement {
                position: GridPosition { column: label_cols, row: current_row_offset + 1 },
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Center,
                content: GridContent::BarLine { height_in_rows: bar_height },
            });
            current_elements.push(GridElement {
                position: GridPosition { column: label_cols, row: current_row_offset },
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Bottom,
                content: GridContent::BarNumber { number: bar_number },
            });
        }
```

Then after the bar line is pushed (around line 372, after `current_col = bar_col + 1;`), add:

```rust
        bar_number += 1;
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test bar_number_emitted_at_start_of_each_row_group 2>&1 | tail -5
```

Expected: `test layout::tests::bar_number_emitted_at_start_of_each_row_group ... ok`

- [ ] **Step 5: Run all layout tests to check for regressions**

```bash
cargo test --lib layout 2>&1 | tail -15
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/layout/mod.rs
git commit -m "feat: emit BarNumber element at start of each row group"
```

---

### Task 3: Render `BarNumber` as SVG text

**Files:**
- Modify: `src/renderer.rs`

- [ ] **Step 1: Write the failing test** — add this test inside the `#[cfg(test)]` block in `src/renderer.rs`:

```rust
#[test]
fn bar_number_renders_as_small_text_above_left_bar() {
    // Two measures force a row wrap → two row groups → two bar numbers in SVG.
    // row_height=24, base_font_size=24*0.6=14.4, bar_number_font=14.4*0.6=8.6 (rounded to 1dp)
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "[score]\n4/4 1 2 3 4 5 6 7 1\n\n",
        "[lyrics]\na b c d e f g h\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(&pages, score.metadata.row_height);
    let svg = &svgs[0];
    // Bar 1 appears in the SVG
    assert!(svg.contains(">1<") || svg.contains(">1 <"),
        "expected bar number 1 in SVG output");
    // Bar 2 appears in the SVG
    assert!(svg.contains(">2<") || svg.contains(">2 <"),
        "expected bar number 2 in SVG output");
    // Small font size 8.6 is used for bar numbers
    assert!(svg.contains("font-size=\"8.6\""),
        "expected bar number font-size 8.6 in SVG; snippet: {}", &svg[..svg.len().min(800)]);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test bar_number_renders_as_small_text_above_left_bar 2>&1 | tail -10
```

Expected: FAIL — font-size 8.6 not found (stub arm emits nothing).

- [ ] **Step 3: Replace the stub `BarNumber` arm in `src/renderer.rs`**

Replace the stub from Task 1 Step 5:

```rust
                GridContent::BarNumber { .. } => {
                    // stub — rendered in Task 3
                }
```

With the real implementation:

```rust
                GridContent::BarNumber { number } => {
                    elements.push_str(&format!(
                        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="auto" font-family="sans-serif">{}</text>"#,
                        x, y, base_font_size * 0.6, number
                    ));
                }
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test bar_number_renders_as_small_text_above_left_bar 2>&1 | tail -5
```

Expected: `test renderer::tests::bar_number_renders_as_small_text_above_left_bar ... ok`

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1 | tail -15
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/renderer.rs
git commit -m "feat: render BarNumber as small SVG text above left system bar"
```
