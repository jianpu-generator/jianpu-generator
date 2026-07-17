#![allow(clippy::disallowed_macros)]
use jianpu_generator::grid_layout::types::{GridContent, Header};
use jianpu_generator::render_config::RenderConfig;

const SOURCE: &str = concat!(
    "# parts\n",
    "Soprano [s1] = notes+lyrics\n",
    "Soprano [s2] = follow[s1]\n",
    "Alto 1[a1] = follow[s1]\n",
    "Alto 2 [a2] = follow[s1]\n",
    "Tenor [t] = follow[s1]\n",
    "\n",
    "# groups\n",
    "Alto [a] = a1 a2\n",
    "Soprano [s] = s1 s2\n",
    "Vocal [v] = a s\n",
    "\n",
    "# score\n",
    "[t] 6,\n",
    "[a2] 6,\n",
    "[a1] 3\n",
    "[s2] 6\n",
    "[s1] 0\n",
);

/// Reported bug: the rendered score shows an unexplained vertical gap between
/// the `s2` row and the `a1` row that isn't present between any other pair of
/// adjacent rows.
///
/// Root cause: `s1` is an all-rest measure (`0`), so it's hidden by the
/// default `hide_resting_parts` setting. But `s2`/`a1`/`a2`/`t` all
/// `follow[s1]`, which also inherits `s1`'s `notes+lyrics` part kind — even
/// though none of these parts, including `s1` itself, ever supplies lyric
/// text anywhere in the score. Each follower still gets an implicit, empty
/// lyric slot, and the identical empty lyric rows get merged into a single
/// leftover row that renders as a blank gap with no visible content.
#[test]
fn no_orphan_empty_lyric_row_when_no_part_has_lyric_text() {
    let score = jianpu_generator::compile(SOURCE, "test.jianpu", &[]).unwrap();
    let compile_result = jianpu_generator::compiler::compile(&score);
    let compile_result = jianpu_generator::consolidator::consolidate(compile_result);
    let config = RenderConfig::from_metadata(&score.metadata);
    let header = Header {
        title: None,
        subtitle: None,
        author: None,
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
    };
    let pages = jianpu_generator::grid_layout::layout(
        &compile_result,
        &config,
        &header,
        595.0,
        842.0,
        None,
    );

    let has_orphan_lyric_row = pages[0].rows.iter().any(|row| {
        !row.elements.is_empty()
            && row.elements.iter().all(|e| {
                matches!(&e.content, GridContent::LyricSyllable(text) if text.trim().is_empty())
            })
    });

    assert!(
        !has_orphan_lyric_row,
        "no part supplies lyric text in this score, so no lyric row should be rendered"
    );
}
