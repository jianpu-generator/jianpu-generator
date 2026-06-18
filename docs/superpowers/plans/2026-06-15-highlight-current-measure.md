# Highlight Current Measure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When the editor caret sits inside a measure, render a semi-transparent amber rectangle behind that measure in the SVG preview — baked in by the Rust/WASM pipeline, no frontend DOM injection.

**Architecture:** `highlighted_measure_index: Option<usize>` is threaded through `grid_layout::layout` → `coordinate_resolver::resolve` → `renderer::render_new` → `serializer::serialize`. Grid-layout computes the measure's bounding box in grid-space (`MeasureHighlight`); the coordinate resolver converts it to an `AbsoluteContent::MeasureHighlight` element prepended to the page's element list (renders behind all content); the renderer and serializer handle it as a new `SvgKind::Rect`. A new WASM export `render_with_highlight` and matching worker/hook wiring complete the loop.

**Tech Stack:** Rust (grid_layout, compositor, renderer, serializer, lib.rs), wasm-bindgen (`crates/jianpu-wasm`), TypeScript/React (worker, useJianpuWorker hook, Preview component)

---

## File Map

| Action | File |
|--------|------|
| Modify | `src/grid_layout/types.rs` — add `MeasureHighlight` struct and field on `GridPage` |
| Modify | `src/grid_layout/layout.rs` — add `highlighted_measure_index` param + compute highlight |
| Create | `src/grid_layout/tests_highlight.rs` — unit tests for highlight location logic |
| Modify | `src/compositor/types.rs` — add `AbsoluteContent::MeasureHighlight` variant |
| Modify | `src/coordinate_resolver/resolve.rs` — convert grid highlight → absolute element |
| Modify | `src/coordinate_resolver/tests.rs` — test highlight resolution |
| Modify | `src/renderer/new_types.rs` — add `SvgKind::Rect` variant |
| Modify | `src/renderer/new_renderer.rs` — handle `AbsoluteContent::MeasureHighlight` |
| Modify | `src/serializer/mod.rs` — serialize `SvgKind::Rect` + test |
| Modify | `src/lib.rs` — add `render_svgs_with_highlight` public function |
| Modify | `crates/jianpu-wasm/src/lib.rs` — add `render_with_highlight` WASM export |
| Modify | `web/src/worker/jianpu.worker.ts` — new request/response types + handler |
| Modify | `web/src/hooks/useJianpuWorker.ts` — `highlightedSvgs` state + effect |
| Modify | `web/src/components/Preview.tsx` — accept + use `highlightedSvgs` |
| Modify | `web/src/App.tsx` — pass `highlightedSvgs` to Preview |

---

### Task 1: Add `MeasureHighlight` to grid_layout types

**Files:**
- Modify: `src/grid_layout/types.rs`

- [ ] **Step 1: Add `MeasureHighlight` struct and field to `GridPage`**

In `src/grid_layout/types.rs`, append after the existing `Header` struct:

```rust
#[derive(Debug, Clone)]
pub struct MeasureHighlight {
    pub row_start: usize, // first row index on this page (inclusive)
    pub row_end: usize,   // last row index on this page (inclusive)
    pub column_start: u32, // first column of the measure (after label columns)
    pub column_end: u32,   // last column (exclusive) of the measure
}
```

Change `GridPage` to:

```rust
#[derive(Debug, Clone)]
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
    pub measure_highlight: Option<MeasureHighlight>,
}
```

- [ ] **Step 2: Fix all `GridPage` construction sites to include the new field**

The only non-test constructor is in `src/grid_layout/layout.rs` (the `layout` function itself, which Task 2 will update). Grep to confirm no other construction sites exist:

```bash
grep -rn "GridPage {" src/ crates/
```

Expected output: matches only in `src/grid_layout/layout.rs` and any test files. Add `measure_highlight: None` to each struct literal found. (Task 2 will set the real value in `layout`.)

- [ ] **Step 3: Verify it compiles**

```bash
cargo check
```

Expected: no errors (field missing errors if any construction sites were missed will surface here).

---

### Task 2: Compute the highlight in `grid_layout::layout`

**Files:**
- Modify: `src/grid_layout/layout.rs`
- Create: `src/grid_layout/tests_highlight.rs`

Background: `layout` builds `page_systems: Vec<Vec<Vec<MeasureBlock>>>` (pages → systems → blocks). Each block's global index in `compile_result.blocks` is its measure index. We need to find which page/row-range/column-range a given measure index occupies.

