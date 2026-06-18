# SVG Variant ID Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `data-variant` attribute to every generated SVG element so each element is identifiable by its musical content type in browser devtools.

**Architecture:** Add `variant: &'static str` to `SvgElement`; populate it in every construction site in the renderer; emit `data-variant="{}"` in every serializer format string.

**Tech Stack:** Rust, SVG

---

### Task 1: Add `variant` field to `SvgElement` and populate it in the renderer

**Files:**
- Modify: `src/renderer/new_types.rs`
- Modify: `src/renderer/new_renderer.rs`

- [ ] **Step 1: Add `variant` field to `SvgElement`**

In `src/renderer/new_types.rs`, update the struct:

```rust
pub struct SvgElement {
    pub x: f32,
    pub y: f32,
    pub variant: &'static str,
    pub kind: SvgKind,
}
```

- [ ] **Step 2: Verify it fails to compile**

```bash
cargo build 2>&1 | head -40
```

Expected: multiple "missing field `variant`" errors in `new_renderer.rs` and `serializer/mod.rs`.

- [ ] **Step 3: Update all `SvgElement` constructions in `new_renderer.rs`**

In `src/renderer/new_renderer.rs`, add `variant` to every `SvgElement { ... }` literal. Use the mapping below — every construction inside a given function gets the same variant string.

`render_note_head` — all four constructions (digit text, duration dot, upper octave dots, lower octave dots) get `variant: "note-head"`:

```rust
// 1. Note digit
results.push(SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "note-head",
    kind: SvgKind::Text {
        content: pitch_to_digit(pitch).to_string(),
        font_size: *base_font_size,
        anchor: TextAnchor::Middle,
        baseline: DominantBaseline::Middle,
        font: FontFamily::Monospace,
        weight: FontWeight::Normal,
        italic: false,
    },
});

// 2. Dotted note circle
results.push(SvgElement {
    x: dot_x,
    y: elem.y,
    variant: "note-head",
    kind: SvgKind::Circle { r: dot_radius },
});

// 3. Upper octave dots
results.push(SvgElement {
    x: elem.x,
    y: dot_y,
    variant: "note-head",
    kind: SvgKind::Circle { r: dot_radius },
});

// 4. Lower octave dots
results.push(SvgElement {
    x: elem.x,
    y: dot_y,
    variant: "note-head",
    kind: SvgKind::Circle { r: dot_radius },
});
```

`render_rest` — both constructions (rest text, duration dot) get `variant: "rest"`:

```rust
results.push(SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "rest",
    kind: SvgKind::Text { ... },
});

results.push(SvgElement {
    x: dot_x,
    y: elem.y,
    variant: "rest",
    kind: SvgKind::Circle { r: dot_radius },
});
```

`render_chord_symbol` — `variant: "chord-symbol"`:

```rust
vec![SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "chord-symbol",
    kind: SvgKind::Text { ... },
}]
```

`render_horizontal_line` — `variant: "horizontal-line"`:

```rust
vec![SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "horizontal-line",
    kind: SvgKind::Line { ... },
}]
```

`render_underline` — `variant: "underline"`:

```rust
vec![SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "underline",
    kind: SvgKind::Line { ... },
}]
```

`render_tie_or_slur` — `variant: "tie-or-slur"`:

```rust
vec![SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "tie-or-slur",
    kind: SvgKind::Path { ... },
}]
```

`render_bar_line` — `variant: "bar-line"`:

```rust
vec![SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "bar-line",
    kind: SvgKind::Line { ... },
}]
```

`render_lyric` — `variant: "lyric"`:

```rust
vec![SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "lyric",
    kind: SvgKind::Text { ... },
}]
```

`AbsoluteContent::Text` inline arm in `render_element` — `variant: "text"`:

```rust
vec![SvgElement {
    x: elem.x,
    y: elem.y,
    variant: "text",
    kind: SvgKind::Text { ... },
}]
```

- [ ] **Step 4: Fix `SvgElement` literals in `serializer/mod.rs` tests**

In `src/serializer/mod.rs`, every `SvgElement { ... }` in the `#[cfg(test)]` block needs `variant`. Use `"text"` for text elements, `"note-head"` for circle elements, and the appropriate value for line/path elements. Exact changes:

`text_doc` helper — add `variant: "text"`:

```rust
elements: vec![SvgElement {
    x: 10.0,
    y: 20.0,
    variant: "text",
    kind: SvgKind::Text { ... },
}],
```

`circle_serializes_correctly` — add `variant: "note-head"`:

```rust
elements: vec![SvgElement {
    x: 5.0,
    y: 5.0,
    variant: "note-head",
    kind: SvgKind::Circle { r: 3.0 },
}],
```

