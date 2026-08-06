//! quickcheck property test generalizing `tests_merge.rs`'s fixed-example
//! `round_trip_unedited_unzipped_text_reproduces_the_same_score`: for any
//! generated document, one pass of extract -> merge -> extract -> merge
//! should already be a fixed point (idempotent) between the Zipped view and
//! the Unzipped view.
//!
//! Idempotency is judged by comparing each pass's *rendered* output
//! (`render_documents_from_source_filtered_with_lyrics`'s typed `SvgDocument`
//! tree), not the intermediate `merge_unzipped_text`/`extract_unzipped_text`
//! strings directly: the repack/reconcile algorithm's token-joining is free to
//! vary incidental whitespace (e.g. how many spaces separate two lyrics
//! tokens) between passes without changing what the document actually
//! renders to, since `split_whitespace()`-based re-tokenization treats any
//! run of whitespace identically. Comparing raw text would fail on that
//! cosmetic drift even though the music is unchanged.

use quickcheck::{Arbitrary, Gen, TestResult};

use super::{extract_unzipped_text, merge_unzipped_text};
use crate::render_documents_from_source_filtered_with_lyrics;

/// Typed SVG document tree for `source`, or `None` if it fails to render
/// (the caller discards the quickcheck case in that case).
fn render(source: &str) -> Option<Vec<crate::renderer::new_types::SvgDocument>> {
    render_documents_from_source_filtered_with_lyrics(source, "test.jianpu", None, None, &[])
        .ok()
        .map(|output| output.documents)
}

/// Quarter-beats in one 4/4 measure (the fixed time signature every
/// generated document uses).
const BEATS_PER_MEASURE: u32 = 16;

/// Duration suffix/quarter-beat-weight pairs a generated note, chord, or
/// percussion hit can use: sixteenth (`=`, 1), eighth (`_`, 2), quarter (no
/// suffix, 4), and dotted quarter (`.`, 6) — see syntax.md's "Duration
/// suffixes"/"Modifiers" (notes) and "Chord/Percussion syntax" (same
/// suffixes accepted on chord and percussion heads).
const DURATION_ATOMS: [(&str, u32); 4] = [("=", 1), ("_", 2), ("", 4), (".", 6)];

/// One measure's worth of tokens, each `head(g)`'s output plus a randomly
/// chosen duration suffix, whose quarter-beat weights sum to exactly
/// `BEATS_PER_MEASURE` so the measure count doesn't drift on repack.
/// Greedily picks any atom whose weight still fits the remaining capacity;
/// since weight 1 (`=`) is always eligible while any capacity remains, this
/// always terminates with an exact-capacity measure, no backtracking needed.
fn generate_beat_filled_measure(
    g: &mut Gen,
    mut head: impl FnMut(&mut Gen) -> String,
) -> Vec<String> {
    let mut remaining = BEATS_PER_MEASURE;
    let mut tokens = Vec::new();
    while remaining > 0 {
        let choices: Vec<(&str, u32)> = DURATION_ATOMS
            .into_iter()
            .filter(|(_, weight)| *weight <= remaining)
            .collect();
        let (suffix, weight) = choices[usize::arbitrary(g) % choices.len()];
        tokens.push(format!("{}{suffix}", head(g)));
        remaining -= weight;
    }
    tokens
}

/// Small closed vocabulary for generated standalone `lyrics` parts — the
/// actual words don't matter, only that a measure's word count is whatever
/// this generator wrote (lyrics-part repack capacity is self-referential,
/// see [`super::scan_measure_token_counts`]).
const LYRIC_WORDS: [&str; 4] = ["la", "da", "na", "yo"];

/// One measure's worth of a standalone `lyrics` part's verse line: 1-4 words,
/// unconstrained by beat capacity since `lyrics`-kind parts are adurational.
fn generate_lyrics_measure(g: &mut Gen) -> Vec<String> {
    let word_count = 1 + usize::arbitrary(g) % 4;
    (0..word_count)
        .map(|_| LYRIC_WORDS[usize::arbitrary(g) % LYRIC_WORDS.len()].to_string())
        .collect()
}

