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

- `# metadata` — **optional**
- `# parts` — **required**
- `# score` — **required**
- Sections must appear in the order above.
- Legacy `# score:Name` / `# lyrics:Name` sections are **not** supported.

Whitespace around `=` in metadata is optional. Metadata values may be quoted with `"`.

---

## Comments

`//` starts a comment that runs to the end of the line. It is recognized anywhere in the file — in the metadata, parts, or score sections, on its own line or trailing other content.

```
# metadata
title = "My Song"  // shown in the header

// this whole line is a comment
author = "Jane Doe"
```

A `//` inside a double-quoted string (e.g. `title = "http://example.com"`) is not treated as a comment.

---

## Metadata

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `title` | no | none | Piece title (rendered in header) |
| `author` | no | none | Author name (rendered in header) |
| `subtitle` | no | none | Subtitle line |
| `max columns` | no | `28` | Maximum grid columns per system line before wrapping |
| `row height` | no | `24` | Vertical spacing of one part row (pixels) |
| `label width` | no | `40` | Horizontal space reserved for part labels (pixels) |
| `note number width` | no | `8` | Horizontal space per note column (pixels) |
| `parts list columns` | no | `4` | Number of columns in the parts list header |

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
| `percussion` | Unpitched GM drum hits | 1 |
| `follow[X]` | Inherit column layout from the part with abbreviation `X` | same as target |

An optional soundfont string `"<number>: <name>"` may follow the kind token (or `follow[X]` bracket) to select the MIDI timbre for that part. The number is the General MIDI program number (0–127). The `<name>` portion is a quoted string and may contain `=` and other characters (for example `"1: Grand = Piano"`). For example: `notes "52: Choir Aahs"` or `follow[A] "1: Grand Piano"`. If omitted on a concrete part, the default is program 52 (Choir Aahs). On a `follow[X]` part, the soundfont is inherited from the target when omitted.

For `percussion` parts, the soundfont number is instead a **GM percussion key** (e.g. `38` = Acoustic Snare, `36` = Bass Drum 1), not a GM program number — all percussion parts share MIDI channel 9 (the GM drum channel) and a single fixed GM Standard Kit program change; the number selects which drum sample within that kit each hit plays. The number is not checked against the melodic instrument catalog.

An optional volume suffix `XX%` (1–3 ASCII digits followed by `%`, parsed as an unsigned 8-bit number; values above 100 or 0 are accepted without error or clamping) may appear after the soundfont string (or after the kind token if there is no soundfont) to set the MIDI volume for that part. For example: `notes "52: Choir Aahs" 47%` or `notes 80%`. If omitted on a concrete part, the default is 100%. On a `follow[X]` part, volume is inherited from the target when omitted and may be overridden with an explicit `XX%` suffix.

An optional octave offset `+N` or `-N` (where N is 1–4) may appear anywhere on the right-hand side to shift every note in that part up or down by N octaves in MIDI output only. For example: `notes -1`, `notes+lyrics +1`, `notes "5: Electric Guitar" -2`, or `follow[A] -1`. The offset does not change octave dots in the rendered SVG. If omitted on a concrete part, the default is 0. On a `follow[X]` part, the octave offset is inherited from the target when omitted and may be overridden with an explicit `+N` or `-N` suffix. Values outside ±4 emit a recoverable error and are clamped to ±4.

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
bpm=92 key=C4 time=4/4 label="Verse 1"
[Melody] 5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_)
[Melody] 白陽旗旛在大道盛宏

[Melody] 3_ (1_1) 0_- 1= 1=
[Melody] 昌花花
```

### Group layout

1. **Optional directive line** — first line containing at least one directive keyword (`bpm=`, `key=`, `time=`, or `label=`)
2. **Data lines** — every data line **must** begin with a `[Abbrev]` prefix (see below); there are no unprefixed/positional lines

Lines are trimmed; leading/trailing spaces on a line are ignored. A completely empty line separates measure groups (it is not a data line).

### Key-based part prefix (`[Abbrev]`)

Every data line must begin with `[Abbrev]` to route it to a specific part by abbreviation, including the first declared part:

```
[A2] 5 6 7 0
```

- A data line with no `[Abbrev]` prefix is a recoverable error; the line is dropped.
- Any number of `[Key]` lines may appear for the same part; they fill that part's slots in declaration order (first line → first slot, second line → second slot, …).
- An unrecognised abbreviation is an error; the line is dropped.
- Parts not covered by any `[Key]` line use their `follow[X]` target's content when declared as such, or are filled with implicit rests/no-lyrics otherwise.
- A measure group with zero valid keyed lines is an error (`measure_no_data_lines`).

**Example — only part C plays, A and B are not-mentioned:**

```jianpu
# parts
A = notes
B = notes
C = notes

