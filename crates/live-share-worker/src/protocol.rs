//! Wire types for the Synced Share worker's HTTP API. Ported from the old
//! TS `live-share-worker/src/protocol.ts` (mirrored, not shared via a
//! workspace package, with `web/src/syncedShare/protocol.ts` -- keep the
//! two in sync by hand when either changes, same constraint as the file
//! this was ported from).
//!
//! JSON field names use `camelCase` on the wire (matching the old TS
//! convention, e.g. `ownerToken`), even though Rust field names stay
//! `snake_case`.

use serde::{Deserialize, Serialize};

/// What `GET /shares/:share_id` returns. `ended` mirrors the owner having
/// pressed "Stop Sync" -- the `docs` row keeps existing (same `share_id`,
/// same `owner_user_id`) so a later "Sync" click reproduces the same link,
/// but a viewer must not treat `content`/`filename` as current once this is
/// true. Never carries `owner_user_id`, `share_id`, any token/hash, or any
/// other internal id -- see `crate::doc::to_public_doc`. `owner_login` is
/// the one deliberate exception: `user_identities.login` (a cached, public
/// GitHub display name, not an internal id) is exposed here specifically so
/// the viewer-facing `SyncedShareBanner` can show "Shared by @login"; it's
/// `None` when the owner has no cached login (task 10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncedDoc {
    pub ended: bool,
    pub filename: String,
    pub content: String,
    pub revision: i64,
    pub owner_login: Option<String>,
}

/// Body of `POST /shares/:share_id` -- only the share's owner is ever
/// allowed to have a write applied (enforced server-side by
/// `crate::resolve_role::resolve_role`, keyed on the identity resolved from
/// `identity_token`, see `crate::identity`).
///
/// `identity_token` is opaque at this layer: for now it's whatever
/// `crate::identity::stub::StubIdentityProvider` accepts (not a real OAuth
/// token -- see that module's doc comment). Struct-style enum variants
/// (never tuple variants, per this repo's no-tuple-in-new-data-structures
/// convention) mirror the old TS `SyncedUpdateRequest` / `SyncedStopRequest`
/// union members, tagged the same way (`type: "update" | "stop"`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SyncedWriteRequest {
    Update {
        identity_token: String,
        filename: String,
        content: String,
        revision: i64,
    },
    Stop {
        identity_token: String,
    },
}

impl SyncedWriteRequest {
    /// The opaque identity token every write carries, regardless of variant.
    pub fn identity_token(&self) -> &str {
        match self {
            SyncedWriteRequest::Update { identity_token, .. }
            | SyncedWriteRequest::Stop { identity_token } => identity_token,
        }
    }
}

/// Body of `POST /shares` -- creates a brand-new, server-generated share
/// (see `TODO-synced-share-rust-d1-migration.md` §1: `shareId` generation
/// moved server-side). Requires a resolved identity; unlike the old TS
/// `index.ts`, there is no more implicit "first POST to a client-chosen id
/// creates it" behavior.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShareRequest {
    pub identity_token: String,
}

/// Response of `POST /shares`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShareResponse {
    pub share_id: String,
}
