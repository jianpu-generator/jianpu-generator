//! Summarizing the queue for display.
use crate::types::{
    Attempt, AttemptOutcome, Lane, LaneReason, LaneStatus, LaneSummary, Queue, QueueSummary,
    POSSIBLY_STUCK_AFTER,
};

/// Builds the user-facing summary of every lane that still has messages.
pub fn summarize(queue: &Queue, now_ms: u64) -> QueueSummary {
    let lanes: Vec<LaneSummary> = queue
        .lanes
        .iter()
        .filter(|lane| !lane.messages.is_empty())
        .map(|lane| summarize_lane(lane, now_ms))
        .collect();
    let total_pending = lanes.iter().map(|lane| lane.pending_messages).sum();
    let lanes_needing_attention = lanes
        .iter()
        .filter(|lane| needs_attention(&lane.reason))
        .count() as u32;
    QueueSummary {
        has_unsynced: !lanes.is_empty(),
        lanes,
        total_pending,
        lanes_needing_attention,
    }
}

fn needs_attention(reason: &LaneReason) -> bool {
    !matches!(reason, LaneReason::Syncing | LaneReason::WaitingToRetry)
}

fn summarize_lane(lane: &Lane, now_ms: u64) -> LaneSummary {
    let attempts: Vec<Attempt> = lane
        .messages
        .first()
        .map(|head| head.attempts.clone())
        .unwrap_or_default();
    let possibly_stuck = is_possibly_stuck(&attempts);
    let reason = match &lane.status {
        LaneStatus::NeedsSignIn => LaneReason::NeedsSignIn,
        LaneStatus::NeedsMerge { .. } => LaneReason::NeedsMerge,
        LaneStatus::Failed { .. } => LaneReason::Failed,
        _ if possibly_stuck => LaneReason::PossiblyStuck,
        LaneStatus::Waiting { .. } => LaneReason::WaitingToRetry,
        LaneStatus::Idle | LaneStatus::Draining => LaneReason::Syncing,
    };
    let retry_in_ms = match &lane.status {
        LaneStatus::Waiting { retry_at_ms } => Some(retry_at_ms.saturating_sub(now_ms)),
        _ => None,
    };
    let failure_reason = match &lane.status {
        LaneStatus::Failed { reason } => Some(reason.clone()),
        _ => None,
    };
    LaneSummary {
        file_id: lane.file_id.clone(),
        reason,
        pending_messages: lane.messages.len() as u32,
        retry_in_ms,
        attempts,
        failure_reason,
    }
}

fn is_possibly_stuck(attempts: &[Attempt]) -> bool {
    let threshold = POSSIBLY_STUCK_AFTER as usize;
    attempts.len() >= threshold
        && attempts
            .iter()
            .rev()
            .take(threshold)
            .all(|attempt| matches!(attempt.outcome, AttemptOutcome::Transient { .. }))
}
