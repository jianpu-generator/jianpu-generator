# Directive Row Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move time signature and BPM labels to a dedicated row above section labels / bar numbers, shown only when time or BPM changes on a system line.

**Architecture:** Add a per-system-line lookahead in `LayoutEngine` to detect whether any measure on the line emits time/BPM labels. When true, shift meta-row (bar number / section label) and part content down by one row, emit directives once on the new top row, and use variable `effective_row_group_height` for wrap, commit, and pagination.

**Tech Stack:** Rust, existing jianpu-generator layout pipeline (no new dependencies)

---

## File Map

| File | Change |
|------|--------|
| `src/layout/layout_engine.rs` | Lookahead, row-offset helpers, directive-row emission, variable row-group height, dynamic pagination |
| `src/layout/mod.rs` | Update existing layout tests; add new directive-row position tests |
| `syntax.md` | Brief rendering note under Directive lines |
| `src/renderer.rs` | No code changes expected (verify tests pass) |

---

### Task 1: Add failing tests for directive row placement

**Files:**
- Modify: `src/layout/mod.rs` (tests module)

- [ ] **Step 1: Add helper to collect time/BPM label elements**

Add near the other test helpers in `src/layout/mod.rs`:

```rust
fn collect_time_sig_labels(pages: &[Page]) -> Vec<&GridElement> {
    pages
        .iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .filter(|e| matches!(e.content, GridContent::TimeSignatureLabel { .. }))
        .collect()
}

fn collect_bpm_labels(pages: &[Page]) -> Vec<&GridElement> {
    pages
        .iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .filter(|e| matches!(e.content, GridContent::BpmLabel { .. }))
        .collect()
}
```

- [ ] **Step 2: Add failing test — time/BPM on directive row above meta row**

```rust
#[test]
fn time_and_bpm_labels_emit_on_directive_row_above_meta_row() {
    let score = make_score("1 2 3 4", "a b c d");
    let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
    let time_labels = collect_time_sig_labels(&pages);
    let bpm_labels = collect_bpm_labels(&pages);
    assert_eq!(time_labels.len(), 1);
    assert_eq!(bpm_labels.len(), 1);
    // header_rows = 2; directive row at +0, meta (bar number) at +1
    assert_eq!(
        time_labels[0].position.row,
        2,
        "time signature should be on directive row (header_rows)"
    );
    assert_eq!(
        bpm_labels[0].position.row,
        2,
        "BPM should be on directive row (header_rows)"
    );
    let bar_numbers: Vec<_> = pages[0]
        .row_groups[0]
        .elements
        .iter()
        .filter(|e| matches!(e.content, GridContent::BarNumber { .. }))
        .collect();
    assert_eq!(bar_numbers.len(), 1);
    assert_eq!(
        bar_numbers[0].position.row,
        3,
        "bar number should be on meta row (header_rows + 1)"
    );
}
```

- [ ] **Step 3: Add failing test — multi-part emits directives once**

Replace the body of `two_part_layout_emits_directives_on_both_parts_rows` (rename test to `two_part_layout_emits_directives_once_on_directive_row`):

```rust
#[test]
fn two_part_layout_emits_directives_once_on_directive_row() {
    let score = make_two_part_score("1 2 3 4", "5 6 7 1");
    let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
    let time_sig_labels = collect_time_sig_labels(&pages);
    let bpm_labels = collect_bpm_labels(&pages);
    assert_eq!(
        time_sig_labels.len(),
        1,
        "time signature label should appear once per change, not per notes part"
    );
    assert_eq!(
        bpm_labels.len(),
        1,
        "BPM label should appear once per change, not per notes part"
    );
}
```

- [ ] **Step 4: Add failing test — no directive row on continuation line**

