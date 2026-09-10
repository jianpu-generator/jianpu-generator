//! Unit tests for `doc::{apply_write, to_public_doc}`, ported from the old
//! `live-share-worker/test/doc.test.ts` and retargeted to the
//! resolved-`user_id` write-guard (no more "first write claims ownership"
//! -- see `src/doc.rs`'s doc comment).
#![allow(clippy::disallowed_macros)]

use live_share_worker::doc::{apply_write, empty_doc, to_public_doc, StoredDoc, WriteError};
use live_share_worker::protocol::{SyncedDoc, SyncedWriteRequest};

fn sample_doc() -> StoredDoc {
    StoredDoc {
        share_id: "share-1".to_string(),
        owner_user_id: "owner-1".to_string(),
        filename: "song.jianpu".to_string(),
        content: "[M] 1".to_string(),
        revision: 1,
        ended: false,
        created_at: 1_000,
        updated_at: 1_000,
    }
}

#[test]
fn a_share_with_no_docs_row_reads_the_same_as_one_shared_then_stopped() {
    assert_eq!(to_public_doc(None), empty_doc());
}

#[test]
fn never_leaks_owner_user_id_to_the_caller() {
    let stored = sample_doc();
    assert_eq!(
        to_public_doc(Some(&stored)),
        SyncedDoc {
            ended: false,
            filename: "song.jianpu".to_string(),
            content: "[M] 1".to_string(),
            revision: 1,
        }
    );
}

#[test]
fn rejects_an_update_from_an_identity_that_does_not_match_the_owner() {
    let existing = sample_doc();
    let request = SyncedWriteRequest::Update {
        identity_token: "irrelevant-at-this-layer".to_string(),
        filename: "tampered.jianpu".to_string(),
        content: "[M] 2".to_string(),
        revision: 2,
    };

    let result = apply_write(&existing, Some("someone-elses-id"), &request, 2_000);

    assert_eq!(result, Err(WriteError::Forbidden));
}

#[test]
fn overwrites_content_on_an_update_from_the_owner() {
    let existing = sample_doc();
    let request = SyncedWriteRequest::Update {
        identity_token: "irrelevant-at-this-layer".to_string(),
        filename: "song.jianpu".to_string(),
        content: "[M] 2".to_string(),
        revision: 2,
    };

    let result = apply_write(&existing, Some("owner-1"), &request, 2_000);

    assert_eq!(
        result,
        Ok(StoredDoc {
            content: "[M] 2".to_string(),
            revision: 2,
            updated_at: 2_000,
            ..existing
        })
    );
}

#[test]
fn marks_the_share_ended_on_stop_keeping_the_doc_so_a_later_update_reproduces_it() {
    let existing = sample_doc();
    let request = SyncedWriteRequest::Stop {
        identity_token: "irrelevant-at-this-layer".to_string(),
    };

    let result = apply_write(&existing, Some("owner-1"), &request, 2_000);

    assert_eq!(
        result,
        Ok(StoredDoc {
            ended: true,
            updated_at: 2_000,
            ..existing
        })
    );
}

#[test]
fn rejects_a_stop_from_a_non_owner_identity() {
    let existing = sample_doc();
    let request = SyncedWriteRequest::Stop {
        identity_token: "irrelevant-at-this-layer".to_string(),
    };

    let result = apply_write(&existing, Some("someone-elses-id"), &request, 2_000);

    assert_eq!(result, Err(WriteError::Forbidden));
}

#[test]
fn rejects_a_write_when_identity_resolution_produced_nothing() {
    let existing = sample_doc();
    let request = SyncedWriteRequest::Stop {
        identity_token: "irrelevant-at-this-layer".to_string(),
    };

    let result = apply_write(&existing, None, &request, 2_000);

    assert_eq!(result, Err(WriteError::Forbidden));
}
