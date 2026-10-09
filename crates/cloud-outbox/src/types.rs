//! Data types shared by every outbox module.

use serde::{Deserialize, Serialize};

pub const QUEUE_SCHEMA_VERSION: u32 = 1;
pub const POSSIBLY_STUCK_AFTER: u32 = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Queue {
    pub lanes: Vec<Lane>,
    pub revisions: Vec<FileRevision>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRevision {
    pub file_id: String,
    pub revision: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lane {
    pub file_id: String,
    pub status: LaneStatus,
    /// The message id currently being sent.
    pub in_flight: Option<String>,
    /// In order; the head is index 0.
    pub messages: Vec<QueuedMessage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedMessage {
    pub message_id: String,
    pub created_at_ms: u64,
    pub message: Message,
    pub attempts: Vec<Attempt>,
}

/// The file id lives on the lane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Message {
    CreateFile { name: String, content: String },
    SaveContent { content: String },
    RenameFile { to: String },
    TrashFile,
    RestoreFile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub at_ms: u64,
    pub outcome: AttemptOutcome,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AttemptOutcome {
    Transient {
        reason: String,
    },
    Unauthorized,
    #[serde(rename_all = "camelCase")]
    RevisionConflict {
        current_revision: i64,
    },
    #[serde(rename_all = "camelCase")]
    NameTaken {
        suggested_name: String,
    },
    Permanent {
        status: u16,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaneStatus {
    Idle,
    Draining,
    #[serde(rename_all = "camelCase")]
    Waiting {
        retry_at_ms: u64,
    },
    NeedsSignIn,
    #[serde(rename_all = "camelCase")]
    NeedsMerge {
        current_revision: i64,
    },
    Failed {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendRequest {
    pub file_id: String,
    pub message_id: String,
    pub message: Message,
    /// 0 if no revision recorded.
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SendResult {
    Ok {
        revision: i64,
    },
    Transient {
        reason: String,
    },
    Unauthorized,
    #[serde(rename_all = "camelCase")]
    Conflict {
        current_revision: i64,
    },
    #[serde(rename_all = "camelCase")]
    NameTaken {
        suggested_name: String,
    },
    Permanent {
        status: u16,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Resolution {
    /// Waiting, NeedsMerge, Failed -> Draining.
    Retry,
    /// Drop the whole lane and its recorded revision.
    Discard,
    #[serde(rename_all = "camelCase")]
    MergedAndSave {
        content: String,
        server_revision: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRecord {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DecodeError {
    UnsupportedVersion { found: u32 },
    Corrupt { key: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MergeOutcome {
    Clean {
        text: String,
    },
    #[serde(rename_all = "camelCase")]
    Conflicted {
        text: String,
        has_markers: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaneReason {
    Syncing,
    WaitingToRetry,
    PossiblyStuck,
    NeedsSignIn,
    NeedsMerge,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneSummary {
    pub file_id: String,
    pub reason: LaneReason,
    pub pending_messages: u32,
    pub retry_in_ms: Option<u64>,
    /// The head message's attempts.
    pub attempts: Vec<Attempt>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSummary {
    /// Lanes with at least one message.
    pub lanes: Vec<LaneSummary>,
    pub total_pending: u32,
    /// NeedsSignIn + NeedsMerge + Failed + PossiblyStuck.
    pub lanes_needing_attention: u32,
    /// Any lane has messages, halted or not.
    pub has_unsynced: bool,
}