```rust
#[test]
fn continuation_line_without_directive_changes_omits_directive_row() {
    // Wrap after first measure; second line has unchanged time/BPM → no directive row.
    let score = make_score("1 2 3 4 | 5 6 7 1", "a b c d e f g h");
    let pages = layout(&score, 300.0, A4_HEIGHT);
    let bar_numbers: Vec<_> = pages
        .iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .filter(|e| matches!(e.content, GridContent::BarNumber { .. }))
        .collect();
    assert_eq!(bar_numbers.len(), 2);
    // First line: directive row at 2, bar at 3. Second line: no directive row, bar at 6 (not 7).
    assert_eq!(bar_numbers[0].position.row, 2 + 1, "first line bar on meta row");
    assert_eq!(
        bar_numbers[1].position.row,
        6,
        "wrapped line without directive changes should not add extra row"
    );
}
```

- [ ] **Step 5: Add failing test — section label below directive row**

```rust
#[test]
fn section_label_renders_below_directive_row_when_both_present() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120 label=\"Verse 1\")\n1 2 3 4\n",
    );
    let pages = parse_and_layout(input);
    let all: Vec<_> = pages[0].row_groups[0].elements.iter().collect();
    let time_row = all
        .iter()
        .find(|e| matches!(e.content, GridContent::TimeSignatureLabel { .. }))
        .unwrap()
        .position
        .row;
    let label_row = all
        .iter()
        .find(|e| matches!(&e.content, GridContent::SectionLabel { text } if text == "Verse 1"))
        .unwrap()
        .position
        .row;
    assert!(label_row > time_row, "section label must be below directive row");
}
```

- [ ] **Step 6: Run tests to verify they fail**

```bash
cargo test time_and_bpm_labels_emit_on_directive_row_above_meta_row \
  two_part_layout_emits_directives_once_on_directive_row \
  continuation_line_without_directive_changes_omits_directive_row \
  section_label_renders_below_directive_row_when_both_present 2>&1 | tail -20
```

Expected: FAIL — bar numbers and time/BPM still on old rows; multi-part still emits twice.

- [ ] **Step 7: Commit**

```bash
git add src/layout/mod.rs
git commit -m "test: add failing tests for directive row layout"
```

---

### Task 2: Add lookahead and row-offset state to LayoutEngine

**Files:**
- Modify: `src/layout/layout_engine.rs`

- [ ] **Step 1: Add fields and helpers to `LayoutEngine`**

Add to the struct (after `bar_number`):

```rust
measure_index: usize,
line_has_directive_row: bool,
effective_row_group_height: u32,
current_page_rows_used: u32,
max_rows_per_page: u32,
```

In `new()`, initialize:

```rust
measure_index: 0,
line_has_directive_row: false,
effective_row_group_height: row_group_height,
current_page_rows_used: 0,
max_rows_per_page: {
    let usable_height = page_height_pt - 2.0 * PAGE_MARGIN;
    ((usable_height / row_height) as u32).saturating_sub(reserved_rows)
},
```

Remove `row_groups_per_page` field and its use in `new()` — replaced by `max_rows_per_page`.

Add free functions before `impl LayoutEngine`:

```rust
fn measure_has_directive_labels(measure: &MultiPartMeasure) -> bool {
    measure.time_signature.is_some() || measure.bpm.is_some()
}

fn line_has_any_directive_labels(
    measures: &[MultiPartMeasure],
    start_idx: usize,
    columns_per_row: u32,
    line_start_col: u32,
) -> bool {
    let mut col = line_start_col;
    for measure in &measures[start_idx..] {
        if measure_has_directive_labels(measure) {
            return true;
        }
        let prefix = compute_prefix_width(measure);
        let width = measure_column_width(measure);
        if col.saturating_add(prefix).saturating_add(width) > columns_per_row {
            break;
        }
        col = col.saturating_add(prefix).saturating_add(width);
    }
    false
}
```

Add methods on `LayoutEngine`:

```rust
fn refresh_line_row_state(&mut self) {
    if !self.is_line_start {
        return;
    }
    self.line_has_directive_row = line_has_any_directive_labels(
        &self.score.measures,
        self.measure_index,
        self.columns_per_row,
        self.label_cols,
    );
    self.effective_row_group_height = self.row_group_height
        + u32::from(self.line_has_directive_row);
}

fn meta_row(&self) -> u32 {
    self.current_row_offset + u32::from(self.line_has_directive_row)
}

fn part_row_base(&self) -> u32 {
    self.current_row_offset + 1 + u32::from(self.line_has_directive_row)
}

fn effective_bar_height(&self) -> u32 {
    self.effective_row_group_height - 1
}
```

- [ ] **Step 2: Change layout loop to track measure index and refresh line state**

Replace `layout()`:

```rust
pub(crate) fn layout(mut self) -> Vec<Page> {
    while self.measure_index < self.score.measures.len() {
        let measure = &self.score.measures[self.measure_index];
        self.refresh_line_row_state();
        self.wrap_line_if_needed(measure);
        self.emit_line_start_elements(measure);
        self.emit_section_label(measure);
        let note_col_start = self.emit_measure_directives(measure);
        self.emit_measure_content(measure, note_col_start);
        self.measure_index += 1;
    }
    self.finalize_pages()
}
```

- [ ] **Step 3: Compile check**

```bash
cargo build 2>&1 | tail -5
```

Expected: compiles (unused-method warnings OK for now).

---

### Task 3: Shift row offsets and emit directives on directive row

**Files:**
- Modify: `src/layout/layout_engine.rs`

- [ ] **Step 1: Update `emit_line_start_elements`**

Replace hard-coded `current_row_offset` / `current_row_offset + 1` with offset helpers:

```rust
fn emit_line_start_elements(&mut self, measure: &MultiPartMeasure) {
    if !self.is_line_start {
        return;
    }

    let part_base = self.part_row_base();
    let bar_h = self.effective_bar_height();

    self.current_elements.push(GridElement {
        position: GridPosition {
            column: self.label_cols,
            row: part_base,
        },
        horizontal_alignment: HorizontalAlignment::Center,
        vertical_alignment: VerticalAlignment::Center,
        content: GridContent::BarLine {
            height_in_rows: bar_h,
        },
    });
    if measure.label.is_none() {
        self.current_elements.push(GridElement {
            position: GridPosition {
                column: self.label_cols,
                row: self.meta_row(),
            },
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Bottom,
            content: GridContent::BarNumber {
                number: self.bar_number,
            },
        });
    }
    self.current_col = self.label_cols + 1;

    if self.has_named_parts {
        let mut row_cursor = self.part_row_base() - 1;
        for part_row in &measure.parts {
            if let Some(name) = part_row.name() {
                self.current_elements.push(GridElement {
                    position: GridPosition {
                        column: 0,
                        row: row_cursor + 1,
                    },
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Center,
                    content: GridContent::PartLabel { text: name.clone() },
                });
            }
            row_cursor += part_row_height(part_row);
        }
    }
    self.is_line_start = false;
}
```

- [ ] **Step 2: Update `emit_section_label`**

```rust
fn emit_section_label(&mut self, measure: &MultiPartMeasure) {
    if let Some(label_text) = &measure.label {
        self.current_elements.push(GridElement {
            position: GridPosition {
                column: self.current_col,
                row: self.meta_row(),
            },
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Bottom,
            content: GridContent::SectionLabel {
                text: label_text.clone(),
            },
        });
    }
}
```

- [ ] **Step 3: Rewrite `emit_measure_directives` — once per measure on directive row**

