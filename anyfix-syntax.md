# Poentaro in anyfix expressions (draft)

A draft of a second surface syntax for Poentaro documents. It is written in
[anyfix expressions](https://github.com/wongjiahau/anyfix-expr) and saved
with the `.anyfix` extension. The current syntax (`syntax.md`) is still the
only one the web app reads. This one is CLI-only for now.

**What it changes.** Metadata, parts, sequence, presets and score directives
become anyfix data. They are named fields, real lists and nested records,
in place of the hand-tokenized `key = value`, positional and
`(-abbrev)` forms.

**What it keeps.** Notes, lyrics and chords stay as text in raw strings,
read by the existing score parser. Their syntax collides with anyfix's
tokens: `,` is the only separator, `.` and `:` are operators, and `#`
always starts a comment.

## Example

```text
# A comment runs from `#` to the end of the line.

metadata {
  title : "Part options",
  title style : { font size : 32, bold : yes },
  max measures per system : 1,
},

parts [
  "Piano" [P]  : notes { sound : 1, volume : 80%, octave : +1 },
  "Echo" [E]   : follow [P] { volume : 50% },
  "Violin" [V] : notes { instrument : violin },
  "Words" [Pv1] : lyrics [P],
],

sequence [ Verse, Chorus WITHOUT [E], "Last chorus" ONLY [P] ],

preset "Piano only" { visible parts : [P], notation : numbered },

score [
  { bpm : 100, key : D4, time : 3/4, label : "Verse" },
  """
  [P] 1 2 3
  [Pv1] do re mi

  [P] 4# 5 6,      # the comma and the hash are note syntax here
  """,
  { key : +2, label : "Chorus", break },
  """
  [P] 1 2 3
  """,
],
```

## Grammar

The grammar is written in anyfix expressions, read the way anyfix-expr's
README reads its own grammar: `name : …` is a rule, `|` is alternation,
`[ ]` is optional, `{ }` is repetition and `"…"` is literal text. A rule
names the abstract tree anyfix-expr produces, not raw characters, so
`key : value` here means the call `key _` of `value`.

```text
document   : { section "," },
section    : metadata | parts | sequence | preset | score,

# ── metadata ────────────────────────────────────────────────────────────
metadata   : "metadata" "{" { field "," } "}",
field      : key ":" value,
key        : word { word },                 # `max measures per system`
value      : string | number | word { word } | sign number
           | "(" value "," value ")"        # directive row offset : (0, -4)
           | "{" { field "," } "}",         # a text style

# ── parts ───────────────────────────────────────────────────────────────
parts      : "parts" "[" { part "," } "]",
part       : display [ "[" abbreviation "]" ] ":" kind,
display    : string,                        # the abbreviation, too, when no [ ] follows
kind       : ( "notes" | "chords" | "percussion" ) [ options ]
           | "follow" "[" abbreviation "]" [ options ]
           | "lyrics" "[" abbreviation "]",
options    : "{" { option "," } "}",
option     : "instrument" ":" word          # violin, soprano, …
           | "sound" ":" number             # GM program, or GM drum key on percussion
           | "volume" ":" number "%"
           | "octave" ":" sign number,
abbreviation : string | ? the source text of the element, as written ?,

# ── sequence ────────────────────────────────────────────────────────────
sequence   : "sequence" "[" { entry "," } "]",
entry      : label [ ( "WITHOUT" | "ONLY" ) "[" { abbreviation "," } "]" ],
label      : string | word,                 # a word that is not all caps

# ── presets ─────────────────────────────────────────────────────────────
preset     : "preset" string "{" { preset_field "," } "}",
preset_field : ( "visible" "parts" | "soloed" "parts" ) ":" "[" { abbreviation "," } "]"
             | "notation" ":" word
             | field,                       # any metadata field

# ── score ───────────────────────────────────────────────────────────────
score      : "score" "[" { element "," } "]",
element    : directives | measures,
directives : "{" { directive "," } "}",     # applies to the next measure
directive  : "bpm" ":" number
           | "key" ":" ( word | string | sign number )   # C4, Bb3, "F#3", +2
           | "time" ":" number "/" number
           | "label" ":" string
           | "merge" "duplicate" "measures" "across" "parts" ":" ( "yes" | "no" )
           | "hide" "resting" "parts" ":" ( "yes" | "no" )
           | "break",
measures   : string,                        # legacy score text: `[Abbr] …` lines,
                                            # measures split by blank lines

sign       : "+" | "-",
```

## The tree each form produces

How anyfix-expr groups each form, and so what the consumer matches on:

| Source | Call name and arguments |
| --- | --- |
| `max measures per system : 4` | `max measures per system _` of `4` |
| `title style : { bold : yes }` | `title style _` of `Braces[bold _ of yes]` |
| `"Piano" [P] : notes { volume : 80% }` | `_ _ _` of `"Piano"`, `Brackets[P]`, `notes _` of `Braces[volume _ of (_ % of 80)]` |
| `"Melody" : notes` | `_ _` of `"Melody"`, `notes` |
| `follow [P]` | `follow _` of `Brackets[P]` |
| `octave : +1` | `octave _` of `+ _` of `1` |
| `time : 3/4` | `time _` of `_ / _` of `3`, `4` |
| `Chorus WITHOUT [A, T]` | `_ WITHOUT _` of `Chorus`, `Brackets[A, T]` |
| `preset "Solo" { … }` | `preset _ _` of `"Solo"`, `Braces[…]` |

## What gets better

- **Part options are named.** The old syntax tells them apart by shape:
  a quoted string is the soundfont, `N%` the volume, `+N` the octave
  offset, a bare word the instrument. `sound : 41` is a number, so the
  `"N: Name"` label (a cross-boundary invariant) disappears from the
  source. The lowering asks `sound_label` for it.
- **No line sniffing for directives.** The old parser decides a measure's
  first line is a directive line when any token looks like `bpm=` or
  `key=`. Here directives are a `{ … }` element of their own. `time : 3/4`,
  `key : +2` and `key : C4` arrive already split by anyfix-expr.
- **`WITHOUT` and `ONLY` can't be mixed.** The old `Chorus(S -A2)` is a
  runtime error. Here the two filters are different call shapes, so the
  mix can't be written. A label can be any string.
- **One assignment syntax.** The old syntax uses `key = value` outside a
  style object and `key: value` inside one, with `title` overloaded for
  both the text and its style. Here everything is `key : value`, and the
  style is `title style`.
- **Multi-word keys.** Keys use anyfix runs (`max measures per system`)
  rather than snake_case.
- **Lists are lists.** `visible parts : [main, S1]` can hold an
  abbreviation with a comma, written as a string.
- **Comments everywhere.** `#` comments are kept as anyfix trivia, so a
  formatter keeps them. Inside a score string, the old `//` comments still
  apply.

## Gotchas

- **All-caps words are keywords.** `S1`, `M` and `A2` split a run, so
  `follow P` would be the call `_ P` of `follow`. Every part reference is
  therefore written in brackets: `follow [P]`, `lyrics [M]`,
  `WITHOUT [A]`. A label that is all caps is written as a string:
  `"A" WITHOUT [S]`.
- **`#` starts a comment.** A sharp key is written as a string:
  `key : "F#3"`.
- **Abbreviations are read as written.** An abbreviation is the source text
  between `[` and `]` (or between commas in a list), so `A1&T` works
  unquoted. Quote it if it contains `,`, `#`, `"`, a bracket, `.` or `:`.
- **Raw string closers.** A score line that starts with `"""` needs the
  string opened with `""""`. The converter does this automatically.

## Status of the prototype

- `poentaro convert anyfix <file.poentaro>` writes `<file>.anyfix` next to
  the input.
- Every CLI command accepts an `.anyfix` file. It is lowered to the current
  syntax, section by section, and then read by the existing parser
  (`src/parser/anyfix/`).
- `src/parser/anyfix/demo_svg_tests.rs` converts every `demo/*.poentaro`
  file and checks that the SVG pages are byte-for-byte equal.
- **No source map yet.** Errors from the existing parser point into the
  lowered text, not into the `.anyfix` file. Errors from the lowering
  itself do point into the `.anyfix` file. Mapping the rest back needs
  `StringLiteral::source_offset` threaded through the score parser.
- **The embedded source is the lowered text.** An SVG or PDF made from an
  `.anyfix` file embeds the lowered current-syntax document, not the
  `.anyfix` source. Apart from that `<metadata>` element, the pages are
  byte-for-byte equal.
- **Not converted:** `//` comments in `# metadata`, `# parts` and
  `# sequence` are dropped. Score comments are kept. A `# parts` line the
  current parser rejects is kept as a `# could not convert:` comment.
- **Labels in `# sequence`.** A label containing `,`, `(` or `)` can't be
  lowered yet, because the current `# sequence` syntax can't hold it.
