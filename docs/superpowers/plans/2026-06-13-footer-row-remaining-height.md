# Footer Row Remaining Height Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the footer row expand to fill all remaining vertical space on each page, with the page-number text anchored to the bottom of that space.

**Architecture:** Two changes in `src/grid_layout/layout.rs`: `make_footer_row` accepts an explicit `height_pt` instead of computing it from `base`, and in `layout()` the remaining height (page height minus margins and body rows) is computed and passed in. `VAlign` on the footer element changes from `Center` to `Bottom`. The coordinate resolver already handles `VAlign::Bottom` correctly — no other files change.

**Tech Stack:** Rust, existing `grid_layout` types (`GridRow`, `GridElement`, `VAlign`)

---

### Task 1: Write failing tests

**Files:**
- Modify: `src/grid_layout/tests.rs`

- [ ] **Step 1: Add two failing tests at the bottom of `src/grid_layout/tests.rs`**

```rust
#[test]
fn footer_row_fills_remaining_page_height() {
    let blocks = vec![make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let page_height = 842.0_f32;
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, page_height);
    let page = &pages[0];
    let non_footer_height: f32 = page.rows[..page.rows.len() - 1]
        .iter()
        .map(|r| r.height_pt)
        .sum();
    let footer_height = page.rows.last().unwrap().height_pt;
    let expected = page_height - 2.0 * crate::grid_layout::PAGE_MARGIN - non_footer_height;
    assert!(
        (footer_height - expected).abs() < 0.001,
        "footer_height={footer_height} expected={expected}"
    );
}

#[test]
fn footer_element_valign_is_bottom() {
    let blocks = vec![make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0);
    let footer_row = pages[0].rows.last().unwrap();
    assert!(
        footer_row.elements.iter().all(|e| e.valign == VAlign::Bottom),
        "footer elements should be VAlign::Bottom"
    );
}
```

- [ ] **Step 2: Run the new tests and confirm they fail**

```bash
cargo test footer_row_fills_remaining_page_height footer_element_valign_is_bottom -- --test-thread=1 2>&1 | tail -20
```

Expected: both tests fail — `footer_row_fills_remaining_page_height` because the footer height is `base * 0.40` not the remaining space, and `footer_element_valign_is_bottom` because valign is currently `Center`.

---

### Task 2: Implement the changes

**Files:**
- Modify: `src/grid_layout/layout.rs`

- [ ] **Step 1: Change `make_footer_row` to accept explicit `height_pt`, set valign to `Bottom`, and delete `footer_row_height`**

Remove this function entirely:
```rust
pub(crate) fn footer_row_height(base: f32) -> f32 {
    base * 0.40
}
```

Change the signature and body of `make_footer_row` from:
```rust
fn make_footer_row(page_num: u32, total_pages: u32, base: f32) -> GridRow {
    GridRow {
        height_pt: footer_row_height(base),
        column_count: 1,
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: format!("{page_num} / {total_pages}"),
                font_size: base * 0.6,
                bold: false,
                italic: false,
            },
        }],
    }
}
```

To:
```rust
fn make_footer_row(page_num: u32, total_pages: u32, base: f32, height_pt: f32) -> GridRow {
    GridRow {
        height_pt,
        column_count: 1,
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Bottom,
            content: GridContent::Text {
                content: format!("{page_num} / {total_pages}"),
                font_size: base * 0.6,
                bold: false,
                italic: false,
            },
        }],
    }
}
```

- [ ] **Step 2: Update the `layout()` function to compute remaining height and pass it in**

In `layout()`, find the footer-building block inside the page loop. It currently reads:

```rust
let mut rows = build_page_rows(&page_sys, header, base, &arc_map, abs_system_index_start);
rows.push(make_footer_row(page_idx as u32 + 1, total_pages, base));
```

Replace it with:

```rust
let mut rows = build_page_rows(&page_sys, header, base, &arc_map, abs_system_index_start);
let body_height: f32 = rows.iter().map(|r| r.height_pt).sum();
let remaining_height = page_height_pt - 2.0 * super::PAGE_MARGIN - body_height;
rows.push(make_footer_row(page_idx as u32 + 1, total_pages, base, remaining_height));
```

Also find the line that uses `footer_row_height` for packing purposes:

```rust
let footer_h = footer_row_height(base);
```

Replace it with the inlined value (this is the minimum reservation during system packing — keep it as `base * 0.40`):

```rust
let footer_h = base * 0.40;
```

- [ ] **Step 3: Build to confirm no compile errors**

```bash
cargo build 2>&1 | tail -20
```

Expected: compiles cleanly with no errors or warnings about unused `footer_row_height`.

- [ ] **Step 4: Run the new tests and confirm they pass**

```bash
cargo test footer_row_fills_remaining_page_height footer_element_valign_is_bottom 2>&1 | tail -20
```

Expected: both tests pass.

- [ ] **Step 5: Run the full test suite**

```bash
cargo test 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/grid_layout/layout.rs src/grid_layout/tests.rs
git commit -m "feat(layout): footer row takes remaining page height, valign bottom"
```

---

## Self-Review

**Spec coverage:**
- ✅ Footer row height = remaining page space — Task 2 Step 2 computes `remaining_height` and passes it in.
- ✅ Content valign bottom — Task 2 Step 1 changes `VAlign::Center` → `VAlign::Bottom`.
- ✅ `footer_row_height()` deleted — Task 2 Step 1 removes it; the packing reservation is inlined.
- ✅ No changes to coordinate resolver — it already handles `VAlign::Bottom` at `resolve.rs:34`.

**Placeholders:** None.

**Type consistency:** `make_footer_row` signature in Task 2 Step 1 matches every call site updated in Task 2 Step 2. ✅
