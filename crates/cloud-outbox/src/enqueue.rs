//! Appending messages to the queue.

use crate::types::{Lane, LaneStatus, Message, Queue, QueuedMessage, BATCH_LANE_PREFIX};

/// Appends `message` to the lane of `file_id`, creating the lane on first use.
/// A `SaveContent` replaces the lane's last message when that message is also
/// a `SaveContent` and is not currently in flight. An idle lane becomes
/// `Draining` so the scheduler picks the new message up. A `RenameFiles`
/// batch ignores `file_id`: it names its own files and gets a lane of its own
/// (`batch:<message id>`).
pub fn enqueue(
    queue: Queue,
    file_id: &str,
    message: Message,
    message_id: &str,
    now_ms: u64,
) -> Queue {
    let lane_id = match &message {
        Message::RenameFiles { .. } => format!("{BATCH_LANE_PREFIX}{message_id}"),
        _ => file_id.to_string(),
    };
    let file_id = lane_id.as_str();
    let queued = QueuedMessage {
        message_id: message_id.to_string(),
        created_at_ms: now_ms,
        message,
        attempts: Vec::new(),
    };
    let Queue {
        mut lanes,
        revisions,
    } = queue;
    match lanes.iter_mut().find(|lane| lane.file_id == file_id) {
        Some(lane) => push_onto_lane(lane, queued),
        None => lanes.push(Lane {
            file_id: file_id.to_string(),
            status: LaneStatus::Draining,
            in_flight: None,
            messages: vec![queued],
        }),
    }
    Queue { lanes, revisions }
}

fn push_onto_lane(lane: &mut Lane, queued: QueuedMessage) {
    let coalescible = matches!(queued.message, Message::SaveContent { .. })
        && lane.messages.last().is_some_and(|last| {
            matches!(last.message, Message::SaveContent { .. })
                && lane.in_flight.as_deref() != Some(last.message_id.as_str())
        });
    if coalescible {
        lane.messages.pop();
    }
    lane.messages.push(queued);
    if lane.status == LaneStatus::Idle {
        lane.status = LaneStatus::Draining;
    }
}
