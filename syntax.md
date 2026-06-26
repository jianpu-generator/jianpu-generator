# Jianpu Generator — `.jianpu` Syntax Reference

This document describes the input syntax accepted by **jianpu-generator** as implemented today. File extension: `.jianpu`.

---

## File structure

A `.jianpu` file has three sections in fixed order:

```
# metadata
…key = value fields…

# parts
…track declarations…

# score
…interleaved score content…
```

- `# metadata` — **required**
- `# parts` — **required**
- `# score` — **required**
- Sections must appear in the order above.
- Legacy `# score:Name` / `# lyrics:Name` sections are **not** supported.

Whitespace around `=` in metadata is optional. Metadata values may be quoted with `"`.

---

## Metadata

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `title` | yes | — | Piece title (rendered in header) |
| `author` | no | none | Author name (rendered in header) |
| `subtitle` | no | none | Subtitle line |
| `max columns` | no | `28` | Maximum grid columns per system line before wrapping |
| `row height` | no | `24` | Vertical spacing of one part row (pixels) |
| `label width` | no | `40` | Horizontal space reserved for part labels (pixels) |
| `note number width` | no | `8` | Horizontal space per note column (pixels) |

---

## Parts section

One track per line. Blank lines are ignored.

```
<display-name> [[<abbreviation>]] = <column> [<column>…]
<display-name> [[<abbreviation>]] = follow[<target-abbreviation>]
```

### Left-hand side

| Form | Display name | Abbreviation (row label) |
|------|--------------|----------------------------|
| `Alto 1 & Tenor [A1&T]` | `Alto 1 & Tenor` | `A1&T` |
| `Melody` | `Melody` | `Melody` |
| `main` | `main` | `main` |

- Square brackets `[Abbr]` denote the **abbreviation** used as the row label and for `[Key]` prefix lines in the score.
- When brackets are omitted, the abbreviation equals the full display name.
- The display name is stored for future legend rendering; row labels use the abbreviation only.

### Right-hand side

| Pattern | Meaning | Score lines per measure |
|---------|---------|-------------------------|
| `chords` | Chord-symbol row | 1 |
| `notes` | Notes only (instrumental) | 1 |
| `notes+lyrics` | Notes + lyrics | 2 (notes, then lyrics) |
| `follow[X]` | Inherit column layout from the part with abbreviation `X` | same as target |

An optional soundfont string `"<number>: <name>"` may follow the kind token (or `follow[X]` bracket) to select the MIDI timbre for that part. The number is the General MIDI program number (0–127). For example: `notes "52: Choir Aahs"` or `follow[A] "1: Grand Piano"`. If omitted, the default is program 52 (Choir Aahs).

Rules:

- Duplicate abbreviations across tracks are an error.
- At least one track must be declared.
- `follow[X]` cannot be used for the first declared part.
- The target abbreviation `X` in `follow[X]` must refer to an already-declared part (declared before the follower).
- A `follow[X]` part that is not explicitly mentioned in a measure copies `X`'s content and is visually suppressed (row not rendered).
- A `follow[X]` part can be partially or fully overridden using `[Key]` prefix lines in the score.

Example (multi-part vocal score with chords):

```
# parts
main = chords
Alto 1 & Tenor [A1&T] = notes+lyrics
Alto 2 [A2] = notes+lyrics
Soprano 1 [S1] = notes+lyrics
Soprano 2 [S2] = notes+lyrics
```

Minimal single-part example:

```
# parts
Melody = notes+lyrics
```

---

## Score section — measure groups

The `[score]` body is split into **measure groups** by **blank lines**. Each group is exactly one bar (measure).

```
(bpm=92 key=C4 time=4/4 label="Verse 1")
1 - - -
5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_)
白陽旗旛在大道盛宏

6m - - -
3_ (1_1) 0_- 1= 1=
昌花花
```

### Group layout

1. **Optional directive line** — first line containing at least one directive keyword (`bpm=`, `key=`, `time=`, or `label=`)
2. **Data lines** — one per score line implied by `# parts`, in track declaration order

