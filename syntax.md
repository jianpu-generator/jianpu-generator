# Jianpu Generator — `.jianpu` Syntax Reference

This document describes the input syntax accepted by **jianpu-generator** as implemented today. File extension: `.jianpu`.

---

## File structure

A `.jianpu` file has up to five sections, which may appear in any order:

```
# metadata
…key = value fields…

# parts
…track declarations…

# groups
…group alias declarations…

# sequence
…comma-separated section labels…

# score
…interleaved score content…
```

- `# metadata` — **optional**
- `# parts` — **required**
- `# groups` — **optional**
- `# sequence` — **optional**
- `# score` — **required**
- Sections may appear in any order.
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
| `title_font_size` | no | `row_height * 1.5` | Font size of the title (points) |
| `author` | no | none | Author name (rendered in header) |
| `author_font_size` | no | `row_height * 0.6` | Font size of the author line (points) |
| `subtitle` | no | none | Subtitle line |
| `subtitle_font_size` | no | `row_height * 0.8` | Font size of the subtitle (points) |
| `max_measures_per_system` | no | `4` | Maximum number of measures per system line before wrapping |
| `row_height` | no | `24` | Vertical spacing of one part row (pixels) |
| `note_number_width` | no | `8` | Horizontal space per note column (pixels) |
| `part_label_width_pt` | no | `40` | Fixed width (points) of the part-label column at the start of each system, shared by every system in the score regardless of how many measures/columns that system's music needs |
| `parts_list_columns` | no | `4` | Number of columns in the parts list header |
| `part_legend_font_size` | no | `row_height * 0.6` | Font size of the part-name legend entries shown in the header (points) |
| `lyrics_font_size` | no | `row_height * 0.6` | Font size of lyric syllables (points) |
| `notes_font_size` | no | `lyrics_font_size` | Font size of note heads, rests, percussion hits, and tuplet brackets (points) |
| `chords_font_size` | no | `lyrics_font_size` | Font size of chord symbols (points) |
| `sequence_font_size` | no | `12` | Font size of the `# sequence` summary line rendered near the top of the score (points) |
| `merge_duplicate_measures_across_parts` | no | `yes` | Score-wide default for whether identical measures from different parts are merged into a single row (`yes`/`no`); can be overridden from a given measure onward with the `merge_duplicate_measures_across_parts=` directive line — see [Directive lines](#directive-lines) |
| `hide_resting_parts` | no | `yes` | Score-wide default for whether an all-rest part is omitted from a measure where other parts have content (`yes`/`no`); can be overridden from a given measure onward with the `hide_resting_parts=` directive line — see [Directive lines](#directive-lines) |
| `hide_system_dividers` | no | `no` | Whether the horizontal divider line between systems is omitted (`yes`/`no`) |
| `directive_row_offset` | no | `0 0` | Translation `"x y"` (points) applied to every rendered directive row (bar number, section label, key, bpm, time signature), moving that row's text without affecting the layout of anything else. Not applied to the `# sequence` summary header. |

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
| `notes+lyrics` | Notes + lyrics | notes, then 1 or more lyric-verse lines |
| `percussion` | Unpitched GM drum hits | 1 |
| `lyrics` | Lyrics-only, adurational | 1 or more lyric-verse lines |
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

## Groups section

An optional `# groups` section — placed after `# parts` and before `# score`.

```
# groups
Soprano [s] = s1 s2
Alto [a] = a1 a2
```

- Same left-hand-side syntax as `# parts`: `<display-name> [<abbreviation>] = <members>`; when brackets are omitted, the abbreviation equals the display name.
- The right-hand side is a space-separated list of member abbreviations.

Each group with an explicit abbreviation (i.e. `[abbreviation]` differs from the display name) is listed in the part-list legend in the SVG/PDF output, alongside part entries. A group is hidden from the legend when a track filter excludes all of its members.

A group's abbreviation may also be used as a `[GroupAbbrev]` key prefix in `# score` to broadcast a line to all of its members at once — see [Key-based part prefix](#key-based-part-prefix-abbrev) below. This requires every resolved member to share the same part kind; a group whose members don't (or whose abbreviation collides with a part's) can still appear in the legend but cannot be used as a score key.

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

1. **Optional directive line** — first line containing at least one directive keyword (`bpm=`, `key=`, `time=`, `label=`, `merge_duplicate_measures_across_parts=`, or `hide_resting_parts=`)
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

`[Abbrev]` may also name a `# groups` abbreviation, broadcasting that line to every part the group resolves to (expanding nested groups transitively):

```
# groups
Soprano [s] = S1 S2

# score
bpm=92 key=C4 time=4/4
[s] 5_ 5_ 5_ 5=
[S2] 6_ 6_ 6_ 6=
```

Here `S1` gets `5_ 5_ 5_ 5=` from the group broadcast; `S2` has its own `[S2]` line, which wins over the broadcast for that slot. Rules:

- Multiple `[GroupAbbrev]` lines fill slots in occurrence order, same as a part key — the group's first line fills every member's first slot, the second line fills every member's second slot, and so on.
- A member's own `[MemberAbbrev]` line always takes precedence over the group broadcast for that slot, regardless of which appears first in the file.
- A group is only usable this way if all of its resolved members share the same part kind (`notes`, `chords`, `notes+lyrics`, `percussion`, or `lyrics`) and its abbreviation does not collide with any part's abbreviation; otherwise the group is invalid and using it as a key produces the same "unrecognised abbreviation" error as an unknown key.

**Row label when members render as one unison row:** when two or more members' compiled content ends up identical (typically because they all took the unmodified group broadcast for that measure), the renderer already merges them into a single row. If every merged member traces to the same `[GroupAbbrev]` broadcast, that row is labeled with the **group's abbreviation** instead of the members' concatenated abbreviations. A member with its own overriding `[MemberAbbrev]` line never merges into that row (it keeps its own row, labeled with its own abbreviation), even if the override happens to be one of two members left in the group:

```
# parts
Soprano 1 [S1] = notes
Soprano 2 [S2] = notes
Soprano 3 [S3] = notes

# groups
Soprano [s] = S1 S2 S3

# score
[s] 1 2 3 4=
[S2] 5 5 5 5=
```

S1 and S3 both take the `[s]` broadcast unmodified and merge into one row labeled `s`; S2 overrides it and renders on its own row labeled `S2`.

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
| `merge_duplicate_measures_across_parts=` | `merge_duplicate_measures_across_parts=no` | Overrides the `#metadata` default from this measure onward (`yes`/`no`) |
| `hide_resting_parts=` | `hide_resting_parts=no` | Overrides the `#metadata` default from this measure onward (`yes`/`no`) |

Rules:

- Multiple directives may appear on one line, separated by whitespace.
- `label=` value must be a quoted string; empty labels are rejected.
- Directives apply to **all** parts. They are stored on the first notes part and propagate through grouping.
- `label` applies only to the measure where it is declared (does not persist to the next bar) — this is true for rendering purposes and whenever no `# sequence` section is present. When a `# sequence` section **is** present, each label additionally denotes a *span* of measures for playback-order purposes: see [`# sequence` — explicit playback order](#sequence--explicit-playback-order) below.
- `bpm`, `key`, and `time` persist until the next directive line overrides them.
- `merge_duplicate_measures_across_parts` and `hide_resting_parts` also persist until the next directive line overrides them; unset, they start from the `#metadata` value (or its default of `yes`) for the first measure.

### Rendering

When `time=` or `bpm=` changes on a measure, the generator may add a **directive row** above the bar-number / section-label row for that system line. Time signature and BPM appear once on that row (not on each part row), aligned with each measure’s note-start column. They do not shift notes or lyrics horizontally. If neither value changes on any measure in the line, the directive row is omitted.

Note names: `A` `B` `C` `D` `E` `F` `G`, with optional `#` or `b` accidental, followed by octave digit (e.g. `4`).

### `# sequence` — explicit playback order

A score may include an optional `# sequence` section — placed after `# parts` and before `# score` — that states the playback order directly, as a comma-separated list of section labels (the same labels set via `label="..."` on a measure's directive line):

```
# sequence
A, B, A

# score
time=4/4 key=C4 bpm=120 label="A"
1 2 3 4
label="B"
5 6 7 1
```

- Each entry in `# sequence` is a label declared with `label="..."` in `# score`; entries are separated by commas, and surrounding whitespace is trimmed.
- A label's **span** covers its measure and every following measure up to (but not including) the next `label="..."` measure, or through the end of the score if there is no following label. Above, `A` spans just its own measure, and `B` spans from its measure to the end of the score.
- Labels may be repeated in `# sequence` (e.g. `A, B, A`) to replay a span more than once.
- Each label must be declared **exactly once** in `# score`; declaring the same label on more than one measure is an error.
- Referencing a label in `# sequence` that was never declared in `# score` is an error; that entry is skipped and the rest of the sequence still resolves.
- `# sequence` only affects **MIDI/WAV playback order** — measures always render once, in written order, with normal bar numbers. However, SVG/PDF output does show the resolved order as a left-aligned line ("Sequence: A › B › A") on the first page, with a blank line of space above it, below the title/subtitle/author/part list. Each label is styled the same as an inline `label="..."` directive (bold, italic).

An entry may carry a `(-abbrev -abbrev ...)` suffix naming part or group abbreviations (as declared in `# parts`/`# groups`) to omit from that specific occurrence's playback — e.g. a chorus written once but replayed several times with a voice dropping out on later repeats:

```
# sequence
Verse, Chorus(-S -A2), Verse, Chorus(-A2), Chorus
```

- The suffix affects **only that occurrence**: here, the first `Chorus` omits Soprano and Alto 2, the second omits only Alto 2, and the third (unmarked) plays every part.
- Omitting a group abbreviation omits every part it resolves to (including transitively, through nested groups).
- An abbreviation that matches no declared part or group is an error; that abbreviation is dropped and the rest of the entry (and sequence) still resolves.
- The written-order rendering itself is unaffected — the score's written-out `Chorus` section always renders with every part, once, per the written-order rule above. However, the omissions **are** shown on the "Sequence: ..." summary line (SVG/PDF, first page), right after the label in plain (non-bold/non-italic) text: `Sequence: Verse › Chorus (-S -A2) › Verse › Chorus (-A2) › Chorus`. This is a reader-facing note only, telling a performer which voices tacet on which repeat — the underlying `Chorus` measures are not duplicated or altered.
- A group abbreviation is shown **as written** on the summary line, not expanded to its members: `Chorus(-U)` renders as `Chorus (-U)`, not `Chorus (-S -A2)`, even though playback omits every part `U` resolves to.

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

### Tuplets

`N:{notes}` brackets `N` notes to be played in the time normally taken by a standard "against" count (3-in-2, 2-in-3, 5-in-4, ...); `N:M:{notes}` overrides that with an explicit `M`. The brace opens right after the count, not before it — `3:{1_1_1_}`, not `{3:1_1_1_}`.

```
3:{1_1_1_} 2_ 3_ 4_ 5_ 6_    an eighth-note triplet, then five plain eighth notes
5:4:{1=1=1=1=1=}             a quintuplet of sixteenth notes, explicit 5-in-4
```

| `N` | Implied `M` (against count) |
|-----|------------------------------|
| 2 | 3 |
| 3 | 2 |
| 4 | 3 |
| 5 | 4 |
| 6 | 4 |
| 7 | 4 |
| 9 | 8 |

Any other `N` has no standard implied ratio — omitting `:M` is a recoverable error ("tuplet ratio for N is ambiguous; use `{N:M:...}` to specify explicitly"); write `N:M:{notes}` instead.

The bracket must contain exactly `N` notes/rests/repeat-atoms (each counts once, same rule as `(…)` group note-counting) — a mismatch at the closing `}` is a recoverable error, though the notes present are still emitted and rendered.

Tuplets nest with `(…)` slur/tie groups in either direction:

```
(3:{1_1_1_} 2_) 3_ 4_ 5_ 6_    slur group wrapping a triplet
3:{(1_1_) 1_} 2_ 3_ 4_ 5_ 6_   triplet wrapping a slur group
```

Unlike `(…)` groups, a tuplet **cannot span lines**: an unclosed `{` at the end of a line is a hard parse error, not a cross-line continuation.

**Note:** the measure-capacity check (below) currently compares each tuplet's *written* (nominal, uncompressed) duration against the bar, not its actual rescaled duration — so a tuplet that only fits the bar once compressed/expanded (the whole point of writing one) can be misjudged as too short or too long at parse time. The triplet example above works because its notes' nominal durations, ignoring the tuplet, already sum to the bar's capacity on their own. Until this is fixed, keep a tuplet's *nominal* duration matching what the bar needs, or use it as a measure's only content.

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
| `-` | Extend the previous **note or rest** by one beat (4 quarter-beats) |
| `-.` | Extend the previous **note or rest** by one *dotted* beat (6 quarter-beats) — the natural beat of a compound meter (e.g. 9/8) |
| `~` | Tie this note to the next note (same pitch and octave required) |

Example: `2 - - -` is a whole note in 4/4 (equivalent to `2---`). Likewise, `0 - - -` (or `0---`) is a whole rest.

`-.` is a standalone extension atom (the `.` must be glued directly after the `-`, with no space) — it is not the same as a `-` suffix followed by a separate dotted note. In 9/8, `1. -. -.` is a note held across the full measure: a dotted quarter (`1.`, 6 quarter-beats) plus two dotted-beat extensions (6 + 6), totaling 18 quarter-beats.

You can also attach dashes as suffixes on a note or rest (`2---`, `0---`). Both forms may be mixed in one measure. Repeated rests (`0 0`, `0 0 0 0`) remain equally valid — `0---` and `0 0 0 0` both produce a whole rest in 4/4.

Shorter rests still use `_`, `=`, or `.` on a single `0` (`0_`, `0=`, `0.`).

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

(e.g. 4/4 → 16, 3/4 → 12). Too many quarter-beats is a parse error. A shortfall extends the last note or rest when possible (so a lone `0` filling an empty measure is equivalent to `0---`). Otherwise it is a parse error.

#### Grouping validation (4/4 only)

In 4/4, the parser rejects rhythm spellings that cross metrical boundaries without exposing the split:

1. **Half-bar boundary:** after beat 1, no single note/rest may span from before beat 3 into beat 3 or beyond (quarter-beat position 8). Use a beam group such as `(2_ 2_)` or a tie instead of a single long value (e.g. `1. 2. 3_ 4_` is invalid; `1. (2_ 2_) 3_ 4_ 0_` is valid). Long notes/rests starting on beat 1 (including a fully extended `1` or `1---`) are allowed.
2. **Dotted-eighth tail:** a dotted eighth note/rest at the start of a beat must be followed immediately by a sixteenth note/rest filling the remaining sixteenth (e.g. `1_. 2= 3_ …`); `1_. 2_ 3_ 4_` is invalid (`2_.` is a dotted eighth, not an eighth).

Other time signatures skip these checks for now. Violations are diagnostics attached to the note (half-bar-boundary crossing is a **warning**; the dotted-eighth-tail rule is a **recoverable error**) — the file still renders.

### Multi-measure rests

This isn't new input syntax — it's automatic rendering behavior. When 2 or more consecutive measures are entirely rests (on every currently-visible part, after any `--tracks` filtering) and none of them carries its own directive (label, navigation marker, time signature/BPM/key change) or diagnostic, they render as a single wide rest bar showing the collapsed measure count, instead of one rest measure per bar. A single isolated all-rest measure still renders normally.

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

### Multiple verses

A `notes+lyrics` part can carry more than one lyric line per measure. Every consecutive `[Part]` line that follows the notes line, up to the next part's line or the end of the measure, is a separate verse, in order (verse 1, verse 2, …):

```
[Melody] 1 2 3 4
[Melody] a b c d
[Melody] one two three four
```

Each verse renders as its own row directly under the notes row, in verse order, and each verse is tallied and tie-paired against the notes row independently — a verse can have its own `-` held syllables and `_` no-lyrics marker.

The number of verse lines is per-measure: one measure can have one verse while the next has two. A part's verse count changing from one measure to the next always starts a new system at that measure boundary, regardless of how much horizontal space is left on the current line — verses can't silently appear or disappear mid-system.

### Standalone `lyrics` parts

A `lyrics`-kind part (see [Right-hand side](#right-hand-side)) is lyrics-only and **adurational** — it has no notes of its own, so nothing to tie syllables to. Every `[Abbrev]` line for it is a full verse line, not a stream of per-note syllables: the whole line renders as **one** left-aligned text block spanning the entire measure's width, however many columns that measure's other parts need.

```
# parts
Melody [M] = notes
Caption [C] = lyrics

# score
[M] 1 2 3 4
[C] a caption for this measure, unrelated to any note
```

- Unlike `notes+lyrics`, there is no leading notes line to pair against — every consecutive `[Abbrev]` line is itself a verse (verse 1, verse 2, …), the same way extra `notes+lyrics` verse lines work.
- `tokenize_lyrics`' word/CJK-character splitting still applies, but only to decide the rendered text (syllables are rejoined with spaces) — a `lyrics` part has no per-syllable columns, no `-` held-syllable semantics, and no lyrics–notes tally check.
- A wide `lyrics` line can widen its measure past what the other parts' notes alone would need, since the block competes for the measure's total pixel width even though it never affects the measure's column *count*.

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
<triad>      ::= "m" | "o" | "+" | "sus2" | "sus4" | "sus"
<extension>  ::= "M7" | "7"
<bass>       ::= <degree> <accidental>?
```

Parsing checks longest suffix first (`M7` before `7`; `sus2`/`sus4` before bare `sus`; `m` before extension). Bare `sus` (no digit) means `sus4`, matching standard chord-chart convention.

| Input | Meaning |
|-------|---------|
| `1` | I major |
| `1m` | I minor |
| `1o` | I diminished |
| `1+` | I augmented |
| `1sus2` | I suspended 2nd |
| `1sus4` | I suspended 4th |
| `1sus` | I suspended 4th (alias for `1sus4`) |
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

When a part is **not mentioned** in a measure (no `[Key]` line covers it), it is filled with rests (`0`) or no-lyrics (`_`). If, after filling, that part's row is all rests for the measure **and at least one other part in the same measure has real content**, the row is **not rendered** for that measure — the vertical space is reclaimed and rows below move up. This suppression is controlled by the `hide_resting_parts` metadata field (default `yes`); set it to `no` to always render every part's row, even when it's all rests.

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