# score
time=4/4 key=C4 bpm=120
[A] 1 2 3 4

[C] 5 6 7 0
```

Measure 2: C plays `5 6 7 0`. A and B have no explicit lines → filled with `0` (rest) and marked not-mentioned (rows suppressed).

**Example — key-based lines in one measure with a follow part:**

```jianpu
# parts
A = notes
B = follow[A]
C = notes

# score
[A] 1 2 3 4
[C] 5 6 7 0
```

A: `1 2 3 4`. B: not mentioned → copies A's content via `follow`. C: `5 6 7 0`.

**Example — follow part with partial key override:**

```jianpu
# parts
Soprano [S] = notes+lyrics
Alto [A] = follow[S]

# score
time=4/4 key=C4 bpm=120
[S] 1 2 3 4
[S] do re mi fa
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
| `dcalcoda` | `dcalcoda` | D.C. al Coda: after this measure, playback restarts from measure 0 |
| `tocoda` | `tocoda` | To Coda: on the second pass only, playback cuts away here to the `coda` measure |
| `coda` | `coda` | Coda: playback resumes here (on the second pass) and continues to the end |
| `segno` | `segno` | Segno: marks the measure that `dsalcoda` jumps back to |
| `dsalcoda` | `dsalcoda` | D.S. al Coda: after this measure, playback restarts from the `segno` measure |

Rules:

- Multiple directives may appear on one line, separated by whitespace.
- `label=` value must be a quoted string; empty labels are rejected.
- Directives apply to **all** parts. They are stored on the first notes part and propagate through grouping.
- `label` applies only to the measure where it is declared (does not persist to the next bar).
- `bpm`, `key`, and `time` persist until the next directive line overrides them.
- `dcalcoda`, `tocoda`, and `coda` are bare keywords (no `=value`) that, like `label`, apply only to the measure where declared and do not persist. They must appear **all three together or not at all** (a partial set is an error), at most once each, and `tocoda` must occur before `coda`.
- `segno`, `dsalcoda`, `tocoda`, and `coda` are the equivalent "D.S. al Coda" marker set: they must appear **all four together or not at all**, at most once each, `segno` must occur at or before `dsalcoda`, and `tocoda` must occur before `coda`. `dcalcoda` cannot be combined with `segno`/`dsalcoda` in the same score — pick one navigation scheme.

### Rendering

When `time=` or `bpm=` changes on a measure, the generator may add a **directive row** above the bar-number / section-label row for that system line. Time signature and BPM appear once on that row (not on each part row), aligned with each measure’s note-start column. They do not shift notes or lyrics horizontally. If neither value changes on any measure in the line, the directive row is omitted. A measure with `dcalcoda`, `tocoda`, `coda`, `segno`, or `dsalcoda` set also forces a directive row for that measure, even without a label.

Note names: `A` `B` `C` `D` `E` `F` `G`, with optional `#` or `b` accidental, followed by octave digit (e.g. `4`).

### D.C./D.S. al Coda navigation (SVG vs. MIDI/WAV)

`dcalcoda`/`tocoda`/`coda`/`segno`/`dsalcoda` render as annotations only — "D.C. al Coda" (italic), "⊕ To Coda", "⊕ Coda", "𝄋 Segno", and "D.S. al Coda" (italic) — on the measure where each is declared. **SVG (and PDF) output always shows measures in written order**; the markers are just text, they never reorder or duplicate anything visually.

**MIDI and WAV output actually replay measures according to the markers**, since this generator also produces playable audio:

- With `dcalcoda`/`tocoda`/`coda`: measures play from the start through the `dcalcoda` measure, then restart from the start and play through the `tocoda` measure, then jump to the `coda` measure and play through to the literal end of the score.
- With `segno`/`dsalcoda`/`tocoda`/`coda`: measures play from the start through the `dsalcoda` measure, then restart from the `segno` measure and play through the `tocoda` measure, then jump to the `coda` measure and play through to the literal end of the score.