Lines are trimmed; leading/trailing spaces on a line are ignored. A completely empty line separates measure groups (it is not a data line).

### Positional mapping

Positional lines (no `[Abbrev]` prefix) fill the **first declared part's** slots in order. All remaining parts are filled via `[Key]` prefix lines, `follow[X]` target copying, or implicit fill.

| Track | Lines per measure (positional) |
|-------|--------------------------------|
| First part only | all its score lines, in order |

Trailing positional lines may be omitted; they are filled with rests (`0`) or no-lyrics (`_`) depending on slot type. You cannot skip a line in the middle.

### Key-based part prefix (`[Abbrev]`)

A data line may begin with `[Abbrev]` to route it to a specific part by abbreviation:

```
[A2] 5 6 7 0
```

- Positional lines (no prefix) belong exclusively to the **first declared part**; they must come before any `[Key]` lines in the group.
- Any number of `[Key]` lines may appear for the same part; they fill that part's slots in declaration order (first line → first slot, second line → second slot, …).
- `[Key]` lines targeting the first declared part's abbreviation are an error; the line is dropped.
- An unrecognised abbreviation is an error; the line is dropped.
- Parts not covered by any positional or `[Key]` line use their `follow[X]` target's content when declared as such, or are filled with implicit rests/no-lyrics otherwise.

**Example — only part C plays, A and B are not-mentioned:**

```
# parts
A = notes
B = notes
C = notes

# score
time=4/4 key=C4 bpm=120
1 2 3 4

[C] 5 6 7 0
```

Measure 2: C plays `5 6 7 0`. A and B have no explicit lines → filled with `0` (rest) and marked not-mentioned (rows suppressed).

**Example — mix positional and key-based in one measure with a follow part:**

```
# parts
A = notes
B = follow[A]
C = notes

# score
1 2 3 4      ← positional → fills A
[C] 5 6 7 0  ← key-based → fills C
```

A: `1 2 3 4`. B: not mentioned → copies A's content via `follow`. C: `5 6 7 0`.

**Example — follow part with partial key override:**

```
# parts
Soprano [S] = notes+lyrics
Alto [A] = follow[S]

# score
time=4/4 key=C4 bpm=120
1 2 3 4
do re mi fa
[A] 5 6 7 1
```

Soprano: notes=`1 2 3 4`, lyrics=`do re mi fa`. Alto: notes=`5 6 7 1` (key override), lyrics=`do re mi fa` (copied from Soprano via follow).

---

## Directive lines

An optional first line of whitespace-separated `key=value` directives sets global values for that measure and onward (until overridden):

```
bpm=92 key=C4 time=4/4 label="Verse 1"
```

| Directive | Example | Effect |
|-----------|---------|--------|
| `bpm=` | `bpm=120` | Tempo (beats per minute) |
| `key=` | `key=C4`, `key=F#3`, `key=Bb4` | Key signature (`1` = this note) |
| `time=` | `time=4/4`, `time=3/4` | Time signature |
| `label=` | `label="Verse 1"` | Section label rendered above the row group |

Rules:

- Multiple directives may appear on one line, separated by whitespace.
- `label=` value must be a quoted string; empty labels are rejected.
- Directives apply to **all** parts. They are stored on the first notes part and propagate through grouping.
- `label` applies only to the measure where it is declared (does not persist to the next bar).
- `bpm`, `key`, and `time` persist until the next directive line overrides them.

### Rendering

When `time=` or `bpm=` changes on a measure, the generator may add a **directive row** above the bar-number / section-label row for that system line. Time signature and BPM appear once on that row (not on each part row), aligned with each measure’s note-start column. They do not shift notes or lyrics horizontally. If neither value changes on any measure in the line, the directive row is omitted.

Note names: `A` `B` `C` `D` `E` `F` `G`, with optional `#` or `b` accidental, followed by octave digit (e.g. `4`).

---

## Notes syntax

Note lines are a sequence of **atoms** (notes, rests, chords, extensions, groups). Whitespace is optional between atoms and is ignored inside `(…)` groups.