/// The declared kinds a generated part can take. `NotesWithLyrics` generates
/// 1-3 verse lines per measure, independently per measure, to fuzz the
/// positional-backfill logic (a measure can't have verse 3 without verses 1
/// and 2 — see `desugar::roles_for_group`/`super::lines_for_part_at_measure`)
/// that a single-line-per-measure kind never exercises.
#[derive(Clone, Copy, Debug)]
enum GeneratedPartKind {
    Notes,
    Chords,
    Percussion,
    Lyrics,
    NotesWithLyrics,
}

const ALL_PART_KINDS: [GeneratedPartKind; 5] = [
    GeneratedPartKind::Notes,
    GeneratedPartKind::Chords,
    GeneratedPartKind::Percussion,
    GeneratedPartKind::Lyrics,
    GeneratedPartKind::NotesWithLyrics,
];

impl GeneratedPartKind {
    /// The `# parts` right-hand-side keyword for this kind.
    fn declaration_keyword(self) -> &'static str {
        match self {
            GeneratedPartKind::Notes => "notes",
            GeneratedPartKind::Chords => "chords",
            GeneratedPartKind::Percussion => "percussion",
            GeneratedPartKind::Lyrics => "lyrics",
            GeneratedPartKind::NotesWithLyrics => "notes+lyrics",
        }
    }

    /// One measure's worth of this kind's score lines: every other kind
    /// writes exactly one line, but `NotesWithLyrics` writes a notes line
    /// followed by 1-3 independently generated verse lines, so a measure's
    /// verse count varies both across measures and independently of any
    /// other measure of the same part.
    fn generate_measure_lines(self, g: &mut Gen) -> Vec<Vec<String>> {
        match self {
            GeneratedPartKind::Notes | GeneratedPartKind::Chords => {
                vec![generate_beat_filled_measure(g, |g| {
                    (1 + u8::arbitrary(g) % 7).to_string()
                })]
            }
            GeneratedPartKind::Percussion => {
                vec![generate_beat_filled_measure(g, |_| "x".to_string())]
            }
            GeneratedPartKind::Lyrics => vec![generate_lyrics_measure(g)],
            GeneratedPartKind::NotesWithLyrics => {
                let notes_line =
                    generate_beat_filled_measure(g, |g| (1 + u8::arbitrary(g) % 7).to_string());
                let verse_count = 1 + usize::arbitrary(g) % 3;
                std::iter::once(notes_line)
                    .chain((0..verse_count).map(|_| generate_lyrics_measure(g)))
                    .collect()
            }
        }
    }
}

/// A minimal, always-parseable `.jianpu` document with 1-3 parts of randomly
/// chosen kinds (`notes`, `chords`, `percussion`, `lyrics`, `notes+lyrics`)
/// and 1-5 measures. `notes`/`chords`/`percussion` measures are filled with a
/// random mix of sixteenth, eighth, quarter, and dotted-quarter tokens
/// summing to a full 4/4 bar; `lyrics` measures get 1-4 words;
/// `notes+lyrics` measures get a beat-filled notes line plus 1-3
/// independently generated verse lines — enough structural and kind
/// variation to exercise the repack/reconcile machinery (including the
/// multi-verse positional-backfill rule) without needing a full
/// grammar-aware generator.
#[derive(Clone, Debug)]
struct RandomJianpuDocument {
    source: String,
}

