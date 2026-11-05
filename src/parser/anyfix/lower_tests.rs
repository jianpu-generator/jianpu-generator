use super::lower;

fn lowered(source: &str) -> String {
    let lowered = lower(source, &[]);
    assert!(lowered.errors.is_empty(), "{:?}", lowered.errors);
    lowered.source
}

fn error_messages(source: &str) -> Vec<String> {
    lower(source, &[])
        .errors
        .into_iter()
        .map(|error| error.message)
        .collect()
}

#[test]
fn metadata_fields_become_key_value_lines() {
    assert_eq!(
        lowered(
            r#"metadata {
                title : "Song",
                max measures per system : 4,
                hide resting parts : no,
                title style : { font size : 32, bold : yes, font family : sans serif },
                directive row offset : (0, -4),
            }"#
        ),
        "# metadata\n\
         title = \"Song\"\n\
         max_measures_per_system = 4\n\
         hide_resting_parts = no\n\
         title = { font_size: 32, bold: yes, font_family: sans_serif }\n\
         directive_row_offset = 0 -4\n"
    );
}

#[test]
fn part_options_become_positional_suffixes() {
    assert_eq!(
        lowered(
            r#"parts [
                "Violin" [V] : notes { octave : -1, volume : 80%, instrument : violin },
                "Echo" [E] : follow [V] { volume : 50% },
                "Words" [Vv1] : lyrics [V],
                "Snare" : percussion { sound : 38 },
                "Alto 1 & Tenor" [A1&T] : notes,
            ]"#
        ),
        "# parts\n\
         Violin [V] = notes violin 80% -1\n\
         Echo [E] = follow[V] 50%\n\
         Words [Vv1] = lyrics[V]\n\
         Snare = percussion \"38: Acoustic Snare\"\n\
         Alto 1 & Tenor [A1&T] = notes\n"
    );
}

#[test]
fn a_sound_number_is_labelled() {
    assert_eq!(
        lowered(r#"parts [ "Piano" [P] : notes { sound : 1 } ]"#),
        "# parts\nPiano [P] = notes \"1: Unknown\"\n"
    );
}

#[test]
fn sequence_filters_become_suffixes() {
    assert_eq!(
        lowered(r#"sequence [ Verse, Chorus WITHOUT [S, A2], "B" ONLY [S], "Last verse" ]"#),
        "# sequence\nVerse, Chorus(-S -A2), B(S), Last verse\n"
    );
}

#[test]
fn a_preset_lists_parts() {
    assert_eq!(
        lowered(r#"preset "Solo" { visible parts : [M, B], soloed parts : [], notation : cello }"#),
        "# preset \"Solo\"\nvisible_parts = M, B\nsoloed_parts = \nnotation = cello\n"
    );
}

#[test]
fn directives_lead_the_next_measure() {
    assert_eq!(
        lowered(
            r#"score [
                { bpm : 92, key : "F#3", time : 3/4, label : "Verse 1", break },
                """
                [M] 1 2 3

                [M] 4# 5 6,   # sharp and low octave
                """,
                { key : +2 },
                """
                [M] 1 2 3
                """,
            ]"#
        ),
        "# score\n\
         bpm=92 key=F#3 time=3/4 label=\"Verse 1\" break\n\
         [M] 1 2 3\n\
         \n\
         [M] 4# 5 6,   # sharp and low octave\n\
         \n\
         key=+2\n\
         [M] 1 2 3\n"
    );
}

#[test]
fn directives_with_no_measures_after_them_are_a_measure_of_their_own() {
    assert_eq!(lowered(r#"score [ { key : C4 } ]"#), "# score\nkey=C4\n");
    assert_eq!(
        lowered(
            r#"score [ { key : C4 }, { bpm : 90 }, """
          [M] 1
          """, { bpm : 80 } ]"#
        ),
        "# score\n\
         key=C4\n\
         \n\
         bpm=90\n\
         [M] 1\n\
         \n\
         bpm=80\n"
    );
}

#[test]
fn an_unknown_section_is_reported() {
    assert_eq!(
        error_messages("lyrics []"),
        ["found `lyrics _` at the top level; expected `metadata {…}`, `parts […]`, `sequence […]`, `preset \"Name\" {…}` or `score […]`"]
    );
}

#[test]
fn an_unknown_part_option_is_reported() {
    assert_eq!(
        error_messages(r#"parts [ "Piano" [P] : notes { loudness : 3 } ]"#),
        ["unknown part option `loudness`; expected instrument, sound, volume or octave"]
    );
}

#[test]
fn anyfix_syntax_errors_are_reported() {
    assert_eq!(lower("parts [", &[]).errors.len(), 1);
}
