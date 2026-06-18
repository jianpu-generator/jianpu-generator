# Bar Line Lyric Height Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make bar lines in each system extend through lyric rows so they cover the full system height.

**Architecture:** `compute_bar_height` in `src/grid_layout/layout.rs` currently sums only musical sub-row heights. Adding `system_lyric_height_pt` to that sum makes every `BarLine { height_pt }` element (left-edge bar and measure bars) span the full system.

**Tech Stack:** Rust, `cargo test`

---

### Task 1: Write failing test, then fix `compute_bar_height`

**Files:**
- Modify: `src/grid_layout/tests.rs` (add test near line 182 where `bar_line_element_has_positive_height_pt` lives)
- Modify: `src/grid_layout/layout.rs:178-191`

- [ ] **Step 1: Add the failing test to `src/grid_layout/tests.rs`**

Add these imports to the existing `use crate::grid_layout::layout::{...}` block at line 66:

```rust
use crate::grid_layout::layout::{
    chord_part_sub_row_heights, expand_system_to_rows, is_chord_only_row, is_lyric_row, layout,
    lyric_row_height, note_part_sub_row_heights, pack_into_systems, system_lyric_height_pt,
    system_musical_height_pt,
};
```

Then add the helper and test after the existing `bar_line_element_has_positive_height_pt` test (after line 192):

```rust
fn make_block_with_lyric_part(bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![
            MeasureRow {
                id: RowId("note".to_string()),
                label: "note".to_string(),
                elements: vec![
                    ColumnElement {
                        column: 0,
                        content: ElementContent::NoteHead {
                            pitch: JianPuPitch::One,
                            octave: 0,
                            dotted: false,
                        },
                    },
                    ColumnElement {
                        column: bar_col,
                        content: ElementContent::BarLine,
                    },
                ],
            },
            MeasureRow {
                id: RowId("lyric".to_string()),
                label: "lyric".to_string(),
                elements: vec![ColumnElement {
                    column: 0,
                    content: ElementContent::Lyric("la".to_string()),
                }],
            },
        ],
        decorations: vec![],
    }
}

#[test]
fn bar_line_height_includes_lyric_rows() {
    let base = 30.0_f32;
    let system = vec![make_block_with_lyric_part(3)];
    let first = system.first().unwrap();
    let expected_height =
        system_musical_height_pt(first, base) + system_lyric_height_pt(first, base);

    let rows = expand_system_to_rows(&system, base, &HashMap::new());
    let bar = rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::BarLine { .. }))
        .expect("should have a BarLine element");
    if let GridContent::BarLine { height_pt } = bar.content {
        assert!(
            (height_pt - expected_height).abs() < 0.001,
            "bar height={height_pt}, expected={expected_height} (musical + lyric)"
        );
    }
}
```

- [ ] **Step 2: Verify the test fails**

```bash
cargo test bar_line_height_includes_lyric_rows 2>&1 | tail -20
```

Expected: test fails because `height_pt` equals only the musical height (63.0 for base=30), not musical + lyric (63.0 + 45.0 = 108.0).

- [ ] **Step 3: Also make `system_musical_height_pt` and `system_lyric_height_pt` pub(crate) if not already**

Check `src/grid_layout/layout.rs` lines 117 and 133 — both are already `pub(crate)`. No change needed. Also confirm `lyric_row_height` (line 75) is `pub(crate)`. If any are missing `pub(crate)`, add it.

- [ ] **Step 4: Fix `compute_bar_height` in `src/grid_layout/layout.rs`**

Replace lines 178-191:

```rust
fn compute_bar_height(first: &MeasureBlock, base: f32) -> f32 {
    system_musical_height_pt(first, base) + system_lyric_height_pt(first, base)
}
```

- [ ] **Step 5: Verify the new test passes**

```bash
cargo test bar_line_height_includes_lyric_rows 2>&1 | tail -10
```

Expected: `test bar_line_height_includes_lyric_rows ... ok`

- [ ] **Step 6: Run full test suite**

```bash
cargo test 2>&1 | tail -20
```

Expected: all tests pass, no regressions.

- [ ] **Step 7: Commit**

```bash
git checkout -b fix/bar-line-lyric-height
git add src/grid_layout/tests.rs src/grid_layout/layout.rs
git commit -m "fix(grid_layout): bar line height includes lyric rows"
```
