# Ditto Label Merge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When a ditto part is omitted from rendering in a measure, append its label (`, <name>`) to the label of the source row it was copied from.

**Architecture:** Change `compile_measure` in `src/compiler/mod.rs` to replace the `filter_map` with a `for` loop. Timed rows are pushed normally; Ditto rows append their label to the last pushed row. No other files change.

**Tech Stack:** Rust, `cargo test`

---

### Task 1: Write a failing test for ditto label merging

**Files:**
- Modify: `src/tests/ditto.rs`

- [ ] **Step 1: Add the failing test**

Append this test to `src/tests/ditto.rs`:

```rust
#[test]
fn ditto_part_label_is_merged_into_source_row_label() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano (S) = notes\n",
        "Alto (A) = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let blocks = crate::compiler::compile(&score);
    assert_eq!(
        blocks[0].rows.len(),
        1,
        "ditto Alto should produce no separate row"
    );
    assert_eq!(
        blocks[0].rows[0].label,
        "S, A",
        "source row label should include ditto part label"
    );
}

- [ ] **Step 2: Run the test to confirm it fails**

```bash
cargo test ditto_part_label_is_merged_into_source_row_label
```

Expected: FAIL — `blocks[0].rows[0].label` is `"S"` not `"S, A"`.

---

### Task 2: Implement ditto label merging in `compile_measure`

**Files:**
- Modify: `src/compiler/mod.rs:18-42`

- [ ] **Step 1: Replace `filter_map` with a `for` loop**

Current `compile_measure` body (lines 18-42):

```rust
fn compile_measure(measure: &MultiPartMeasure, bar_number: usize) -> MeasureBlock {
    let decorations = collect_decorations(measure, bar_number);
    let rows = measure
        .parts
        .iter()
        .enumerate()
        .filter_map(|(part_idx, part_row)| {
            let slice = part_row.rendered_slice()?;
            let label = part_row.name().cloned().unwrap_or_default();
            let id = RowId(
                part_row
                    .name()
                    .cloned()
                    .unwrap_or_else(|| format!("__anon_{part_idx}")),
            );
            let elements = compile_part_slice(slice);
            Some(MeasureRow {
                id,
                label,
                elements,
            })
        })
        .collect();
    MeasureBlock { rows, decorations }
}
```

Replace with:

```rust
fn compile_measure(measure: &MultiPartMeasure, bar_number: usize) -> MeasureBlock {
    let decorations = collect_decorations(measure, bar_number);
    let mut rows: Vec<MeasureRow> = Vec::new();
    for (part_idx, part_row) in measure.parts.iter().enumerate() {
        match part_row.rendered_slice() {
            Some(slice) => {
                let label = part_row.name().cloned().unwrap_or_default();
                let id = RowId(
                    part_row
                        .name()
                        .cloned()
                        .unwrap_or_else(|| format!("__anon_{part_idx}")),
                );
                let elements = compile_part_slice(slice);
                rows.push(MeasureRow { id, label, elements });
            }
            None => {
                // Ditto row: append its label to the source row's label.
                if let Some(last) = rows.last_mut() {
                    let ditto_label = part_row.name().map(String::as_str).unwrap_or("");
                    if !ditto_label.is_empty() {
                        last.label.push_str(", ");
                        last.label.push_str(ditto_label);
                    }
                }
            }
        }
    }
    MeasureBlock { rows, decorations }
}
```

- [ ] **Step 2: Run the new test**

```bash
cargo test ditto_part_label_is_merged_into_source_row_label
```

Expected: PASS.

- [ ] **Step 3: Run the full test suite**

```bash
cargo test
```

Expected: 306 passed (was 305 before this task).

- [ ] **Step 4: Commit**

```bash
git add src/compiler/mod.rs src/tests/ditto.rs
git commit -m "feat: merge ditto part labels into source row label"
```