```rust
fn emit_measure_directives(&mut self, measure: &MultiPartMeasure) -> u32 {
    let directive_col_start = self.current_col;
    let mut directive_advance = 0u32;

    if !self.line_has_directive_row {
        return directive_col_start;
    }

    let directive_row = self.current_row_offset;
    let mut dc = directive_col_start;

    if let Some(ts) = &measure.time_signature {
        self.current_elements.push(GridElement {
            position: GridPosition {
                column: dc,
                row: directive_row,
            },
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Center,
            content: GridContent::TimeSignatureLabel {
                numerator: ts.numerator,
                denominator: ts.denominator,
            },
        });
        dc += 2;
        directive_advance += 2;
    }

    if let Some(bpm) = measure.bpm {
        self.current_elements.push(GridElement {
            position: GridPosition {
                column: dc,
                row: directive_row,
            },
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Center,
            content: GridContent::BpmLabel { bpm },
        });
        directive_advance += 2;
    }

    self.current_col = directive_col_start + directive_advance;
    self.current_col
}
```

- [ ] **Step 4: Update `emit_measure_content` to use `part_row_base()`**

Change `main_row_cursor` initialization and per-measure bar line row:

```rust
let mut main_row_cursor = self.part_row_base() - 1;
// ... existing loop unchanged ...

let bar_col = note_col_start + max_notes_width;
self.current_elements.push(GridElement {
    position: GridPosition {
        column: bar_col,
        row: self.part_row_base(),
    },
    // ... rest unchanged
});
```

- [ ] **Step 5: Run Task 1 tests**

```bash
cargo test time_and_bpm_labels_emit_on_directive_row_above_meta_row \
  two_part_layout_emits_directives_once_on_directive_row \
  section_label_renders_below_directive_row_when_both_present 2>&1 | tail -10
```

Expected: PASS for these three.

- [ ] **Step 6: Commit**

```bash
git add src/layout/layout_engine.rs
git commit -m "feat(layout): emit time/BPM on directive row above meta row"
```

---

### Task 4: Variable row-group height, wrap, and pagination

**Files:**
- Modify: `src/layout/layout_engine.rs`
- Modify: `src/layout/mod.rs` (update row-position assertions in existing tests)

- [ ] **Step 1: Update `push_bottom_system_bar`**

```rust
fn push_bottom_system_bar(&mut self) {
    self.current_elements.push(GridElement {
        position: GridPosition {
            column: 0,
            row: self.current_row_offset + self.effective_row_group_height,
        },
        horizontal_alignment: HorizontalAlignment::Left,
        vertical_alignment: VerticalAlignment::Top,
        content: GridContent::HorizontalBar {
            from_column: 0,
            to_column: self.current_col,
        },
    });
}
```

- [ ] **Step 2: Update `commit_row_group`**

```rust
fn commit_row_group(&mut self) {
    if let Some(elements) =
        nonempty::NonEmpty::from_vec(std::mem::take(&mut self.current_elements))
    {
        self.current_page_row_groups.push(RowGroup {
            elements,
            height_in_rows: self.effective_row_group_height,
            width_in_columns: self.current_col,
        });
        self.current_page_rows_used += self.effective_row_group_height;
    }
}
```

- [ ] **Step 3: Update `maybe_start_new_page`**

```rust
fn maybe_start_new_page(&mut self) {
    if self.current_page_row_groups.is_empty() {
        return;
    }
    if self.current_page_rows_used < self.max_rows_per_page {
        return;
    }
    self.pages.push(Page {
        header: self.make_header(),
        footer: Footer {
            page: self.pages.len() as u32 + 1,
            total: 0,
        },
        row_groups: std::mem::take(&mut self.current_page_row_groups),
        page_width_pt: self.page_width_pt,
    });
    self.current_row_offset = self.header_rows;
    self.current_page_rows_used = 0;
}
```

- [ ] **Step 4: Update `wrap_line_if_needed` row advance**

Replace `self.current_row_offset += self.row_group_height` with:

```rust
self.current_row_offset += self.effective_row_group_height;
self.is_line_start = true;
self.line_has_directive_row = false;
self.effective_row_group_height = self.row_group_height;
```

