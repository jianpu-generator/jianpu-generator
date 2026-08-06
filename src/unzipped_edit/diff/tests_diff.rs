//! Pure algorithm tests for [`super::diff_tokens`], independent of any
//! Lyrics/measure domain type.

use super::*;

#[test]
fn identical_sequences_produce_an_all_equal_walk_with_strictly_increasing_owner_measure() {
    let original = vec!["a", "b", "c"];
    let owners = vec![0, 1, 2];
    let edited = vec!["a", "b", "c"];
    let script = diff_tokens(&original, &owners, &edited);

    assert_eq!(script.len(), 3);
    let mut last_owner: Option<usize> = None;
    for token in &script {
        match token {
            DiffToken::Equal { owner_measure, .. } => {
                if let Some(last) = last_owner {
                    assert!(*owner_measure > last);
                }
                last_owner = Some(*owner_measure);
            }
            DiffToken::Insert { .. } => panic!("expected an all-Equal walk"),
        }
    }
}

#[test]
fn a_pure_append_produces_trailing_inserts_only() {
    let original = vec!["a", "b"];
    let owners = vec![0, 1];
    let edited = vec!["a", "b", "c", "d"];
    let script = diff_tokens(&original, &owners, &edited);

    assert_eq!(script.len(), 4);
    assert!(matches!(
        script[0],
        DiffToken::Equal {
            edited_index: 0,
            owner_measure: 0
        }
    ));
    assert!(matches!(
        script[1],
        DiffToken::Equal {
            edited_index: 1,
            owner_measure: 1
        }
    ));
    assert!(matches!(script[2], DiffToken::Insert { edited_index: 2 }));
    assert!(matches!(script[3], DiffToken::Insert { edited_index: 3 }));
}

#[test]
fn a_prepend_produces_a_leading_insert_before_the_original_equal_run() {
    let original = vec!["b", "c"];
    let owners = vec![0, 1];
    let edited = vec!["a", "b", "c"];
    let script = diff_tokens(&original, &owners, &edited);

    assert_eq!(script.len(), 3);
    assert!(matches!(script[0], DiffToken::Insert { edited_index: 0 }));
    assert!(matches!(
        script[1],
        DiffToken::Equal {
            edited_index: 1,
            owner_measure: 0
        }
    ));
    assert!(matches!(
        script[2],
        DiffToken::Equal {
            edited_index: 2,
            owner_measure: 1
        }
    ));
}

#[test]
fn a_mid_sequence_insert_splits_the_equal_run_around_it() {
    let original = vec!["a", "c"];
    let owners = vec![0, 1];
    let edited = vec!["a", "b", "c"];
    let script = diff_tokens(&original, &owners, &edited);

    assert_eq!(script.len(), 3);
    assert!(matches!(
        script[0],
        DiffToken::Equal {
            edited_index: 0,
            owner_measure: 0
        }
    ));
    assert!(matches!(script[1], DiffToken::Insert { edited_index: 1 }));
    assert!(matches!(
        script[2],
        DiffToken::Equal {
            edited_index: 2,
            owner_measure: 1
        }
    ));
}

#[test]
fn a_deletion_is_simply_absent_from_the_walk() {
    let original = vec!["a", "b", "c"];
    let owners = vec![0, 1, 2];
    let edited = vec!["a", "c"];
    let script = diff_tokens(&original, &owners, &edited);

    assert_eq!(script.len(), 2);
    assert!(matches!(
        script[0],
        DiffToken::Equal {
            edited_index: 0,
            owner_measure: 0
        }
    ));
    assert!(matches!(
        script[1],
        DiffToken::Equal {
            edited_index: 1,
            owner_measure: 2
        }
    ));
}

#[test]
fn duplicate_tokens_still_produce_a_valid_length_preserving_partition() {
    // "la la la" edited to "la la la la" (one more "la" appended): which
    // specific "la" is called Equal vs Insert is content-interchangeable, so
    // this only asserts the walk's shape (length, edited_index coverage,
    // match count), not one exact partition.
    let original = vec!["la", "la", "la"];
    let owners = vec![0, 1, 2];
    let edited = vec!["la", "la", "la", "la"];
    let script = diff_tokens(&original, &owners, &edited);

    assert_eq!(script.len(), 4);
    let mut edited_indices: Vec<usize> = script
        .iter()
        .map(|token| match token {
            DiffToken::Equal { edited_index, .. } | DiffToken::Insert { edited_index } => {
                *edited_index
            }
        })
        .collect();
    edited_indices.sort_unstable();
    assert_eq!(edited_indices, vec![0, 1, 2, 3]);

    let equal_count = script
        .iter()
        .filter(|token| matches!(token, DiffToken::Equal { .. }))
        .count();
    assert_eq!(equal_count, 3, "all three original tokens should match");
}
