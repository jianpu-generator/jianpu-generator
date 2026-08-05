//! [`merge_unzipped_text`]: the whole-document merge-back entry point. See
//! the parent module's doc comment for the overall Unzipped Edit design;
//! the algorithm itself is split across `parse` (Phase 3a: split unzipped
//! text into per-part blocks), `repack` (Phase 3b: re-bar each block into
//! measures), and `reassemble` (Phase 3c-3d: reconcile measure counts and
//! rebuild the `# score` section).

use crate::desugar;
use crate::parser;

use super::capacity::scan_measure_capacities;
use super::parse::{split_unzipped_blocks, validate_unzipped_blocks};
use super::reassemble::{
    build_raw_groups_for_desugar, reconcile_bucket_lengths, render_score_lines,
};
use super::repack::repack_all_parts;
use super::{resolve_document_context, UnzippedEditError};

/// Merge whole-document edits to Unzipped Edit text back into `source`'s
/// `# score` section, returning the full updated source.
///
/// See the parent module doc comment for the full repack/reconcile/reassemble
/// algorithm (Phase 3a-3d): split `unzipped_text` into per-part primary and
/// lyrics-verse blocks (`parse::split_unzipped_blocks`), repack each into
/// measures by beat (or, for `Lyrics`-role content, token) capacity
/// (`repack::repack_all_parts`), pad every part's every bucket to the same
/// measure count (`reassemble::reconcile_bucket_lengths`), then reassemble
/// the `# score` section (`reassemble::build_raw_groups_for_desugar`) —
/// pinning directive lines to their original measure index and never
/// regenerating them, force-filling any positionally-required-but-absent
/// verse/notes line — and run one final `desugar::desugar_groups` pass so
/// implicit rests/verses are filled in for parts that come up short at a
/// given measure.
pub fn merge_unzipped_text(source: &str, unzipped_text: &str) -> Result<String, UnzippedEditError> {
    let context = resolve_document_context(source);
    let blocks = split_unzipped_blocks(unzipped_text)?;
    validate_unzipped_blocks(&blocks, &context.declarations)?;

    let raw_groups = parser::score::measure_group::collect_groups(&context.score_content);
    let original_measure_count = raw_groups.len();
    let capacities = scan_measure_capacities(&context.score_content);

    let (mut part_buckets, mut verse_buckets_per_part) =
        repack_all_parts(&context, &blocks, &capacities)?;
    let new_total = reconcile_bucket_lengths(
        original_measure_count,
        &mut part_buckets,
        &mut verse_buckets_per_part,
    );
    let raw_groups_for_desugar = build_raw_groups_for_desugar(
        &context,
        &raw_groups,
        &part_buckets,
        &verse_buckets_per_part,
        new_total,
    );

    let (desugared, slots_per_group, _errors, _references) = desugar::desugar_groups(
        raw_groups_for_desugar,
        &context.declarations,
        &context.resolved_groups,
        context.score_offset,
    )
    .map_err(|_| UnzippedEditError::ParseFailed)?;

    let score_lines = render_score_lines(&context.declarations, &desugared, &slots_per_group);
    let new_score_content = if score_lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", score_lines.join("\n\n"))
    };

    let mut result = String::with_capacity(
        context.score_offset
            + new_score_content.len()
            + source
                .len()
                .saturating_sub(context.score_offset + context.score_content.len()),
    );
    result.push_str(source.get(..context.score_offset).unwrap_or(source));
    result.push_str(&new_score_content);
    result.push_str(
        source
            .get(context.score_offset + context.score_content.len()..)
            .unwrap_or(""),
    );
    Ok(result)
}