Example: `((1 1) 5 5)` is equivalent to `((11)55)`.

### Pitch and rest

| Token part | Meaning |
|------------|---------|
| `1`–`7` | Scale degree (movable do) |
| `0` | Rest |

### Duration suffixes

Duration is measured in **quarter-beats** (sixteenth-note units). In 4/4, one full beat = 4 quarter-beats; a full 4/4 bar = 16 quarter-beats.

| Suffix | Quarter-beats | Typical name (4/4) |
|--------|---------------|---------------------|
| *(none)* | 4 | Quarter note (one beat) |
| `_` | 2 | Eighth note |
| `=` | 1 | Sixteenth note |

Suffix order is flexible (`1_,'` and `1',_` are equivalent).

### Octave markers

| Suffix | Meaning |
|--------|---------|
| `'` | Raise octave (each `'` = one octave up) |
| `,` | Lower octave (each `,` = one octave down) |

`'` and `,` **cannot be mixed** on the same note.

Examples: `1'` (octave up), `1,,` (two octaves down), `3_,'` (eighth note, up one octave).

### Modifiers

| Suffix | Meaning |
|--------|---------|
| `.` | Dotted (add half the base duration). Cannot combine with `=` (sixteenth) notes. |
| `-` | Extend the previous **note** by one beat (4 quarter-beats) |

Example: `2 - - -` is a whole note in 4/4 (equivalent to `2---`).

You can also attach dashes as suffixes on a note (`2---`). Both forms may be mixed in one measure.

**Rests cannot use `-`.** Conventional 简谱 lengthens rests by repeating `0`, not增时线. These are errors:

- `0-`, `0---` (suffix dashes on a rest)
- `0 -`, `0 - - -` (standalone dashes after a rest)

Use repeated rests instead: `0 0` (half rest in 4/4), `0 0 0 0` (whole rest). Shorter rests still use `_`, `=`, or `.` on a single `0` (`0_`, `0=`, `0.`).

### Tie and slur groups

Parentheses connect notes with tie/slur arcs (happi123-style 连音符). A group may span measures: the opening `(` can appear at the end of one bar and the closing `)` at the start of the next.

| Form | Meaning |
|------|---------|
| `(12)` | Slur/tie from 1 into 2 |
| `(433)` | Slur chain across 4→3→3 |
| `(6-7)` | Note 6 extended one beat (`6-`), slurred into 7 |
| `111(1` … `2)345` | Cross-measure slur: `(1` opens in bar 1, `2)` closes in bar 2 |
| `(3= (2_1_))` | Nested groups: outer slur 3→2→1, inner slur 2→1 |

Groups may be **nested**: a `(…)` inside another `(…)` adds an inner tie/slur arc while the outer group still connects all enclosed notes. Each nested group must still contain at least 2 notes.

A group must contain **at least 2 notes** (counting notes across a cross-measure open/close). Single-note groups like `(5)` are invalid.

Adjacent digits without spaces also start new notes: `505` is three quarter notes; `(12)31` is a group plus two more notes.

Trailing duration may be omitted when the remaining measure beats extend the last note. In 4/4, `1` is equivalent to `1---`; `1 2` is equivalent to `1 2--`.

### Inline directives (notes row)

These tokens may also appear in a notes line (uncommon; usually placed in `(...)` directive rows instead):

| Token | Meaning |
|-------|---------|
| `bpm=N` | Tempo change |
| `1=<Note><octave>` | Key change, e.g. `1=C4`, `1=Bb4` (only when followed by A–G) |
| `N/N` | Time signature change, e.g. `4/4` |

Note: `1=` followed by a digit pitch (e.g. `1=,`) is a sixteenth note, not a key change.

### Measure validation

Note and rest durations in a row must fill the measure capacity. For time signature `N/D`:

```
measure capacity = N × (16 / D) quarter-beats
```

(e.g. 4/4 → 16, 3/4 → 12). Too many quarter-beats is a parse error. A shortfall extends the last note/rest when possible; otherwise it is a parse error.