- [ ] **Step 1: Write failing test for `compute_measure_highlight_location`**

Create `src/grid_layout/tests_highlight.rs`:

```rust
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::layout::compute_measure_highlight_location;
use crate::grid_layout::types::Header;

fn simple_block(col_count: u32) -> MeasureBlock {
    // col_count is the number of note columns; block_column_width = col_count + 1 (bar line)
    let elements: Vec<ColumnElement> = (0..col_count)
        .map(|c| ColumnElement {
            column: c,
            content: ElementContent::NoteHead {
                pitch: crate::ast::parsed::JianPuPitch::One,
                octave: 0,
                dotted: false,
            },
        })
        .chain(std::iter::once(ColumnElement {
            column: col_count,
            content: ElementContent::BarLine,
        }))
        .collect();
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId("S".to_string()),
            label: String::new(),
            elements,
        }],
        decorations: vec![],
    }
}

fn no_header() -> Header {
    Header {
        title: String::new(),
        subtitle: None,
        author: String::new(),
    }
}

#[test]
fn returns_none_for_out_of_range_measure_index() {
    // 1 page, 1 system, 2 blocks → measure indices 0 and 1
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 2, &no_header(), 20.0);
    assert!(result.is_none());
}

#[test]
fn first_block_in_single_system_has_correct_column_range() {
    // LABEL_COLS = 4, block_column_width(4-note block) = 5
    // measure 0 → column_start = 4, column_end = 9
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 0, &no_header(), 20.0)
        .expect("should find measure 0");
    let (_page_idx, highlight) = result;
    assert_eq!(highlight.column_start, 4, "column_start should be LABEL_COLS");
    assert_eq!(highlight.column_end, 9, "column_end = LABEL_COLS + block_col_width");
}

#[test]
fn second_block_column_start_follows_first_block_width() {
    // measure 1 → column_start = 4 + 5 = 9, column_end = 14
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 1, &no_header(), 20.0)
        .expect("should find measure 1");
    let (_page_idx, highlight) = result;
    assert_eq!(highlight.column_start, 9);
    assert_eq!(highlight.column_end, 14);
}

#[test]
fn measure_on_second_page_returns_correct_page_index() {
    // page 0: system with measure 0
    // page 1: system with measure 1
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> = vec![
        vec![vec![simple_block(4)]],
        vec![vec![simple_block(4)]],
    ];
    let result = compute_measure_highlight_location(&page_systems, 1, &no_header(), 20.0)
        .expect("should find measure 1");
    let (page_idx, _) = result;
    assert_eq!(page_idx, 1, "measure 1 is on page 1");
}
```

- [ ] **Step 2: Run test to confirm it fails**

```bash
cargo test compute_measure_highlight_location 2>&1 | head -30
```

Expected: compilation error (`compute_measure_highlight_location` does not exist yet).

- [ ] **Step 3: Register the test module in `layout.rs`**

At the bottom of `src/grid_layout/layout.rs`, add:

```rust
#[cfg(test)]
#[path = "tests_highlight.rs"]
mod tests_highlight;
```

- [ ] **Step 4: Add `system_musical_row_count` helper to `layout.rs`**

Add this function before `layout` in `src/grid_layout/layout.rs`:

```rust
fn system_musical_row_count(system: &[MeasureBlock], base: f32) -> usize {
    let Some(first) = system.first() else {
        return 0;
    };
    first
        .rows
        .iter()
        .map(|part_template| {
            if is_lyric_row(part_template) {
                1
            } else {
                let sub_count = if is_chord_only_row(part_template) { 4 } else { 6 };
                sub_count + if has_lyrics(part_template) { 1 } else { 0 }
            }
        })
        .sum()
}
```

- [ ] **Step 5: Add `compute_measure_highlight_location` to `layout.rs`**

Add this function (make it `pub(crate)` so the test module can call it) before `layout` in `src/grid_layout/layout.rs`:

```rust
pub(crate) fn compute_measure_highlight_location(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    highlighted_measure_index: usize,
    header: &Header,
    base: f32,
) -> Option<(usize, crate::grid_layout::types::MeasureHighlight)> {
    use crate::grid_layout::types::MeasureHighlight;
    let header_row_count = make_header_rows(header, base).len();
    let mut global_measure_index: usize = 0;

    for (page_idx, page_sys) in page_systems.iter().enumerate() {
        let mut row_offset = header_row_count;
        for (sys_idx, system) in page_sys.iter().enumerate() {
            if sys_idx > 0 {
                row_offset += 1; // separator row
            }
            let first = system.first()?;
            if has_any_decoration(first) {
                row_offset += 1; // decoration row
            }
            let musical_row_count = system_musical_row_count(system, base);
            let row_start = row_offset;
            let row_end = row_offset + musical_row_count.saturating_sub(1);

            let mut col_offset: u32 = LABEL_COLS;
            for block in system {
                let col_w = block_column_width(block);
                if global_measure_index == highlighted_measure_index {
                    return Some((
                        page_idx,
                        MeasureHighlight {
                            row_start,
                            row_end,
                            column_start: col_offset,
                            column_end: col_offset + col_w,
                        },
                    ));
                }
                col_offset += col_w;
                global_measure_index += 1;
            }
            row_offset += musical_row_count;
        }
    }
    None
}
```

- [ ] **Step 6: Run tests to confirm they pass**

```bash
cargo test tests_highlight 2>&1 | tail -20
```

Expected: all 4 tests pass.

- [ ] **Step 7: Update `layout` to accept `highlighted_measure_index` and set `measure_highlight`**

Change the signature of `layout` in `src/grid_layout/layout.rs`:

```rust
pub fn layout(
    compile_result: &CompileResult,
    config: &RenderConfig,
    header: &Header,
    page_width_pt: f32,
    page_height_pt: f32,
    highlighted_measure_index: Option<usize>,
) -> Vec<GridPage> {
```

Inside `layout`, after `page_systems.push(current_page);` (line ~564) and before building the `pages` vec, add:

```rust
    let highlight_info: Option<(usize, crate::grid_layout::types::MeasureHighlight)> =
        highlighted_measure_index.and_then(|idx| {
            compute_measure_highlight_location(&page_systems, idx, header, base)
        });
```

In the existing page-building loop, change the `GridPage { ... }` construction from:

```rust
        pages.push(GridPage {
            width_pt: page_width_pt,
            height_pt: page_height_pt,
            rows,
        });
```

to:

```rust
        let measure_highlight = highlight_info
            .as_ref()
            .and_then(|(h_page, h)| {
                if *h_page == page_idx {
                    Some(h.clone())
                } else {
                    None
                }
            });
        pages.push(GridPage {
            width_pt: page_width_pt,
            height_pt: page_height_pt,
            rows,
            measure_highlight,
        });
```

- [ ] **Step 8: Fix the caller of `layout` in `src/lib.rs`**

In `src/lib.rs`, find `render_svgs`:

```rust
pub fn render_svgs(score: &Score) -> Vec<String> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header { ... };
    let compile_result = compiler::compile(score);
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0);
    ...
}
```

Change the `layout` call to pass `None`:

```rust
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
```

- [ ] **Step 9: Verify everything compiles and existing tests pass**

```bash
cargo test 2>&1 | tail -30
```

Expected: all tests pass, no compilation errors.

- [ ] **Step 10: Commit**

```bash
git add src/grid_layout/types.rs src/grid_layout/layout.rs src/grid_layout/tests_highlight.rs src/lib.rs
git commit -m "feat: add MeasureHighlight to GridPage and compute it in layout"
```

---

### Task 3: Add `AbsoluteContent::MeasureHighlight` and update coordinate resolver

**Files:**
- Modify: `src/compositor/types.rs`
- Modify: `src/coordinate_resolver/resolve.rs`
- Modify: `src/coordinate_resolver/tests.rs`

- [ ] **Step 1: Write failing test**

Append to `src/coordinate_resolver/tests.rs`:

```rust
#[test]
fn measure_highlight_produces_prepended_rect_element() {
    use crate::grid_layout::types::MeasureHighlight;

    // page: 595pt wide, PAGE_MARGIN=25, usable=545
    // row 0: height=30, col_count=10 → col_width = 545/10 = 54.5
    // row 1: height=20, col_count=10
    // highlight: row_start=0, row_end=1, column_start=4, column_end=6
    // expected rect: x = 25 + 4*54.5 = 243.0, y = 25.0 (PAGE_MARGIN)
    // width = (6-4)*54.5 = 109.0, height = 30.0 + 20.0 = 50.0
    let page = crate::grid_layout::types::GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![
            GridRow {
                height_pt: 30.0,
                column_count: 10,
                elements: vec![],
            },
            GridRow {
                height_pt: 20.0,
                column_count: 10,
                elements: vec![],
            },
        ],
        measure_highlight: Some(MeasureHighlight {
            row_start: 0,
            row_end: 1,
            column_start: 4,
            column_end: 6,
        }),
    };
    let abs = resolve(&[page], 12.0);
    assert!(!abs[0].elements.is_empty(), "should have elements");
    let first = &abs[0].elements[0];
    assert!(
        matches!(first.content, AbsoluteContent::MeasureHighlight { .. }),
        "first element should be MeasureHighlight, got {:?}",
        first.content
    );
    if let AbsoluteContent::MeasureHighlight { width, height } = first.content {
        assert!((width - 109.0).abs() < 0.1, "width={width}");
        assert!((height - 50.0).abs() < 0.1, "height={height}");
    }
    assert!((first.x - 243.0).abs() < 0.1, "x={}", first.x);
    assert!((first.y - 25.0).abs() < 0.1, "y={}", first.y);
}

#[test]
fn page_with_no_highlight_produces_no_extra_element() {
    let page = crate::grid_layout::types::GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 10,
            elements: vec![],
        }],
        measure_highlight: None,
    };
    let abs = resolve(&[page], 12.0);
    assert!(abs[0].elements.is_empty());
}
```

- [ ] **Step 2: Run test to confirm failure**

```bash
cargo test measure_highlight_produces_prepended_rect_element 2>&1 | head -20
```

Expected: compilation error — `AbsoluteContent::MeasureHighlight` does not exist yet.

- [ ] **Step 3: Add `MeasureHighlight` variant to `AbsoluteContent`**

In `src/compositor/types.rs`, add to the `AbsoluteContent` enum after the `Text` variant:

```rust
    MeasureHighlight { width: f32, height: f32 },
```

- [ ] **Step 4: Update `resolve_page` to convert `GridPage.measure_highlight`**

In `src/coordinate_resolver/resolve.rs`, replace the `resolve_page` function with:

```rust
fn resolve_page(page: &GridPage, note_number_width: f32) -> AbsolutePage {
    use crate::grid_layout::PAGE_MARGIN;
    let usable_width = page.width_pt - 2.0 * PAGE_MARGIN;
    let mut elements: Vec<AbsoluteElement> = Vec::new();
    let mut row_y = PAGE_MARGIN;

    // Collect per-row y positions so we can compute highlight bounds after the loop.
    let mut row_tops: Vec<f32> = Vec::with_capacity(page.rows.len());

    for row in &page.rows {
        row_tops.push(row_y);
        let col_width = row.column_width_pt(usable_width);
        for el in &row.elements {
            let x_start = PAGE_MARGIN + el.column as f32 * col_width;
            let span_width = el.column_span as f32 * col_width;
            let x = match el.halign {
                HAlign::Start => x_start,
                HAlign::Center => x_start + span_width * 0.5,
                HAlign::End => x_start + span_width,
            };
            let y = match el.valign {
                VAlign::Top => row_y,
                VAlign::Center => row_y + row.height_pt * 0.5,
                VAlign::Bottom => row_y + row.height_pt,
            };
            if let GridContent::Underline { level } = &el.content {
                let note_center_x = x_start + col_width * 0.5;
                let ul_x = note_center_x - note_number_width * 0.5;
                let ul_width = (el.column_span as f32 - 1.0) * col_width + note_number_width;
                elements.push(AbsoluteElement {
                    x: ul_x,
                    y,
                    content: AbsoluteContent::Underline {
                        width: ul_width,
                        level: *level,
                    },
                });
                continue;
            }
            if matches!(
                el.content,
                GridContent::TieOrSlur | GridContent::TieOrSlurTail | GridContent::TieOrSlurHead
            ) {
                let arc_x = match &el.content {
                    GridContent::TieOrSlur | GridContent::TieOrSlurTail => {
                        x_start + col_width * 0.5
                    }
                    GridContent::TieOrSlurHead => x_start,
                    _ => unreachable!(),
                };
                let arc_width = match &el.content {
                    GridContent::TieOrSlur => (el.column_span as f32 - 1.0) * col_width,
                    GridContent::TieOrSlurTail => {
                        el.column_span as f32 * col_width - col_width * 0.5
                    }
                    GridContent::TieOrSlurHead => {
                        (el.column_span as f32 - 1.0) * col_width + col_width * 0.5
                    }
                    _ => unreachable!(),
                };
                elements.push(AbsoluteElement {
                    x: arc_x,
                    y,
                    content: AbsoluteContent::TieOrSlur { width: arc_width },
                });
                continue;
            }
            if let Some(content) = grid_to_absolute(&el.content, span_width, el.halign) {
                elements.push(AbsoluteElement { x, y, content });
            }
        }
        row_y += row.height_pt;
    }

    // Prepend highlight rect if present so it renders behind all note content.
    if let Some(ref h) = page.measure_highlight {
        if h.row_start < page.rows.len() && h.row_end < page.rows.len() {
            let highlight_y = row_tops[h.row_start];
            let highlight_height: f32 = page.rows[h.row_start..=h.row_end]
                .iter()
                .map(|r| r.height_pt)
                .sum();
            // Use the column_count of the first highlighted row for col_width.
            let row = &page.rows[h.row_start];
            let col_width = row.column_width_pt(usable_width);
            let highlight_x = PAGE_MARGIN + h.column_start as f32 * col_width;
            let highlight_width = (h.column_end - h.column_start) as f32 * col_width;
            elements.insert(
                0,
                AbsoluteElement {
                    x: highlight_x,
                    y: highlight_y,
                    content: AbsoluteContent::MeasureHighlight {
                        width: highlight_width,
                        height: highlight_height,
                    },
                },
            );
        }
    }

    AbsolutePage {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements,
    }
}
```

