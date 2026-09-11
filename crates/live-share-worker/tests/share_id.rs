//! Unit tests for `generate_unique_id`'s collision-handling loop, covering
//! the TODO §8 fix: a collision must re-roll a brand new, still-`length`-
//! character candidate, never mint one that's longer than `length` (which
//! `web/src/syncedShareUrl.ts`'s fixed-length `SHARE_ID_PATTERN` couldn't
//! parse back out of a share link).
//!
//! Drives `generate_unique_id` with a fake, deterministic candidate
//! generator (rather than the real `js_sys::Math::random()`-based one),
//! since that real generator panics outside a wasm target.
#![allow(clippy::disallowed_macros)]

use std::cell::RefCell;

use live_share_worker::share_id::generate_unique_id;

const LENGTH: usize = 11;

/// A fake candidate generator that returns `"cccccccccc0"`, `"cccccccccc1"`,
/// `"cccccccccc2"`, ... -- each call's output is distinct but always
/// exactly `LENGTH` characters, so tests can assert on both "which
/// candidate won" and "did the length ever change".
fn fake_candidates() -> impl FnMut() -> String {
    let mut next = 0u32;
    move || {
        let candidate = format!("cccccccccc{next}");
        next += 1;
        candidate
    }
}

#[tokio::test]
async fn returns_the_first_candidate_when_nothing_collides() {
    let id = generate_unique_id::<_, _, _, ()>(fake_candidates(), |_candidate| async { Ok(false) })
        .await
        .unwrap();
    assert_eq!(id, "cccccccccc0");
    assert_eq!(id.len(), LENGTH);
}

/// The core regression test: on a collision, the fallback must re-roll a
/// fresh, same-length candidate -- not extend the colliding one to a
/// longer id.
#[tokio::test]
async fn rerolls_a_same_length_candidate_on_collision_instead_of_extending_it() {
    let calls = RefCell::new(0u32);
    let id = generate_unique_id::<_, _, _, ()>(fake_candidates(), |candidate| {
        let mut calls = calls.borrow_mut();
        *calls += 1;
        let is_first_call = *calls == 1;
        async move {
            assert_eq!(
                candidate.len(),
                LENGTH,
                "re-rolled candidate must stay the same length as the original"
            );
            Ok(is_first_call)
        }
    })
    .await
    .unwrap();

    assert_eq!(*calls.borrow(), 2, "expected exactly one re-roll");
    assert_eq!(
        id, "cccccccccc1",
        "should have re-rolled to the 2nd candidate"
    );
    assert_eq!(id.len(), LENGTH);
}

/// Keeps re-rolling across several collisions in a row, still never
/// growing the id's length and still landing on the first free candidate.
#[tokio::test]
async fn keeps_rerolling_across_multiple_consecutive_collisions() {
    let calls = RefCell::new(0u32);
    let id = generate_unique_id::<_, _, _, ()>(fake_candidates(), |_candidate| {
        let mut calls = calls.borrow_mut();
        *calls += 1;
        let should_collide = *calls <= 5;
        async move { Ok(should_collide) }
    })
    .await
    .unwrap();

    assert_eq!(*calls.borrow(), 6);
    assert_eq!(id, "cccccccccc5");
    assert_eq!(id.len(), LENGTH);
}

/// A collision check erroring out (e.g. a real D1 failure) must propagate
/// rather than be swallowed into "treat as unique".
#[tokio::test]
async fn propagates_an_error_from_the_exists_check() {
    let result = generate_unique_id::<_, _, _, &str>(fake_candidates(), |_candidate| async {
        Err("db unavailable")
    })
    .await;
    assert_eq!(result, Err("db unavailable"));
}