#### Grouping validation (4/4 only)

In 4/4, the parser rejects rhythm spellings that cross metrical boundaries without exposing the split:

1. **Half-bar boundary:** after beat 1, no single note/rest may span from before beat 3 into beat 3 or beyond (quarter-beat position 8). Use a beam group such as `(2_ 2_)` or a tie instead of a single long value (e.g. `1. 2. 3_ 4_` is invalid; `1. (2_ 2_) 3_ 4_ 0_` is valid). Long notes/rests starting on beat 1 (including a fully extended `1` or `1---`) are allowed.
2. **Dotted-eighth tail:** a dotted eighth note/rest at the start of a beat must be followed immediately by a sixteenth note/rest filling the remaining sixteenth (e.g. `1_. 2= 3_ …`); `1_. 2_ 3_ 4_` is invalid (`2_.` is a dotted eighth, not an eighth).

Other time signatures skip these checks for now. Violations are parse errors.

### Examples

| Token | Meaning |
|-------|---------|
| `1` | Quarter note on degree 1 |
| `3_` | Eighth note on degree 3 |
| `5=` | Sixteenth note on degree 5 |
| `1_.` | Dotted eighth note |
| `(12)` | Quarter notes 1 and 2, slurred/tied |
| `6,` | Degree 6, one octave down |
| `0` | Quarter rest |
| `0 0` | Half rest (two quarter rests) |
| `0 0 0 0` | Whole rest in 4/4 |
| `0_` | Eighth rest |
| `1. 1= 6=, (2_=2_)` | Mixed durations, octaves, and a slur group |

---

## Lyrics syntax

Lyrics lines are plain text tokenised into syllables:

| Script | Rule |
|--------|------|
| CJK (Chinese, Japanese, Korean) | Each character is one syllable |
| Latin | Space-separated words/syllables |

### Syllable break (`-` attached to a word)

A `-` **attached** to the end of a Latin syllable marks a word split across notes — the hyphen is part of the syllable text:

```
1 1 5 5
twin- kle twin- kle     ← "twinkle" split across two notes each
```

This is distinct from a **standalone** `-` surrounded by whitespace (held syllable, below).

### Held syllable (`-` within lyrics)

A `-` **inside** a lyrics line marks the **preceding** syllable as *held* — it stretches across tied notes:

```
he llo - world     ← "llo" is held across the tied note
你 - - 好           ← first 你 is held across two tied notes
```

This is distinct from `-` on a notes line (duration extension) and distinct from `_` (see below).

### No-lyrics marker (`_`)

A lyrics line whose **entire** trimmed content is `_` means **zero syllables** for that part in this measure (instrumental bar):

```
1 2 3 4
do re mi fa

5 6 7 1
_
```

- `_` is valid **only** on lyrics columns.
- On notes or chord columns, `_` alone is a parse error (`_` is already the eighth-note duration prefix on notes lines).

### Empty lyrics

Empty lyrics lines are **not** allowed. Whitespace-only lines are treated as measure separators, not as empty lyrics. To express silence, write `_`.

### Lyrics–notes tally

In each measure, the number of lyric syllables must match the number of notes that take lyrics in the paired notes row:

- Each non-rest note head counts, except a **tie continuation** (same pitch immediately after a tied note, including across a bar line).
- Held-syllable markers (`-`) count as their own syllables — e.g. `你 - 好` is three syllables for three lyric slots.
- The `_` no-lyrics marker skips this check (zero syllables allowed regardless of notes).

Mismatch is a parse error, e.g. `lyrics has 3 syllables but notes need 4 in part 'Soprano'`.

---

## Chord syntax

Chord lines use Nashville number symbols. Duration works like notes: each token occupies one beat; `-` extends the previous chord.

| Token | Meaning |
|-------|---------|
| `0` | Chord rest |
| `-` | Extend previous chord one beat |
| `<symbol>` | Chord (see grammar below) |

### Chord symbol grammar