- [ ] **Step 5: Run tests**

```bash
cargo test coordinate_resolver 2>&1 | tail -20
```

Expected: all tests including the two new ones pass.

- [ ] **Step 6: Handle new variant in renderer (prevents compile error from exhaustive match)**

The renderer's `render_element` will fail to compile when it encounters the new `AbsoluteContent::MeasureHighlight` variant. Jump ahead to add a stub arm now so the full test suite runs:

In `src/renderer/new_renderer.rs`, add to the match in `render_element`:

```rust
        AbsoluteContent::MeasureHighlight { width, height } => {
            vec![SvgElement {
                x: elem.x,
                y: elem.y,
                variant: "measure-highlight",
                kind: SvgKind::Rect { width: *width, height: *height },
            }]
        }
```

This will fail to compile until Task 4 adds `SvgKind::Rect`. Add the variant now.

In `src/renderer/new_types.rs`, add to `SvgKind`:

```rust
    Rect { width: f32, height: f32 },
```

Then add the stub serializer arm in `src/serializer/mod.rs` inside `serialize_element`:

```rust
        SvgKind::Rect { width, height } => {
            out.push_str(&format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="rgba(255,200,0,0.25)" rx="2"/>"#,
                el.x, el.y, width, height
            ));
        }
```

- [ ] **Step 7: Verify full test suite passes**

```bash
cargo test 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/compositor/types.rs src/coordinate_resolver/resolve.rs src/coordinate_resolver/tests.rs src/renderer/new_types.rs src/renderer/new_renderer.rs src/serializer/mod.rs
git commit -m "feat: add MeasureHighlight through coordinate_resolver, renderer, and serializer"
```

---

### Task 4: Add serializer test for `SvgKind::Rect`

**Files:**
- Modify: `src/serializer/mod.rs`

- [ ] **Step 1: Write test for rect serialization**