On the first pass, the `tocoda` measure is just a normal measure — the cut only happens on the second pass.

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

### Accidentals (`#` / `b`)

Append `#` (sharp) or `b` (flat) immediately after a scale-degree digit to raise or lower the pitch by one semitone.

| Notation | Meaning |
|----------|---------|
| `7#`     | Scale degree 7, raised one semitone (leading tone sharpened) |
| `1b`     | Scale degree 1, lowered one semitone |
| `4#`     | Scale degree 4, raised one semitone (tritone) |

Accidentals can be combined with octave modifiers and all duration modifiers: `7#'` (sharp 7, octave up), `1b_` (flat 1, eighth note), `4#.` (sharp 4, dotted).

Rests (`0`) do not accept accidentals.

### Modifiers

| Suffix | Meaning |
|--------|---------|
| `.` | Dotted (add half the base duration). Cannot combine with `=` (sixteenth) notes. |
| `-` | Extend the previous **note** by one beat (4 quarter-beats) |
| `~` | Tie this note to the next note (same pitch and octave required) |

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

A group must contain **at least 2 notes** (counting notes across a cross-measure open/close). Single-note groups like `(5)` trigger a non-fatal **warning** (`group_too_few_notes`); rendering still proceeds.

### Tie (`~`)

`~` is written immediately after the octave modifier and before any duration modifiers:

```
4~---4---       tie two 4-beat notes
4'~4'           tie two high-4 quarter notes
4~.4.           tie quarter to dotted quarter
4~---4~---4---  chain of three tied notes
(4~---4--- 3)   tie inside a slur
```

Rules:
- Pitch, accidental, and octave must all match the next note — otherwise a recoverable error is emitted and the arc is suppressed.
- `~` on a rest is an error.
- `~` on the last note of the piece (no following note) is an error.
- Ties span freely across measure boundaries.
- Ties may appear inside slur groups `(…)`.

A tie differs from a slur `(…)` in that it requires identical pitch, and carries distinct semantic meaning (duration extension vs. phrasing).

### Repeat the last note/chord (`r`, bare `_`/`=`)

`r` repeats the last sounded pitch/chord as a fresh one-beat attack (a new note, not a tie/sustain). A bare `_` or `=` — one **not** glued directly after a digit — repeats it as an eighth-note or sixteenth-note attack respectively:

```
5 r r __        note 5, then three more 5s: a beat, a beat, and two eighths
5 0 r           note 5, a rest, then another beat of 5 (rests are skipped)
5~_             note 5 tied into its own eighth-note repeat
5__~5           note 5, an eighth-note repeat, tied out into the next note 5
```

Rules:
- "Last pitched note/chord" skips over intervening rests, and persists across measure boundaries (like ties/slurs).
- `r` never takes suffixes: `r_`, `r.`, `r'` are two atoms in sequence (`r` then a fresh atom), not `r` with a suffix glued on. Write repeats as multiple `r`s instead.
- Using `r`/`_`/`=` with no prior pitched note/chord on the track is a recoverable error; the token is dropped.
- A `~` glued directly after a repeat atom (`r`/`_`/`=`) ties that repeat into the following note, following the same rules as any other tie (matching pitch required, dangling tie is an error).

**Gotcha — maximal munch:** whitespace is cosmetic everywhere else in this grammar, but not here. `5_` (glued, no space) is unchanged: it's still note 5 shortened to an eighth note. Only a `_`/`=` that is *not* glued directly after a digit is a repeat atom — so `5 _` (with a space) repeats note 5's pitch as an eighth note, while `5_` does not. There are two exceptions, both because the glued character can't be read as a suffix of the preceding note in that position:
- Right after a tie: `5~_` glues fine (the `~` already claimed that spot).
- Right after another occurrence of the *same* suffix character: `5__` is note 5 shortened to an eighth note (first `_`) plus a repeated eighth-note attack (second `_`), not a no-op double-shorten. Likewise `5==` is a sixteenth note plus a repeated sixteenth, and this chains — `5___` is a note plus two repeats. Mixing different suffix characters still combines onto one atom as before: `5_=` is a single sixteenth note.

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

