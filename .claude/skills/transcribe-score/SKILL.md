---
name: transcribe-score
description: Transcribe a real, public-domain musical score into this repo's .jianpu format — either starting a new file or adding a part/voice to one already transcribed — by finding the actual source and parsing it programmatically, instead of composing freehand. Use when asked to transcribe, continue, or add a part to a .jianpu file "from the real score" / "from the original" / by composer+opus, or when a requested part doesn't exist in whatever source was already used for the file.
---

# Transcribing real scores into `.jianpu`

The goal is never to compose "in the style of" a piece — it's to find the
real score and transcribe it faithfully, matching whatever conventions the
file's already-transcribed measures established. If the file has no existing
measures yet, you're establishing those conventions from scratch and should
say so explicitly.

`scores/Air on G String.jianpu` (Bach, BWV 1068) is a worked example of this
whole procedure, including a part (`d`, second violin) added later from a
*different* source edition than the one used for the rest of the file — see
§0 and §1 below for why that came up and how it was resolved.

## 0. Decide what you actually need to find

- If the file has existing measures, treat them as ground truth for
  conventions — don't re-derive them, verify new material against them.
- If the requested part/voice **doesn't exist** in whatever source was used
  for the rest of the file (e.g. the file was transcribed from a reduction —
  flute + guitar — and the new part is "second violin"), don't compose it
  freehand and don't assume it's secretly in the existing source under a
  different name. Go find the edition that actually contains that voice —
  see §1.

## 1. Finding the source

- **Mutopia Project** (mutopiaproject.org) — public domain, LilyPond source
  always available (`piece-info.cgi?id=N` → download link), easiest to parse
  programmatically.
- **IMSLP** (imslp.org) — huge library, but mostly scanned PDFs. Check its
  "Sheet Music" section anyway (even if you already have a source from
  elsewhere) — it lists *every* edition/arrangement uploaded, including which
  ones ship non-scan engraving files (MusicXML, Capella, Finale, or a
  Mutopia-hosted LilyPond source) versus scan-only PDFs. Search for an
  edition whose editor/description mentions the instrumentation you need.
- A composer's work can have **multiple distinct catalog entries** for
  different arrangements of the same piece (e.g. a reduction vs. the
  original full scoring) — these are typically separate pieces/IDs, not
  files under one entry. A composer's catalog *listing* page can be
  incomplete; if a targeted web search turns up a piece-info id the listing
  didn't show, trust the direct id lookup.
- **IMSLP fetch gotcha**: don't trust `WebFetch`'s summary of an IMSLP work
  page — its HTML→markdown pass over these JS-heavy pages is unreliable
  (repeated fetches of the same URL can return different, sometimes
  hallucinated-looking summaries, including false claims that a section
  "wasn't in the provided content"). Fetch the raw wikitext instead and grep
  it yourself:
  ```
  curl 'https://imslp.org/api.php?action=parse&page=<Page Title>&format=json&prop=wikitext'
  ```
  extract `.parse.wikitext['*']`, then search it directly (e.g. for
  `musicxml`, an editor's name, or an instrument name).
- Always confirm public-domain/license status; prefer plain-text engraving
  markup (LilyPond, MusicXML) over image/PDF-only scores.

## 2. Parsing a LilyPond source

Hand-roll a small parser rather than reading rhythm by eye. Key rules:

- Strip markup that doesn't affect the note stream: comments (`%...`), grace
  notes (`\acciaccatura{...}`/`\appoggiatura{...}`), `\trill`. **Then also
  strip every remaining `\word` command generically**
  (`re.sub(r'\\[a-zA-Z]+', '', text)`) — an unstripped command like
  `\fermata` mis-tokenizes into spurious extra notes (its individual letters
  match the note-token regex, e.g. `f`,`e`,`r`→rest,`a`,`a`), silently
  inflating total duration with no explicit error. This only surfaces later
  as an unexplained cumulative-duration mismatch, so strip proactively.
- Tokenize pitch as `letter, accidental(is/es), octave-marks('/,),
  force-mark(!/?), duration, dots` — in that order. A force-mark regex placed
  before the octave-marks silently misparses.
- Compute absolute octave via LilyPond's `\relative` "nearest fourth" rule:
  each note's octave is the closest to the previous note's pitch, then
  adjusted by its own `'`/`,` marks. A chord's first listed note updates the
  reference for what follows.
- Convert pitch → movable-do scale degree relative to the piece's key.
- Track duration inheritance: a note with no explicit duration number
  inherits the last *explicitly written* duration (including inside chords).
- **Check whether the source needs repeat-unnesting.** A `\repeat volta N
  {...} \alternative {...}` wrapper sometimes lives only in a separate
  rhythm/skip "cue" voice, while the actual note voices already write every
  bar out flat and literal (including a physically-duplicated pair of bars
  for the two alternative endings). Detect this by comparing the two
  alternative-ending bars' text byte-for-byte: if identical (or
  near-identical), the voice is already a flat linear pass — just delete one
  copy of the duplicate bar before parsing, rather than reimplementing
  unnesting logic. If the endings differ or there's a genuine nested repeat,
  you do need to manually unnest into a linear pass, matching however the
  existing `.jianpu` file already handles repeats (usually: no repeats at
  all, just written out once).

## 3. Establishing alignment and scaling