`line_serializes_correctly` — add `variant: "bar-line"`:

```rust
elements: vec![SvgElement {
    x: 0.0,
    y: 0.0,
    variant: "bar-line",
    kind: SvgKind::Line {
        x2: 50.0,
        y2: 0.0,
        stroke_width: 1.0,
    },
}],
```

`path_serializes_correctly` — add `variant: "tie-or-slur"`:

```rust
elements: vec![SvgElement {
    x: 0.0,
    y: 0.0,
    variant: "tie-or-slur",
    kind: SvgKind::Path {
        control_x: 25.0,
        control_y: -10.0,
        end_x: 50.0,
        end_y: 0.0,
        stroke_width: 1.5,
    },
}],
```

- [ ] **Step 5: Verify it compiles and tests pass**

```bash
cargo test 2>&1 | tail -20
```

Expected: all tests pass (the serializer does not yet emit `data-variant`, but the struct change compiles cleanly).

- [ ] **Step 6: Commit**

```bash
git add src/renderer/new_types.rs src/renderer/new_renderer.rs src/serializer/mod.rs
git commit -m "feat: add variant field to SvgElement"
```

---

### Task 2: Emit `data-variant` in the serializer (TDD)

**Files:**
- Modify: `src/serializer/mod.rs`

- [ ] **Step 1: Write failing tests**

In `src/serializer/mod.rs`, in the `#[cfg(test)]` block, add these four tests (after the existing ones):

```rust
#[test]
fn text_element_has_data_variant() {
    let result = serialize(&[text_doc("hello")]);
    assert!(result[0].contains(r#"data-variant="text""#));
}

#[test]
fn circle_element_has_data_variant() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 5.0,
            y: 5.0,
            variant: "note-head",
            kind: SvgKind::Circle { r: 3.0 },
        }],
    };
    let result = serialize(&[doc]);
    assert!(result[0].contains(r#"data-variant="note-head""#));
}

#[test]
fn line_element_has_data_variant() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 0.0,
            y: 0.0,
            variant: "bar-line",
            kind: SvgKind::Line {
                x2: 50.0,
                y2: 0.0,
                stroke_width: 1.0,
            },
        }],
    };
    let result = serialize(&[doc]);
    assert!(result[0].contains(r#"data-variant="bar-line""#));
}

#[test]
fn path_element_has_data_variant() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 0.0,
            y: 0.0,
            variant: "tie-or-slur",
            kind: SvgKind::Path {
                control_x: 25.0,
                control_y: -10.0,
                end_x: 50.0,
                end_y: 0.0,
                stroke_width: 1.5,
            },
        }],
    };
    let result = serialize(&[doc]);
    assert!(result[0].contains(r#"data-variant="tie-or-slur""#));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test serializer 2>&1 | tail -30
```

Expected: the four new tests fail with assertion errors (attribute not yet in output).

- [ ] **Step 3: Update `serialize_element` to emit `data-variant`**

In `src/serializer/mod.rs`, update `serialize_element` — add `data-variant="{}"` with `el.variant` to each format string, placed after the geometry attributes and before the style attributes:

`SvgKind::Text` format string:

```rust
out.push_str(&format!(
    r#"<text x="{:.1}" y="{:.1}" data-variant="{}" font-size="{:.1}" text-anchor="{}" dominant-baseline="{}" font-family="{}" font-weight="{}" {}>{}</text>"#,
    el.x, el.y, el.variant, font_size, anchor_str, baseline_str, font_str, weight_str, style_str,
    escape_xml(content)
));
```

`SvgKind::Line` format string:

```rust
out.push_str(&format!(
    r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" data-variant="{}" stroke="black" stroke-width="{:.1}"/>"#,
    el.x, el.y, x2, y2, el.variant, stroke_width
));
```

`SvgKind::Circle` format string:

```rust
out.push_str(&format!(
    r#"<circle cx="{:.1}" cy="{:.1}" data-variant="{}" r="{:.1}" fill="black"/>"#,
    el.x, el.y, el.variant, r
));
```

`SvgKind::Path` format string:

```rust
out.push_str(&format!(
    r#"<path d="M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}" data-variant="{}" fill="none" stroke="black" stroke-width="{:.1}"/>"#,
    el.x, el.y, control_x, control_y, end_x, end_y, el.variant, stroke_width
));
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1 | tail -20
```

Expected: all tests pass including the four new `data-variant` assertions.

- [ ] **Step 5: Commit**

```bash
git add src/serializer/mod.rs
git commit -m "feat: emit data-variant attribute on all SVG elements"
```
