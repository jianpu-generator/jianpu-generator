//! Choosing the next message to send and when to wake.

use crate::types::{Lane, LaneStatus, Queue, SendRequest};

/// The queue after choosing a message, and the request to perform (if any).
#[derive(Debug, Clone, PartialEq)]
pub struct BeginSend {
    pub queue: Queue,
    pub request: Option<SendRequest>,
}

fn is_eligible(lane: &Lane, now_ms: u64) -> bool {
    !lane.messages.is_empty()
        && lane.in_flight.is_none()
        && match lane.status {
            LaneStatus::Idle | LaneStatus::Draining => true,
            LaneStatus::Waiting { retry_at_ms } => retry_at_ms <= now_ms,
            LaneStatus::NeedsSignIn | LaneStatus::NeedsMerge { .. } | LaneStatus::Failed { .. } => {
                false
            }
        }
}

/// Picks the eligible lane whose head is oldest and marks that head in flight.
pub fn begin_send(mut queue: Queue, now_ms: u64) -> BeginSend {
    let chosen = queue
        .lanes
        .iter()
        .enumerate()
        .filter(|(_, lane)| is_eligible(lane, now_ms))
        .min_by_key(|(_, lane)| lane.messages.first().map(|head| head.created_at_ms))
        .map(|(index, _)| index);
    let Some(index) = chosen else {
        return BeginSend {
            queue,
            request: None,
        };
    };
    let revisions = queue.revisions.clone();
    let Some(lane) = queue.lanes.get_mut(index) else {
        return BeginSend {
            queue,
            request: None,
        };
    };
    let expected_revision = revisions
        .iter()
        .find(|entry| entry.file_id == lane.file_id)
        .map_or(0, |entry| entry.revision);
    let request = lane.messages.first().map(|head| SendRequest {
        file_id: lane.file_id.clone(),
        message_id: head.message_id.clone(),
        message: head.message.clone(),
        expected_revision,
    });
    lane.in_flight = request.as_ref().map(|sent| sent.message_id.clone());
    lane.status = LaneStatus::Draining;
    BeginSend { queue, request }
}

/// The earliest retry time among `Waiting` lanes, if any.
pub fn next_wake_ms(queue: &Queue) -> Option<u64> {
    queue
        .lanes
        .iter()
        .filter_map(|lane| match lane.status {
            LaneStatus::Waiting { retry_at_ms } => Some(retry_at_ms),
            _ => None,
        })
        .min()
}
