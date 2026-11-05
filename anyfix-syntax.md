# Poentaro in anyfix expressions

A surface syntax for Poentaro documents, written in
[anyfix expressions](https://github.com/wongjiahau/anyfix-expr). The web app
and the CLI read it, and tell it from the section syntax (`syntax.md`) by
content.

**What it changes.** Metadata, parts, sequence and presets
become anyfix data. They are named fields, real lists and nested records,
in place of the hand-tokenized `key = value`, positional and
`(-abbrev)` forms.

**What it keeps.** The score is one raw string holding the `# score` body
exactly as `syntax.md` describes it: directive lines, notes, lyrics, chords,
blank lines and `//` comments, read by the existing score parser. Their syntax collides with anyfix's
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

score """
  bpm=100 key=D4 time=3/4 label="Verse"
  [P] 1 2 3
  [Pv1] do re mi

  [P] 4# 5 6,      // the comma and the hash are note syntax here

  key=+2 label="Chorus" break
  [P] 1 2 3
  """,
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
abbreviation : string | ? words, numbers and symbols, joined without spaces ?,

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
score      : "score" string,                 # the `# score` body of syntax.md

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
| `Chorus WITHOUT [A, T]` | `_ WITHOUT _` of `Chorus`, `Brackets[A, T]` |
| `preset "Solo" { … }` | `preset _ _` of `"Solo"`, `Braces[…]` |

## What gets better

- **Part options are named.** The old syntax tells them apart by shape:
  a quoted string is the soundfont, `N%` the volume, `+N` the octave
  offset, a bare word the instrument. `sound : 41` is a number, so the
  `"N: Name"` label (a cross-boundary invariant) disappears from the
  source. The lowering asks `sound_label` for it.
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
- **`#` starts a comment.** Outside the score string, a sharp key is written
  as a string: `key : "F#3"`. Inside it, `key=F#3` is score text as usual.
- **Abbreviations are read from the tree.** The words, numbers and symbols
  of an abbreviation are joined without spaces, so `A1&T` works unquoted
  and still reads `A1&T` after the formatter spaces it out as `A1 & T`.
  Quote one that contains whitespace, `,`, `#`, `"`, a bracket, `.` or `:`.
- **Raw string closers.** A score line that starts with `"""` needs the
  string opened with `""""`. The converter does this automatically.

## Tooling

- `poentaro convert-anyfix <file.poentaro>` writes `<file>.anyfix` next to
  the input, in anyfix-expr's canonical layout.
- `poentaro format <file>` lays the document out with anyfix-expr's
  formatter, then formats the score string the way the current syntax's
  `# score` is formatted (redundant lines dropped, parts sorted, whitespace
  collapsed). A score string with a `//` comment is left alone.
- **Canonical form.** The formatter drops every `:` it doesn't need and
  spaces out symbols, so `title : "Song"` becomes `title "Song"` and
  `volume : 80%` becomes `volume: 80 %`. Both spellings are the same tree,
  so both read the same. The examples above keep the `:` for readability.
- The web app and every CLI command read this syntax by content, whatever
  the file is called. A document is lowered to the section syntax, section by
  section, and then read by the existing parser (`document::Document`,
  `src/parser/anyfix/`). Section syntax files still load.
- `demo/*.poentaro` and `new_file_template.poentaro` are written in this
  syntax. `src/parser/anyfix/demo_svg_tests.rs` converts every
  `fixtures/section-syntax-demos/*.poentaro` file and checks that the SVG pages
  are byte-for-byte equal.
- **Source map.** `parser::anyfix::SourceMap` maps positions between the
  document and its lowered form. Every score line maps byte for byte, so
  errors, selections and highlights land on the character.
- **Not converted:** `//` comments in `# metadata`, `# parts` and
  `# sequence` are dropped. Score comments are kept. A `# parts` line the
  current parser rejects is kept as a `# could not convert:` comment.
- **Labels in `# sequence`.** A label containing `,`, `(` or `)` can't be
  lowered yet, because the current `# sequence` syntax can't hold it.
