# Grouping Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reject 4/4 `.jianpu` scores whose rhythm spelling crosses the half-bar boundary or leaves a dotted-eighth beat tail without a beam group.

**Architecture:** Add `src/grouping.rs` with hardcoded 4/4 rules. Call `validate_measure_grouping` from `validate_and_pad_beats` after implicit padding so durations are final. Unit tests in `grouping.rs`; integration via `parser::parse` errors.

**Tech Stack:** Rust, existing parser AST (`ParsedNote`, `ParsedRest`, `ScoreEvent`)

---

## File Map

| File | Change |
|------|--------|
| `src/grouping.rs` | **Create** — validator + tests |
| `src/lib.rs` | Register `pub mod grouping;` |
| `src/parser/score/interleaved_parser.rs` | Pass time sig into `validate_and_pad_beats`; call validator |
| `syntax.md` | Document grouping validation under Measure validation |

---

### Task 1: Rule 1 — half-bar boundary

**Files:**
- Create: `src/grouping.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Register module**

Add to `src/lib.rs` after `pub mod grouper;`:

```rust
pub mod grouping;
```

- [ ] **Step 2: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    const HEADER: &str = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120)\n",
    );

    #[test]
    fn rejects_half_bar_crossing() {
        let input = concat!(HEADER, "1. 2. 3 4\n");
        let err = parser::parse(input, "test.jianpu").unwrap_err();
        assert!(err.message.contains("half-bar boundary"));
    }

    #[test]
    fn accepts_half_bar_split_with_beam_group() {
        let input = concat!(HEADER, "1. (2_ 2) 3_ 4_ 5_\n");
        assert!(parser::parse(input, "test.jianpu").is_ok());
    }
}
```

- [ ] **Step 3: Run test — expect fail**

Run: `cargo test grouping::tests::rejects_half_bar_crossing -- --nocapture`
Expected: FAIL (module/function missing)

- [ ] **Step 4: Implement rule 1**

```rust
use crate::ast::parsed::ScoreEvent;
use crate::error::{JianPuError, Spanned};

const HALF_BAR_BOUNDARY: u32 = 8;

pub fn validate_measure_grouping(
    events: &[Spanned<ScoreEvent>],
    time_num: u8,
    time_den: u8,
) -> Result<(), JianPuError> {
    if time_num != 4 || time_den != 4 {
        return Ok(());
    }

    let mut pos = 0u32;
    for event in events {
        let (duration, span) = match &event.value {
            ScoreEvent::Note(n) => (n.duration, &event.span),
            ScoreEvent::Rest(r) => (r.duration, &event.span),
            _ => continue,
        };

        if pos < HALF_BAR_BOUNDARY && pos + duration > HALF_BAR_BOUNDARY {
            return Err(JianPuError::new(
                span.clone(),
                "note/rest crosses the half-bar boundary (beat 2→3); use a beam group or tie to show the split"
                    .to_string(),
            ));
        }

        pos += duration;
    }

    Ok(())
}
```

- [ ] **Step 5: Run tests — expect pass**

Run: `cargo test grouping::tests -- --nocapture`

- [ ] **Step 6: Commit**

```bash
git add src/grouping.rs src/lib.rs
git commit -m "feat(grouping): reject half-bar boundary crossings in 4/4"
```

---

### Task 2: Rule 2 — dotted-eighth tail

**Files:**
- Modify: `src/grouping.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn rejects_dotted_eighth_without_tail_group() {
    let input = concat!(HEADER, "1_. 2_. 3_ 4_\n");
    let err = parser::parse(input, "test.jianpu").unwrap_err();
    assert!(err.message.contains("dotted eighth"));
}

#[test]
fn accepts_dotted_eighth_with_sixteenth_tail_group() {
    let input = concat!(HEADER, "1_. (2=) 3_ 4_ 5_ 6_ 7_ 1_\n");
    assert!(parser::parse(input, "test.jianpu").is_ok());
}

#[test]
fn rejects_dotted_eighth_rest_without_tail_group() {
    let input = concat!(HEADER, "0_. 1_ 2_ 3_ 4_ 5_ 6_ 7_\n");
    let err = parser::parse(input, "test.jianpu").unwrap_err();
    assert!(err.message.contains("dotted eighth"));
}
```

- [ ] **Step 2: Run tests — expect fail**

Run: `cargo test grouping::tests::rejects_dotted_eighth -- --nocapture`

- [ ] **Step 3: Implement rule 2 in `validate_measure_grouping`**

After advancing `pos` for each timed event, when `dotted && duration == 3 && pos_before % 4 == 0`:

1. Peek next timed event index.
2. If missing → error.
3. If `ScoreEvent::Note` with `group_membership > 0`: sum durations of that group's segment (walk while `group_continuation > 0` on prior note); require sum == 1.
4. If `ScoreEvent::Rest` with `duration == 1`: accept (sixteenth rest tail; rests lack `group_membership`).
5. Otherwise → error with `"dotted eighth must be followed by a beam group filling the remaining sixteenth"`.

Skip consumed tail events in the main loop index when a group segment is validated.

- [ ] **Step 4: Run all grouping tests**

Run: `cargo test grouping::tests -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add src/grouping.rs
git commit -m "feat(grouping): reject ungrouped dotted-eighth tails in 4/4"
```

---

### Task 3: Wire into parser pipeline

**Files:**
- Modify: `src/parser/score/interleaved_parser.rs`

- [ ] **Step 1: Extend `validate_and_pad_beats` signature**

```rust
fn validate_and_pad_beats(
    mut events: Vec<Spanned<ScoreEvent>>,
    expected: u32,
    time_num: u8,
    time_den: u8,
) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError>
```

After padding block, before `Ok(events)`:

```rust
crate::grouping::validate_measure_grouping(&events, time_num, time_den)?;
```

- [ ] **Step 2: Update call site in `process_data_line`**

```rust
let events = validate_and_pad_beats(
    token_parser::parse_tokens(tokens, group_state)?,
    beats_expected,
    *ctx.time_num,
    *ctx.time_den,
)?;
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add src/parser/score/interleaved_parser.rs
git commit -m "feat(parser): run 4/4 grouping validation after measure padding"
```

---

### Task 4: Document syntax

**Files:**
- Modify: `syntax.md`

- [ ] **Step 1: Add subsection under Measure validation**

```markdown
### Grouping validation (4/4 only)

In 4/4, the parser rejects rhythm spellings that cross metrical boundaries without exposing the split:

1. **Half-bar boundary:** no single note/rest may span from before beat 3 into beat 3 or beyond (quarter-beat position 8). Use a beam group such as `(2_ 2)` or a tie instead of a single long value (e.g. `1. 2. 3 4` is invalid; `1. (2_ 2) 3_ 4_ 5_` is valid).
2. **Dotted-eighth tail:** a dotted eighth note/rest at the start of a beat must be followed by a beam group filling the remaining sixteenth (e.g. `1_. (2=) 3_ …`); `1_. 2_. 3_ 4_` is invalid.

Other time signatures skip these checks for now. Violations are parse errors.
```

- [ ] **Step 2: Commit**

```bash
git add syntax.md
git commit -m "docs(syntax): document 4/4 grouping validation rules"
```