impl Arbitrary for RandomJianpuDocument {
    fn arbitrary(g: &mut Gen) -> Self {
        let part_count = 1 + usize::arbitrary(g) % 3;
        let measure_count = 1 + usize::arbitrary(g) % 5;
        let parts: Vec<(String, GeneratedPartKind)> = (0..part_count)
            .map(|index| {
                let kind = ALL_PART_KINDS[usize::arbitrary(g) % ALL_PART_KINDS.len()];
                (format!("Part{index}"), kind)
            })
            .collect();

        let mut parts_section = String::new();
        for (abbrev, kind) in &parts {
            parts_section.push_str(&format!("{abbrev} = {}\n", kind.declaration_keyword()));
        }

        let mut measure_groups: Vec<String> = Vec::with_capacity(measure_count);
        for _ in 0..measure_count {
            let mut part_lines: Vec<String> = Vec::with_capacity(parts.len());
            for (abbrev, kind) in &parts {
                for tokens in kind.generate_measure_lines(g) {
                    part_lines.push(format!("[{abbrev}] {}", tokens.join(" ")));
                }
            }
            measure_groups.push(part_lines.join("\n"));
        }
        let score_content = measure_groups.join("\n\n");

        let source = format!(
            "# metadata\ntitle = \"QC\"\n\n# parts\n{parts_section}\n# score\n{score_content}\n"
        );
        RandomJianpuDocument { source }
    }
}

/// A single, `Arbitrary`-generated whitespace-token edit: append, insert, or
/// delete one token, applied to whatever extracted text a given
/// `RandomJianpuDocument` happens to produce. Kept as its own `Arbitrary`
/// type (rather than a plain closure over a `Gen`) since quickcheck's
/// `quickcheck!` macro only accepts `Arbitrary` property arguments.
#[derive(Clone, Debug)]
struct RandomTokenEdit {
    variant: u8,
    position_seed: usize,
    word_index: usize,
}

impl Arbitrary for RandomTokenEdit {
    fn arbitrary(g: &mut Gen) -> Self {
        RandomTokenEdit {
            variant: u8::arbitrary(g) % 3,
            position_seed: usize::arbitrary(g),
            word_index: usize::arbitrary(g),
        }
    }
}

impl RandomTokenEdit {
    /// Applies this edit to `text`'s whitespace tokens before the *first*
    /// `merge_unzipped_text` call: appends, inserts, or deletes a single
    /// token. This is what actually exercises the diff-anchored lyrics
    /// repack path beyond the unedited case — the existing property already
    /// passes on zero edits, which alone doesn't validate that a genuine
    /// edit round-trips correctly (this is the scenario that caught the
    /// original bug and the reverted naive fix). A no-op (returns `text`
    /// unchanged) when `text` has no tokens to edit.
    fn apply(&self, text: &str) -> String {
        let mut tokens: Vec<String> = text.split_whitespace().map(String::from).collect();
        if tokens.is_empty() {
            return text.to_string();
        }
        let word = LYRIC_WORDS[self.word_index % LYRIC_WORDS.len()].to_string();
        match self.variant % 3 {
            0 => tokens.push(word),
            1 => {
                let index = self.position_seed % (tokens.len() + 1);
                tokens.insert(index, word);
            }
            _ => {
                let index = self.position_seed % tokens.len();
                tokens.remove(index);
            }
        }
        tokens.join(" ")
    }
}