Other time signatures skip these checks for now. Violations are diagnostics attached to the note (half-bar-boundary crossing is a **warning**; the dotted-eighth-tail rule is a **recoverable error**) — the file still renders.

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
[Melody] 1 1 5 5
[Melody] twin- kle twin- kle     ← "twinkle" split across two notes each
```

This is distinct from a **standalone** `-` surrounded by whitespace (held syllable, below).

### Held syllable (`-` within lyrics)

A `-` **inside** a lyrics line marks the **preceding** syllable as *held* — it stretches across tied notes:

```
[Melody] he llo - world     ← "llo" is held across the tied note
[Melody] 你 - - 好           ← first 你 is held across two tied notes
```

This is distinct from `-` on a notes line (duration extension) and distinct from `_` (see below).

### No-lyrics marker (`_`)

A lyrics line whose **entire** trimmed content is `_` means **zero syllables** for that part in this measure (instrumental bar):

```
[Melody] 1 2 3 4
[Melody] do re mi fa

[Melody] 5 6 7 1
[Melody] _
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

Mismatch is a non-fatal **warning** (rendering continues, with empty-string syllables inserted for underflow), e.g. `[Soprano] lyrics underflow: ran out of syllables at syllable 3 (fewer syllables than notes)` or `[Soprano] lyrics overflow: 1 extra syllable(s) after all notes are consumed`.

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

### Repeating the last chord

`r` and bare `_`/`=` work the same way as on notes lines — see [Repeat the last note/chord](#repeat-the-last-notechord-r-bare-_) above: `1 r` repeats chord `1` for another beat, and `1 _` repeats it as an eighth note.

### Tie and slur groups

Parentheses work identically to notes lines. Spaces inside groups are ignored. Examples: `(1-6m-)`, `(1 - 6m -)`.

Example:

```
[chords] 1 - 6m -
[Melody] _1 _1 _1 =1 =1 1_ 6, (6_)
```

---

## Percussion syntax

Percussion lines carry unpitched GM drum hits. Duration works like notes: each token occupies one beat; `-` extends the previous hit.

| Token | Meaning |
|-------|---------|
| `0` | Rest |
| `x` | Hit |
| `-` | Extend previous hit one beat |

Duration suffixes (`_`, `=`, `.`), tie/slur groups (`(...)`), and the repeat-last-atom shorthand (`r`, bare `_`/`=`) work the same way as on notes lines — see [Notes syntax](#notes-syntax). Octave markers (`'`, `,`) and accidentals are not allowed on percussion lines, since hits have no pitch.

Example — snare and bass drum hitting simultaneously:

```
# parts
Snare = percussion "38: Acoustic Snare"
Kick = percussion "36: Bass Drum 1"

# score
[Snare] 0 x 0 x
[Kick] x 0 x 0
```

---

## Not-mentioned parts

When a part is **not mentioned** in a measure (no `[Key]` line covers it), its row is **not rendered** for that measure — the vertical space is reclaimed and rows below move up.

- A `follow[X]` part that is not mentioned copies `X`'s content (audio plays the same as X).
- A non-follow part that is not mentioned is filled with rests (`0`) or no-lyrics (`_`).
- All measures sharing a system line must render identical rows. A measure whose rendered shape differs starts a new system line.

### Omitted lines — fill table

| Situation | Result |
|-----------|--------|
| Part not mentioned; declared as `follow[X]` | Copies X's content; row suppressed |
| Part not mentioned; no follow target; notes/chord slot | Silently filled with rests (`0`) |
| Part not mentioned; no follow target; lyrics slot | Silently filled with no-lyrics (`_`) |
| Data line missing `[Abbrev]` prefix | Error; line dropped |
| `[Key]` line with unrecognised abbreviation | Error; line dropped |
| No valid keyed lines in a measure group | Error (`measure_no_data_lines`) |

**Example — part B not mentioned:**

```jianpu
# parts
A = chords
B = notes

# score
[A] 1 2m 3 4

[A] 1 - - -
[B] 1 2 3 4
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

```jianpu
# metadata
title = "Demo"
author = "Author"

# parts
Melody [M] = notes+lyrics
Harmony [H] = follow[M]

# score

bpm=120 key=C4 time=4/4 label="Verse"
[M] 1 2 4 5
[M] do re mi fa

[M] 1 2 4 5
[M] _
[H] 3 5 6 7
[H] do re mi fa
```

Bar 1: Melody plays `1 2 4 5` / `do re mi fa`. Harmony is not mentioned → copies Melody, row suppressed.  
Bar 2: Melody plays `1 2 4 5` / `_` (no lyrics). Harmony uses `[H]` key lines to override both slots.