In `src/serializer/mod.rs`, add inside the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn rect_serializes_with_amber_fill() {
        let doc = SvgDocument {
            width_pt: 100.0,
            height_pt: 100.0,
            elements: vec![SvgElement {
                x: 10.0,
                y: 20.0,
                variant: "measure-highlight",
                kind: SvgKind::Rect {
                    width: 50.0,
                    height: 30.0,
                },
            }],
        };
        let result = serialize(&[doc]);
        assert!(result[0].contains("<rect"), "should contain rect");
        assert!(result[0].contains(r#"x="10.0""#), "should have x");
        assert!(result[0].contains(r#"y="20.0""#), "should have y");
        assert!(result[0].contains(r#"width="50.0""#), "should have width");
        assert!(result[0].contains(r#"height="30.0""#), "should have height");
        assert!(
            result[0].contains("rgba(255,200,0,0.25)"),
            "should have amber fill"
        );
        assert!(result[0].contains(r#"rx="2""#), "should have corner radius");
    }
```

- [ ] **Step 2: Run the test**

```bash
cargo test rect_serializes_with_amber_fill 2>&1 | tail -10
```

Expected: PASS (the implementation was added in Task 3 Step 6).

- [ ] **Step 3: Commit**

```bash
git add src/serializer/mod.rs
git commit -m "test: add serializer test for rect/amber-fill element"
```

---

### Task 5: Add `render_svgs_with_highlight` to `src/lib.rs`

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Add `render_svgs_with_highlight` function**

In `src/lib.rs`, add after `render_svgs_from_source_filtered_with_lyrics`:

```rust
/// Parse, group, optionally filter tracks and lyrics, and render SVG page strings
/// with an optional measure highlight baked into the SVG.
///
/// `highlighted_measure_index` is the 0-based measure index to highlight.
/// When `None`, output is identical to `render_svgs_from_source_filtered_with_lyrics`.
pub fn render_svgs_with_highlight(
    source: &str,
    filename: &str,
    highlighted_measure_index: Option<usize>,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<Vec<String>, JianPuError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
    };
    let compile_result = compiler::compile(&score);
    let grid_pages =
        grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, highlighted_measure_index);
    let abs = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32);
    let docs = renderer::new_renderer::render_new(&abs, &config);
    Ok(serializer::serialize(&docs))
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo check 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add render_svgs_with_highlight public function"
```

---

### Task 6: Add `render_with_highlight` WASM export

**Files:**
- Modify: `crates/jianpu-wasm/src/lib.rs`

- [ ] **Step 1: Add the helper and export**

In `crates/jianpu-wasm/src/lib.rs`, add after the `render_response` function:

```rust
fn render_with_highlight_response(
    source: &str,
    highlighted_measure_index: usize,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
) -> RenderResponse {
    let tracks = enabled_tracks.as_deref();
    let lyrics = disabled_lyrics.as_deref();
    match jianpu_generator::render_svgs_with_highlight(
        source,
        "input.jianpu",
        Some(highlighted_measure_index),
        tracks,
        lyrics,
    ) {
        Ok(svgs) => RenderResponse::Ok { svgs },
        Err(e) => RenderResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, e)],
        },
    }
}
```

Then add the wasm-bindgen export:

```rust
/// Render `.jianpu` source with a measure highlighted.
///
/// Returns the same structured value as [`render`]:
/// - `{ "status": "ok", "svgs": ["<svg>...</svg>", ...] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[wasm_bindgen]
pub fn render_with_highlight(
    source: &str,
    highlighted_measure_index: usize,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
) -> JsValue {
    to_js_value(&render_with_highlight_response(
        source,
        highlighted_measure_index,
        enabled_tracks,
        disabled_lyrics,
    ))
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo check -p jianpu-wasm 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 3: Rebuild WASM**

```bash
cd web && npm run build:wasm 2>&1 | tail -20
```

Expected: successful build (or follow the project's existing WASM build command — check `web/package.json` for the exact script name if `build:wasm` doesn't exist).

- [ ] **Step 4: Commit**

```bash
git add crates/jianpu-wasm/src/lib.rs
git commit -m "feat: add render_with_highlight WASM export"
```

---

### Task 7: Update worker with `renderWithHighlight` request/response

**Files:**
- Modify: `web/src/worker/jianpu.worker.ts`

- [ ] **Step 1: Add the WASM function binding**

In `web/src/worker/jianpu.worker.ts`, after the `generateWavForMeasure` binding, add:

```typescript
const renderWithHighlight =
  'render_with_highlight' in jianpuWasm
    ? (jianpuWasm.render_with_highlight as (
        source: string,
        highlightedMeasureIndex: number,
        enabledTracks?: string[],
        disabledLyrics?: string[],
      ) => RenderResult)
    : null
```

- [ ] **Step 2: Add the new request/response types**

In `WorkerRequest`, add:

```typescript
  | {
      type: 'renderWithHighlight'
      source: string
      id: number
      highlightedMeasureIndex: number
      enabledTracks?: string[]
      disabledLyrics?: string[]
    }
```

In `WorkerResponse`, add:

```typescript
  | { type: 'highlightOk'; id: number; svgs: string[] }
  | { type: 'highlightErr'; id: number; diagnostics: Diagnostic[] }
```

- [ ] **Step 3: Add handler in `self.onmessage`**

Before the final `if (msg.type !== 'render') return` guard at the bottom of `self.onmessage`, add:

```typescript
  if (msg.type === 'renderWithHighlight') {
    if (!renderWithHighlight) {
      postMessage({
        type: 'highlightErr',
        id: msg.id,
        diagnostics: [
          {
            severity: 'error',
            message: 'render_with_highlight is not available in this build.',
            span: { start: 0, end: 0 },
          },
        ],
      } satisfies WorkerResponse)
      return
    }
    const result = renderWithHighlight(
      msg.source,
      msg.highlightedMeasureIndex,
      msg.enabledTracks,
      msg.disabledLyrics,
    )
    if (result.status === 'ok') {
      postMessage({
        type: 'highlightOk',
        id: msg.id,
        svgs: result.svgs,
      } satisfies WorkerResponse)
      return
    }
    postMessage({
      type: 'highlightErr',
      id: msg.id,
      diagnostics: result.diagnostics,
    } satisfies WorkerResponse)
    return
  }
```

- [ ] **Step 4: Verify TypeScript compiles**

```bash
cd web && npx tsc --noEmit 2>&1 | head -20
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add web/src/worker/jianpu.worker.ts
git commit -m "feat: add renderWithHighlight worker request/response handler"
```

---

### Task 8: Update `useJianpuWorker` with `highlightedSvgs` state

**Files:**
- Modify: `web/src/hooks/useJianpuWorker.ts`

- [ ] **Step 1: Add state, refs, and return field**

In the `JianpuWorkerState` interface (around line 30), add:

```typescript
  highlightedSvgs: string[]
```

In the hook body, after the `measureAudioGenerating` state declaration, add:

```typescript
  const [highlightedSvgs, setHighlightedSvgs] = useState<string[]>([])
  const highlightRenderRequestIdRef = useRef(0)
  const latestHighlightRenderIdRef = useRef(0)
```

- [ ] **Step 2: Handle `highlightOk` and `highlightErr` in the message handler**

Inside the `worker.onmessage` callback, add before the `if (msg.type === 'err')` block:

```typescript
      if (msg.type === 'highlightOk') {
        if (msg.id !== latestHighlightRenderIdRef.current) return
        setHighlightedSvgs(msg.svgs)
        return
      }

      if (msg.type === 'highlightErr') {
        if (msg.id !== latestHighlightRenderIdRef.current) return
        // Silently ignore highlight render errors — fall back to plain svgs
        return
      }
```

- [ ] **Step 3: Add effect to fire `renderWithHighlight` when `currentMeasureIndex` changes**

Add after the existing `notifyCursorOffset` effect (around line 395):

```typescript
  useEffect(() => {
    if (currentMeasureIndex === null) {
      setHighlightedSvgs([])
      return
    }
    const worker = workerRef.current
    if (!worker) return

    const id = ++highlightRenderRequestIdRef.current
    latestHighlightRenderIdRef.current = id

    worker.postMessage({
      type: 'renderWithHighlight',
      source: sourceRef.current,
      id,
      highlightedMeasureIndex: currentMeasureIndex,
      enabledTracks: enabledTracksRef.current,
      disabledLyrics: disabledLyricsRef.current,
    } satisfies WorkerRequest)
  }, [currentMeasureIndex])
```

Also reset `highlightedSvgs` when `source` changes (cursor measure resets, but SVGs are stale until re-render finishes). The existing source-change effect already sets `currentMeasureIndex` to `null`, which will trigger the effect above to clear `highlightedSvgs`. No separate action needed.

- [ ] **Step 4: Return `highlightedSvgs` from the hook**

In the `return { ... }` at the bottom of `useJianpuWorker`, add:

```typescript
    highlightedSvgs,
```

- [ ] **Step 5: Verify TypeScript compiles**

```bash
cd web && npx tsc --noEmit 2>&1 | head -20
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add web/src/hooks/useJianpuWorker.ts
git commit -m "feat: add highlightedSvgs state to useJianpuWorker"
```

---

### Task 9: Update `Preview` and `App` to use `highlightedSvgs`

**Files:**
- Modify: `web/src/components/Preview.tsx`
- Modify: `web/src/App.tsx`

- [ ] **Step 1: Add `highlightedSvgs` prop to `Preview`**

In `web/src/components/Preview.tsx`, update `PreviewProps` interface:

```typescript
interface PreviewProps {
  svgs: string[]
  highlightedSvgs?: string[]
  // ... rest unchanged
}
```

Update destructuring in `Preview`:

```typescript
export function Preview({
  svgs,
  highlightedSvgs = [],
  // ... rest unchanged
}: PreviewProps) {
```

Replace the `svgs.map(...)` in the render return with:

```typescript
        {(highlightedSvgs.length > 0 ? highlightedSvgs : svgs).map((svg) => (
          <div
            key={svg}
            className="preview-page"
            // biome-ignore lint/security/noDangerouslySetInnerHtml: trusted SVG from local WASM renderer
            dangerouslySetInnerHTML={{ __html: svg }}
          />
        ))}
```

Also update the empty check:

```typescript
        {svgs.length === 0 && highlightedSvgs.length === 0 && !rendering ? (
          <p className="preview-empty">{emptyMessage}</p>
        ) : null}
```

- [ ] **Step 2: Pass `highlightedSvgs` from `App`**

In `web/src/App.tsx`, destructure `highlightedSvgs` from `useJianpuWorker`:

```typescript
  const {
    parts,
    partsLoading,
    svgs,
    highlightedSvgs,
    // ... rest unchanged
  } = useJianpuWorker(source, disabledParts, disabledLyrics, store.active)
```

Pass it to `Preview`:

```typescript
          <Preview
            svgs={svgs}
            highlightedSvgs={highlightedSvgs}
            // ... rest unchanged
```

- [ ] **Step 3: Verify TypeScript compiles**

```bash
cd web && npx tsc --noEmit 2>&1 | head -20
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add web/src/components/Preview.tsx web/src/App.tsx
git commit -m "feat: show highlighted SVGs in Preview when cursor is inside a measure"
```

---

### Task 10: End-to-end smoke test

- [ ] **Step 1: Start the dev server**

```bash
cd web && npm run dev
```

- [ ] **Step 2: Open the browser and verify the feature**

1. Open the web editor.
2. Load or type a multi-measure `.jianpu` source (e.g. `simple.jianpu`).
3. Click inside the first measure in the editor.
4. Confirm: a semi-transparent amber rectangle appears behind the measure in the SVG preview.
5. Move the cursor to a different measure.
6. Confirm: the amber rectangle moves to the new measure.
7. Move the cursor to a non-measure area (e.g. `[metadata]` section).
8. Confirm: the amber rectangle disappears (falls back to unhighlighted SVGs).

- [ ] **Step 3: Commit if any minor fixups were made during testing**

```bash
git add -p
git commit -m "fix: <describe any fixup>"
```

---

## Self-Review

**Spec coverage:**
- [x] Semi-transparent amber rect — `rgba(255,200,0,0.25)` in serializer
- [x] Spans full vertical stack — `row_start` / `row_end` cover all system rows
- [x] Column range computed from block position — `column_start` / `column_end` in `compute_measure_highlight_location`
- [x] Persistent / updates on cursor move — effect on `currentMeasureIndex` re-fires `renderWithHighlight`
- [x] Resets when cursor leaves measure — effect clears `highlightedSvgs` when `currentMeasureIndex` is `null`
- [x] Baked into WASM, no DOM injection — `render_with_highlight` WASM export
- [x] Computed at grid_layout stage — `MeasureHighlight` uses grid-space coordinates
- [x] `AbsoluteContent::MeasureHighlight` variant (not a new struct on `AbsolutePage`) — coordinate resolver prepends to elements
- [x] `SvgKind::Rect` variant — renderer + serializer handle it
- [x] `render_with_highlight` WASM / `renderWithHighlight` worker pattern — mirrors `generate_wav_for_measure`
- [x] `highlightedSvgs` in same hook as play-current-measure — `useJianpuWorker`

**Type consistency check:**
- `MeasureHighlight { row_start: usize, row_end: usize, column_start: u32, column_end: u32 }` — used consistently in Task 1, 2, 3
- `AbsoluteContent::MeasureHighlight { width: f32, height: f32 }` — defined in Task 3, used in renderer in Task 3
- `SvgKind::Rect { width: f32, height: f32 }` — defined in Task 3, serialized in Task 3
- `highlightedSvgs: string[]` — state in Task 8, prop in Task 9, destructured in App Task 9
- `renderWithHighlight` / `highlightOk` / `highlightErr` — defined in Task 7, consumed in Task 8