quickcheck::quickcheck! {
    fn prop_extract_merge_round_trip_is_idempotent(doc: RandomJianpuDocument) -> TestResult {
        let source = doc.source;
        let Ok(extracted) = extract_unzipped_text(&source) else {
            return TestResult::discard();
        };
        let Ok(merged) = merge_unzipped_text(&source, &extracted.text) else {
            return TestResult::discard();
        };
        let Ok(reextracted) = extract_unzipped_text(&merged) else {
            return TestResult::discard();
        };
        let Ok(remerged) = merge_unzipped_text(&merged, &reextracted.text) else {
            return TestResult::discard();
        };

        let (Some(rendered_merged), Some(rendered_remerged)) = (render(&merged), render(&remerged))
        else {
            return TestResult::discard();
        };

        TestResult::from_bool(rendered_merged == rendered_remerged)
    }

    /// Same oracle as `prop_extract_merge_round_trip_is_idempotent`, but the
    /// *first* `merge_unzipped_text` call is fed a small single-token edit
    /// (append/insert/delete, via `RandomTokenEdit::apply`) rather than the
    /// untouched extracted text — the round trip from `merged` onward should
    /// still be a fixed point.
    fn prop_extract_merge_round_trip_is_idempotent_after_one_edit(
        doc: RandomJianpuDocument,
        edit: RandomTokenEdit
    ) -> TestResult {
        let source = doc.source;
        let Ok(extracted) = extract_unzipped_text(&source) else {
            return TestResult::discard();
        };
        let edited_text = edit.apply(&extracted.text);
        let Ok(merged) = merge_unzipped_text(&source, &edited_text) else {
            return TestResult::discard();
        };
        let Ok(reextracted) = extract_unzipped_text(&merged) else {
            return TestResult::discard();
        };
        let Ok(remerged) = merge_unzipped_text(&merged, &reextracted.text) else {
            return TestResult::discard();
        };

        let (Some(rendered_merged), Some(rendered_remerged)) = (render(&merged), render(&remerged))
        else {
            return TestResult::discard();
        };

        TestResult::from_bool(rendered_merged == rendered_remerged)
    }
}

#[test]
fn debug_find_failing_case() {
    for size in 1..200 {
        let mut g = Gen::new(size);
        let doc = RandomJianpuDocument::arbitrary(&mut g);
        let source = doc.source.clone();
        let Ok(extracted) = extract_unzipped_text(&source) else {
            continue;
        };
        let Ok(merged) = merge_unzipped_text(&source, &extracted.text) else {
            continue;
        };
        let Ok(reextracted) = extract_unzipped_text(&merged) else {
            continue;
        };
        let Ok(remerged) = merge_unzipped_text(&merged, &reextracted.text) else {
            continue;
        };
        let (Some(rendered_merged), Some(rendered_remerged)) = (render(&merged), render(&remerged))
        else {
            continue;
        };
        if rendered_merged != rendered_remerged {
            println!("FAILING SOURCE (size={size}):\n{source}");
            println!("EXTRACTED:\n{:?}", extracted.text);
            println!("REEXTRACTED:\n{:?}", reextracted.text);
            println!("MERGED:\n{merged:?}");
            println!("REMERGED:\n{remerged:?}");

            for (label, doc_source) in [("merged", &merged), ("remerged", &remerged)] {
                let (sections, _) = crate::parser::load_document_sections(doc_source);
                let (parts_content, parts_offset) = sections.parts;
                let (score_content, score_offset) = sections.score;
                let (declarations, _) =
                    crate::parser::parts_parser::parse_parts(&parts_content, parts_offset, &[]);
                let raw_groups =
                    crate::parser::score::measure_group::collect_groups(&score_content);
                println!("[{label}] raw_groups count = {}", raw_groups.len());
                for (i, group) in raw_groups.iter().enumerate() {
                    println!(
                        "[{label}] group {i}: {:?}",
                        group.iter().map(|l| &l.0).collect::<Vec<_>>()
                    );
                }
                let (desugared, slots_per_group, errors, _refs) =
                    crate::desugar::desugar_groups(raw_groups, &declarations, &[], score_offset)
                        .unwrap();
                for (i, (group, slots)) in desugared.iter().zip(slots_per_group.iter()).enumerate()
                {
                    println!("[{label}] measure {i}:");
                    for (line, slot) in group.iter().rev().zip(slots.iter().rev()) {
                        println!(
                            "[{label}]   track={} role={:?} content={:?}",
                            slot.track_index, slot.role, line.content
                        );
                    }
                }
                println!("[{label}] errors: {errors:?}");
            }

            panic!("found failing case");
        }
    }
}