1. Parse the already-transcribed portion of the source.
2. Find the duration-scaling factor between source and `.jianpu` measures
   (e.g. "1 real 4/4 bar = 2 jianpu measures", every duration doubles).
   Verify on a simple bar (a single held whole note scales to the expected
   measure count).
3. Confirm by matching scale-degree sequences (ignoring octave) across
   several consecutive bars, not just one. Minor discrepancies (an
   ornamental note added or dropped) are normal — note them, don't assume
   your parser is wrong.
4. **Bucket the continuous, un-barred note stream directly into
   jianpu-measure-sized windows** — i.e. `(real bar length) / scaling
   factor` units per bucket — rather than bucketing into real-bar-sized
   chunks and then figuring out how each bar's notes split across its N
   mapped measures. The jianpu measure grid does not respect real barlines,
   only cumulative duration; bucketing directly avoids a whole class of
   "these notes don't obviously split evenly" confusion.
5. When a note's duration straddles a bucket boundary: split at every
   jianpu-measure (16 quarter-beat) boundary it crosses, then, on each
   resulting per-measure fragment, apply the half-bar grouping-validation
   rule (no note/rest may start before quarter-beat 8 and cross it, unless
   it starts at 0 — see `syntax.md`). Chain tie marks through every fragment;
   only the final fragment carries the note's original tie-to-next flag.
   **Automate this check** (assert no fragment starts in `(0,8)` and ends
   past `8`) rather than eyeballing each measure — it scales correctly to
   pieces with dozens of measures.

## 4. Reverse-engineering conventions (don't assume, derive)

- **Melody**: usually literal (degree + scaled duration), occasionally
  dropping or adding a fast ornamental note versus the strict source — that
  is a known, acceptable liberty already present in this repo's
  transcriptions, not something to force onto a *new* part you're adding.
  Write new parts literally/mechanically unless there's a specific reason
  not to, and say so.
- **Octave notation**: don't assume marks track the source's absolute
  register — verify. If there's already a **verified part in the same file**
  covering the same bars from the same or a closely related source voice
  (e.g. this new part's counterpart, like Violin I when adding Violin II),
  derive the octave-mark rule as an explicit formula calibrated against it,
  instead of eyeballing renders note by note:
  1. `scaleposition(note) = letter_index(c=0,d=1,e=2,f=3,g=4,a=5,b=6) + 7 ×
     lilypond_relative_octave` (monotonic in pitch, +7 per real octave).
  2. From one note in the verified part with a known jianpu mark, solve for
     the tonic reference's scale position such that
     `floor((scaleposition − reference) / 7) == 0` matches "no mark".
  3. For any other note: band = `floor((scaleposition(note) − reference) /
     7)`. `0` → no mark, `>0` → that many `'`, `<0` → that many `,`.
  4. Verify against 2–3 more already-written measures before trusting it —
     a scale degree 7 sitting immediately below a degree 1 must land one
     band *lower*, even though it's only a semitone away; that's the case
     most likely to expose a formula bug.
  If no such already-verified counterpart exists, fall back to
  render-and-inspect (§6) note-by-note instead.
- **Chords**: derive the exact reduction rule (root+quality from combined
  harmony+bass notes, slash-bass from whatever the bass actually plays,
  harmonic-rhythm granularity) from one already-transcribed measure compared
  against the parsed source for that same instant — don't assume a
  one-chord-per-measure or triad-only convention.
- Be upfront with the user about which choices are "verified against the
  source" versus "a reasonable interpretation" (chord quality on ambiguous
  chromatic passages is often genuine judgment, not a mechanical fact).

## 5. Writing the new measures

- Convert each note's duration via the scaling factor, then to jianpu
  suffixes: plain digit = quarter (4 quarter-beats), `_` = eighth, `=` =
  sixteenth, `.` = dotted (+50%), trailing `-` = +1 beat (4qb) each.
- **Tie-mark placement gotcha** (undocumented, verified by checking existing
  files and the rendered tie arc): when a note has an explicit duration
  suffix (`_`, `=`, `.`), the tie goes **after** the suffix (`4=~`, not
  `4~=` — the documented order mis-tokenizes into a wrong duration). When a
  note has no suffix (plain digit, optionally with trailing `-` extensions),
  the tie goes right after the pitch/octave, before the dashes (`4~---`).
- Sum durations per measure and confirm they total the measure capacity
  before running the tool.

## 6. Validating

- `cargo run -- check "<file>"` — must print only `"<file>": ok`. Any
  diagnostic names the problem and which line/part; isolate with a scratch
  file containing just the suspect measure if needed.
- `cargo run -- generate svg "<file>"` — must render with **no pink
  (error-highlight) measures**. Always run `check` too, since `generate`
  doesn't print diagnostics to the console.
- Rasterize and visually inspect: `rsvg-convert -o out.png "<file>.N.svg"`,
  zooming into specific measures/dots with `rsvg-convert -z <N>` + a
  Python/Pillow crop when checking fine detail (e.g. multi-comma octave
  dots, tie arcs).
- Sanity-check the musical ending — does it land on a plausible final
  chord/note given the other parts (e.g. a new inner voice landing on a
  chord tone under the melody's tonic)?
- Clean up generated `.svg`/`.png` output from the repo root before
  finishing (this repo doesn't commit generated output).

## After transcribing

If you learn something generalizable that isn't already covered above (a new
gotcha, a better technique, a source worth remembering), update this file —
that's the point of it being a skill instead of one-off scratch notes.
