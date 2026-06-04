# Bar Numbers Design

## Summary

Show a small bar number above the left system bar at the start of every row group, counting from bar 1 sequentially. No special handling for pickup bars — users pad them with rests.

## Data Model

Add one variant to `GridContent` in `src/layout/types.rs`:

```rust
BarNumber { number: u32 },
```

Positioning convention:
- `column = label_cols` (same column as the left system bar)
- `row = current_row_offset` (the header row directly above the staff)
- `HorizontalAlignment::Left`
- `VerticalAlignment::Bottom` — baseline sits flush above the left system bar

## Layout Logic (`src/layout/mod.rs`)

1. Introduce `let mut bar_number: u32 = 1;` before the measure loop.
2. At every `is_line_start` block (first line and each row-wrap), emit:
   ```rust
   GridElement {
       position: GridPosition { column: label_cols, row: current_row_offset },
       horizontal_alignment: HorizontalAlignment::Left,
       vertical_alignment: VerticalAlignment::Bottom,
       content: GridContent::BarNumber { number: bar_number },
   }
   ```
3. After emitting the bar line at the end of each measure, increment: `bar_number += 1;`

## Renderer (`src/renderer.rs`)

Add a match arm for `GridContent::BarNumber { number }`:

```rust
GridContent::BarNumber { number } => {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="auto" font-family="sans-serif">{}</text>"#,
        x, y, base_font_size * 0.6, number
    ));
}
```

Font size is 60% of the note-head size to keep it clearly subordinate.

## Testing

Add a test in `src/layout/mod.rs` that:
- Lays out a score with enough measures to produce at least 2 row groups.
- Asserts each row group contains a `BarNumber` element at `column = label_cols`, at the first row of that row group (the header row above the staff, i.e. `row = header_rows + n * row_group_height` for the nth row group).
- Asserts the numbers are sequential starting from 1.
