//! Pure, D1-free resolution of the `share_id` a `POST /shares` call should
//! return, covering the idempotent-creation invariant added by
//! `live-share-worker/migrations/0002_docs_external_file_id.sql` (a unique
//! index on `(owner_user_id, external_file_id)`): the same GitHub-backed
//! file, shared again by the same owner, always resolves to the same share
//! rather than minting a second one.
//!
//! Generic over injected `lookup`/`create` async closures, same technique as
//! `share_id::generate_unique_id`'s collision-handling loop, so
//! `tests/share_creation.rs` can drive this with fake, deterministic D1
//! stand-ins -- no real D1 needed. See `crate::handlers::create_share` for
//! the real `db`-backed closures this is wired up with in production.

use std::future::Future;

/// What one attempt to insert a fresh `docs` row reported back. `Conflict`
/// covers the unique-constraint race between two concurrent `POST /shares`
/// calls for the same `(owner_user_id, external_file_id)` -- exactly which
/// real D1 error maps to `Conflict` is `crate::handlers::create_share`'s
/// concern, not modeled here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateAttempt {
    Created(String),
    Conflict,
}

/// What can keep `resolve_share_id` from producing a `share_id`. `Upstream`
/// is whatever the injected `lookup`/`create` closures themselves reported
/// (a real D1 failure, in production). The other two variants are states
/// `resolve_share_id` doesn't assume away just because they're not supposed
/// to happen -- see each variant's doc comment -- so the caller (which knows
/// a concrete error type to report) decides how to surface them, rather
/// than this D1-free module asserting an invariant at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveShareIdError<E> {
    Upstream(E),
    /// A local-only create (no `external_file_id`) reported
    /// `CreateAttempt::Conflict`. Structurally shouldn't happen -- migration
    /// 0002's partial unique index only fires when `external_file_id` is
    /// not `NULL` -- but a local-only create still goes through the same
    /// `create` closure as an external-file one, so this module can't rule
    /// it out by construction.
    LocalCreateConflicted,
    /// A `Conflict` for an `external_file_id` create, but the immediate
    /// re-lookup still found nothing -- the competing insert that caused
    /// the conflict hasn't committed yet (or something else removed it
    /// again in between).
    ConflictWithNoWinner,
}

impl<E> From<E> for ResolveShareIdError<E> {
    fn from(error: E) -> Self {
        ResolveShareIdError::Upstream(error)
    }
}

/// Resolves the `share_id` to hand back for a `POST /shares` call:
///
/// - `external_file_id` is `None` (a local-only file): always mints a fresh
///   share, skipping the lookup entirely -- there's no account-independent
///   key to reuse for it.
/// - `external_file_id` is `Some`: looks up an existing share for this
///   owner + file first, so re-sharing the same file as the same GitHub
///   account reproduces the same link instead of minting a second one. A
///   miss falls through to minting one; a `Conflict` reported by `create`
///   (two concurrent misses both raced past the lookup) re-runs the lookup
///   once more and returns the winner's id instead of failing the request.
pub async fn resolve_share_id<Lookup, LookupFut, Create, CreateFut, E>(
    external_file_id: Option<&str>,
    mut lookup_existing: Lookup,
    mut create: Create,
) -> Result<String, ResolveShareIdError<E>>
where
    Lookup: FnMut() -> LookupFut,
    LookupFut: Future<Output = Result<Option<String>, E>>,
    Create: FnMut() -> CreateFut,
    CreateFut: Future<Output = Result<CreateAttempt, E>>,
{
    if external_file_id.is_none() {
        return match create().await? {
            CreateAttempt::Created(share_id) => Ok(share_id),
            CreateAttempt::Conflict => Err(ResolveShareIdError::LocalCreateConflicted),
        };
    }

    if let Some(share_id) = lookup_existing().await? {
        return Ok(share_id);
    }

    match create().await? {
        CreateAttempt::Created(share_id) => Ok(share_id),
        CreateAttempt::Conflict => match lookup_existing().await? {
            Some(share_id) => Ok(share_id),
            None => Err(ResolveShareIdError::ConflictWithNoWinner),
        },
    }
}