```
<chord>      ::= <degree> <accidental>? <triad>? <extension>? ("/" <bass>)?
<degree>     ::= 1–7
<accidental> ::= "#" | "b"
<triad>      ::= "m" | "o" | "+"
<extension>  ::= "M7" | "7"
<bass>       ::= <degree> <accidental>?
```

Parsing checks longest suffix first (`M7` before `7`; `m` before extension).

| Input | Meaning |
|-------|---------|
| `1` | I major |
| `1m` | I minor |
| `1o` | I diminished |
| `1+` | I augmented |
| `17` | I dominant 7th |
| `1M7` | I major 7th |
| `1m7` | I minor 7th |
| `1#m7` | I♯ minor 7th |
| `3b` | ♭III major |
| `1/5` | I major, 5 in bass (e.g. C/G) |
| `6m/5` | vi minor, 5 in bass (e.g. Am/G) |

### Duration suffixes

Chord heads accept the same suffixes as notes: `_`, `=`, `.`, and suffix `-`. Octave markers (`'`, `,`) are not allowed on chord lines.

### Tie and slur groups

Parentheses work identically to notes lines. Spaces inside groups are ignored. Examples: `(1-6m-)`, `(1 - 6m -)`.

Example:

```
1 - 6m -
_1 _1 _1 =1 =1 1_ 6, (6_)
```

---

## Not-mentioned parts

When a part is **not mentioned** in a measure (no positional or `[Key]` lines cover it), its row is **not rendered** for that measure — the vertical space is reclaimed and rows below move up.

- A `follow[X]` part that is not mentioned copies `X`'s content (audio plays the same as X).
- A non-follow part that is not mentioned is filled with rests (`0`) or no-lyrics (`_`).
- All measures sharing a system line must render identical rows. A measure whose rendered shape differs starts a new system line.
- The first declared part is always considered mentioned (positional lines always fill it first).

### Omitted lines — fill table

| Situation | Result |
|-----------|--------|
| Part not mentioned; declared as `follow[X]` | Copies X's content; row suppressed |
| Part not mentioned; no follow target; notes/chord slot | Silently filled with rests (`0`) |
| Part not mentioned; no follow target; lyrics slot | Silently filled with no-lyrics (`_`) |
| `[Key]` line for first part's abbreviation | Error; line dropped |
| `[Key]` line with unrecognised abbreviation | Error; line dropped |
| More positional lines than first part's slots | Error; excess lines dropped |
| Fewer than one data line per group | Error |

**Example — part B not mentioned:**

```
# parts
A = chords
B = notes

# score
1 2m 3 4

1 - - -
1 2 3 4
```

Measure 1: A plays `1 2m 3 4`, B is not mentioned → filled with rests, row suppressed.
Measure 2: A plays `1 - - -`, B plays `1 2 3 4`.

---

## Quick reference — special line forms

| Whole line | Column | Meaning |
|------------|--------|---------|
| `_` | lyrics only | No lyrics this bar |
| *(omitted)* | any | Rest fill or follow-target copy; row suppressed |
| `(...)` | directive | Global bpm/key/time/label for this bar |
| `[Abbrev] <content>` | notes, lyrics, chord | Key-based line targeting the named part by abbreviation |

---

## Complete minimal example

```
# metadata
title = "Demo"
author = "Author"

# parts
Melody [M] = notes+lyrics
Harmony [H] = follow[M]

# score

(bpm=120 key=C4 time=4/4 label="Verse")
1 - 4m 5
do re mi fa

1 - 4m 5
_
[H] 3 - 6m 7
[H] do re mi fa
```

Bar 1: Melody plays `1 - 4m 5` / `do re mi fa`. Harmony is not mentioned → copies Melody, row suppressed.  
Bar 2: Melody plays `1 - 4m 5` / `_` (no lyrics). Harmony uses `[H]` key lines to override both slots.

---

## Further reading

Design specs with additional rationale live in `docs/superpowers/specs/`:

- `2026-06-04-interleaved-syntax-design.md` — interleaved `# score` format
- `2026-06-05-label-directive-design.md` — `label=` directive
- `2026-06-06-chord-track-design.md` — `chord:` columns