(Next iteration's `refresh_line_row_state` recomputes directive row for the new line.)

Also call `self.maybe_start_new_page()` after wrap (already present).

- [ ] **Step 5: Pre-check page break before starting a new row group**

At the top of `refresh_line_row_state`, after computing `effective_row_group_height`, add:

```rust
if self.is_line_start
    && !self.current_page_row_groups.is_empty()
    && self.current_page_rows_used + self.effective_row_group_height > self.max_rows_per_page
{
    self.pages.push(Page { /* same as maybe_start_new_page */ });
    self.current_row_offset = self.header_rows;
    self.current_page_rows_used = 0;
}
```

Extract shared page-flush logic into `fn flush_page(&mut self)` to avoid duplication.

- [ ] **Step 6: Update existing tests with shifted row numbers**

In `src/layout/mod.rs`, update these assertions:

| Test | Old row | New row (with directive row on first line) |
|------|---------|---------------------------------------------|
| `bar_number_emitted_at_start_of_each_row_group` — first bar | 2 | 3 |
| `bar_number_emitted_at_start_of_each_row_group` — second bar | 6 | 6 (wrapped line has no directive row) |
| `bar_number_emitted_on_first_row_group_even_without_wrap` | 2 | 3 |
| `section_label_element_emitted_at_correct_position` | (implicit +0) | assert `el.position.row == 3` when directives present |

Run full layout tests:

```bash
cargo test layout::tests 2>&1 | tail -15
```

Expected: all layout tests PASS including `continuation_line_without_directive_changes_omits_directive_row`.

- [ ] **Step 7: Commit**

```bash
git add src/layout/layout_engine.rs src/layout/mod.rs
git commit -m "feat(layout): variable row-group height for directive row"
```

---

### Task 5: Mid-line time change test and full suite

**Files:**
- Modify: `src/layout/mod.rs`

- [ ] **Step 1: Add mid-line time signature change test**

```rust
#[test]
fn mid_line_time_signature_change_uses_directive_row_for_whole_line() {
    let score = make_score_raw(
        "(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d\n\n(time=3/4)\n1 2 3\ne f g\n",
        "",
    );
    let pages = layout(&score, A4_WIDTH, A4_HEIGHT);
    let time_labels = collect_time_sig_labels(&pages);
    assert_eq!(time_labels.len(), 2);
    assert_eq!(
        time_labels[0].position.row, time_labels[1].position.row,
        "both time labels on same directive row"
    );
    assert_eq!(time_labels[0].position.column, 3);
    assert!(
        time_labels[1].position.column > time_labels[0].position.column,
        "second time label at later measure column"
    );
}
```

- [ ] **Step 2: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: all tests PASS (293+ unit tests, integration tests).

- [ ] **Step 3: Commit**

```bash
git add src/layout/mod.rs
git commit -m "test: cover mid-line time signature on directive row"
```

---

### Task 6: Update syntax.md

**Files:**
- Modify: `syntax.md`

- [ ] **Step 1: Add rendering note under Directive lines**

After the existing rules list in the `## Directive lines` section, add:

```markdown
### Rendering

When `time=` or `bpm=` changes on a measure, the generator may add a **directive row** above the bar-number / section-label row for that system line. Time signature and BPM appear once on that row (not on each part row). If neither value changes on any measure in the line, the directive row is omitted.
```

- [ ] **Step 2: Commit**

```bash
git add syntax.md
git commit -m "docs: describe directive row rendering in syntax.md"
```

---

## Self-Review (spec coverage)

| Spec requirement | Task |
|------------------|------|
| Directive row above meta row | Task 1–3 |
| Only when time/BPM change on line | Task 1, 4 (lookahead + continuation test) |
| Once per measure, not per part | Task 1, 3 |
| Mid-line changes | Task 5 |
| No repeat after wrap | Task 1 (`continuation_line_…`) |
| Section label below directive row | Task 1 |
| Variable row-group height | Task 4 |
| Dynamic pagination | Task 4 |
| syntax.md note | Task 6 |
| Renderer unchanged | Task 5 (full suite) |
