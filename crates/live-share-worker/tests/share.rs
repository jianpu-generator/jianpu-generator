//! Unit tests for `share::to_public_doc` -- the viewer-facing view of a
//! share that points at a cloud `files` row.
#![allow(clippy::disallowed_macros)]

use live_share_worker::protocol::SyncedDoc;
use live_share_worker::share::{empty_doc, to_public_doc, ShareView};

fn live_view() -> ShareView {
    ShareView {
        ended_at: None,
        trashed_at: None,
        filename: "song.jianpu".to_string(),
        content: "[M] 1".to_string(),
        revision: 3,
        owner_login: Some("octocat".to_string()),
    }
}

#[test]
fn a_share_id_with_no_row_reads_as_the_empty_doc() {
    assert_eq!(to_public_doc(None), empty_doc());
}

#[test]
fn a_live_share_shows_the_files_current_content_and_owner_login() {
    assert_eq!(
        to_public_doc(Some(&live_view())),
        SyncedDoc {
            ended: false,
            filename: "song.jianpu".to_string(),
            content: "[M] 1".to_string(),
            revision: 3,
            owner_login: Some("octocat".to_string()),
        }
    );
}

#[test]
fn carries_no_owner_login_when_none_is_cached() {
    let view = ShareView {
        owner_login: None,
        ..live_view()
    };
    assert_eq!(to_public_doc(Some(&view)).owner_login, None);
}

#[test]
fn a_stopped_share_hides_the_files_content() {
    let view = ShareView {
        ended_at: Some(2_000),
        ..live_view()
    };
    assert_eq!(
        to_public_doc(Some(&view)),
        SyncedDoc {
            owner_login: Some("octocat".to_string()),
            ..empty_doc()
        }
    );
}

#[test]
fn a_share_whose_file_is_in_the_bin_reads_as_ended() {
    let view = ShareView {
        trashed_at: Some(2_000),
        ..live_view()
    };
    assert_eq!(
        to_public_doc(Some(&view)),
        SyncedDoc {
            owner_login: Some("octocat".to_string()),
            ..empty_doc()
        }
    );
}

#[test]
fn never_leaks_an_owner_id_on_the_wire() -> Result<(), serde_json::Error> {
    let json = serde_json::to_value(to_public_doc(Some(&live_view())))?;
    let keys: Vec<&String> = json
        .as_object()
        .map(|object| object.keys().collect())
        .unwrap_or_default();
    assert_eq!(
        keys,
        vec!["content", "ended", "filename", "ownerLogin", "revision"]
    );
    Ok(())
}
