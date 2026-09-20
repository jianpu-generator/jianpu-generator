//! Unit tests for `share_creation::resolve_share_id`, covering the
//! idempotent-creation decision added for GitHub-backed files (see
//! `live-share-worker/migrations/0002_docs_external_file_id.sql`): reusing
//! an existing share for the same owner + external file, minting a fresh
//! one when there's no match (or no external file id at all), and falling
//! back to the winner's id on a concurrent-create race.
//!
//! Drives `resolve_share_id` with fake, deterministic `lookup`/`create`
//! closures -- no real D1 needed, same technique as `tests/share_id.rs`.
#![allow(clippy::disallowed_macros)]

use std::cell::RefCell;

use live_share_worker::share_creation::{resolve_share_id, CreateAttempt, ResolveShareIdError};

#[tokio::test]
async fn returns_the_existing_share_id_without_creating_a_new_one() {
    let create_calls = RefCell::new(0u32);

    let result = resolve_share_id::<_, _, _, _, ()>(
        Some("octocat/repo/scores/song.jianpu"),
        || async { Ok(Some("existing-share".to_string())) },
        || {
            *create_calls.borrow_mut() += 1;
            async { Ok(CreateAttempt::Created("should-not-be-used".to_string())) }
        },
    )
    .await;

    assert_eq!(result, Ok("existing-share".to_string()));
    assert_eq!(
        *create_calls.borrow(),
        0,
        "an existing share must be reused, not re-created"
    );
}

#[tokio::test]
async fn mints_a_fresh_share_when_no_existing_share_matches_the_owner_and_file() {
    let result = resolve_share_id::<_, _, _, _, ()>(
        Some("octocat/repo/scores/song.jianpu"),
        || async { Ok(None) },
        || async { Ok(CreateAttempt::Created("new-share".to_string())) },
    )
    .await;

    assert_eq!(result, Ok("new-share".to_string()));
}

#[tokio::test]
async fn skips_the_lookup_entirely_for_a_local_only_file() {
    let lookup_calls = RefCell::new(0u32);

    let result = resolve_share_id::<_, _, _, _, ()>(
        None,
        || {
            *lookup_calls.borrow_mut() += 1;
            async { Ok(Some("should-not-be-looked-up".to_string())) }
        },
        || async { Ok(CreateAttempt::Created("local-share".to_string())) },
    )
    .await;

    assert_eq!(result, Ok("local-share".to_string()));
    assert_eq!(
        *lookup_calls.borrow(),
        0,
        "a local-only file has no account-independent key to look up by"
    );
}

#[tokio::test]
async fn resolves_to_the_winners_share_id_on_a_concurrent_create_conflict() {
    let lookup_calls = RefCell::new(0u32);

    let result = resolve_share_id::<_, _, _, _, ()>(
        Some("octocat/repo/scores/song.jianpu"),
        || {
            let mut calls = lookup_calls.borrow_mut();
            *calls += 1;
            let is_first_call = *calls == 1;
            async move {
                Ok(if is_first_call {
                    None
                } else {
                    Some("winner-share".to_string())
                })
            }
        },
        || async { Ok(CreateAttempt::Conflict) },
    )
    .await;

    assert_eq!(result, Ok("winner-share".to_string()));
    assert_eq!(
        *lookup_calls.borrow(),
        2,
        "expected the initial lookup plus one re-lookup after the conflict"
    );
}

#[tokio::test]
async fn reports_conflict_with_no_winner_when_the_re_lookup_still_finds_nothing() {
    let result = resolve_share_id::<_, _, _, _, ()>(
        Some("octocat/repo/scores/song.jianpu"),
        || async { Ok(None) },
        || async { Ok(CreateAttempt::Conflict) },
    )
    .await;

    assert_eq!(result, Err(ResolveShareIdError::ConflictWithNoWinner));
}

#[tokio::test]
async fn reports_local_create_conflicted_instead_of_panicking_on_an_impossible_conflict() {
    let result = resolve_share_id::<_, _, _, _, ()>(
        None,
        || async { Ok(None) },
        || async { Ok(CreateAttempt::Conflict) },
    )
    .await;

    assert_eq!(result, Err(ResolveShareIdError::LocalCreateConflicted));
}

#[tokio::test]
async fn propagates_an_error_from_the_lookup() {
    let result = resolve_share_id::<_, _, _, _, &str>(
        Some("octocat/repo/scores/song.jianpu"),
        || async { Err("db unavailable") },
        || async { Ok(CreateAttempt::Created("unused".to_string())) },
    )
    .await;

    assert_eq!(result, Err(ResolveShareIdError::Upstream("db unavailable")));
}

#[tokio::test]
async fn propagates_an_error_from_create() {
    let result = resolve_share_id::<_, _, _, _, &str>(
        Some("octocat/repo/scores/song.jianpu"),
        || async { Ok(None) },
        || async { Err("db unavailable") },
    )
    .await;

    assert_eq!(result, Err(ResolveShareIdError::Upstream("db unavailable")));
}
