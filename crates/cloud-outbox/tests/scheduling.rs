use cloud_outbox::scheduling::{begin_send, next_wake_ms};
use cloud_outbox::types::*;

type TestResult = Result<(), String>;

fn lane(file_id: &str, status: LaneStatus, created: &[u64]) -> Lane {
    Lane {
        file_id: file_id.into(),
        status,
        in_flight: None,
        messages: created
            .iter()
            .map(|&created_at_ms| QueuedMessage {
                message_id: format!("{file_id}-{created_at_ms}"),
                created_at_ms,
                message: Message::SaveContent {
                    content: "x".into(),
                },
                attempts: vec![],
            })
            .collect(),
    }
}

fn queue(lanes: Vec<Lane>) -> Queue {
    Queue {
        lanes,
        revisions: vec![],
    }
}

macro_rules! expect {
    ($actual:expr, $wanted:expr $(,)?) => {{
        let (actual, wanted) = ($actual, $wanted);
        if actual == wanted {
            Ok(())
        } else {
            Err(format!("expected {wanted:?}, got {actual:?}"))
        }
    }};
}

fn sent_id(queue: Queue, now: u64) -> Option<String> {
    begin_send(queue, now).request.map(|r| r.message_id)
}

#[test]
fn picks_oldest_head_across_lanes() -> TestResult {
    let q = queue(vec![
        lane("a", LaneStatus::Idle, &[30]),
        lane("b", LaneStatus::Draining, &[10, 5]),
        lane("c", LaneStatus::Idle, &[20]),
    ]);
    expect!(sent_id(q, 100), Some("b-10".into()))
}

#[test]
fn ties_go_to_lane_order() -> TestResult {
    let q = queue(vec![
        lane("a", LaneStatus::Idle, &[10]),
        lane("b", LaneStatus::Idle, &[10]),
    ]);
    expect!(sent_id(q, 100), Some("a-10".into()))
}

#[test]
fn marks_head_in_flight_and_draining() -> TestResult {
    let q = queue(vec![lane("a", LaneStatus::Idle, &[1, 2])]);
    let result = begin_send(q, 100);
    let first = &result.queue.lanes[0];
    expect!(first.in_flight.clone(), Some("a-1".into()))?;
    expect!(first.status.clone(), LaneStatus::Draining)?;
    expect!(first.messages.len(), 2)
}

#[test]
fn does_not_double_send_in_flight_head() -> TestResult {
    let q = queue(vec![lane("a", LaneStatus::Idle, &[1])]);
    let once = begin_send(q, 100);
    let twice = begin_send(once.queue, 100);
    expect!(twice.request, None)
}

#[test]
fn skips_ineligible_lanes() -> TestResult {
    let q = queue(vec![
        lane("a", LaneStatus::Waiting { retry_at_ms: 500 }, &[1]),
        lane("b", LaneStatus::NeedsSignIn, &[2]),
        lane(
            "c",
            LaneStatus::NeedsMerge {
                current_revision: 3,
            },
            &[3],
        ),
        lane("d", LaneStatus::Failed { reason: "r".into() }, &[4]),
        lane("e", LaneStatus::Idle, &[]),
        lane("f", LaneStatus::Idle, &[50]),
    ]);
    let result = begin_send(q.clone(), 100);
    expect!(result.request.map(|r| r.message_id), Some("f-50".into()))?;
    let only_blocked = Queue {
        lanes: q.lanes[..5].to_vec(),
        revisions: vec![],
    };
    let none = begin_send(only_blocked.clone(), 100);
    expect!(none.request, None)?;
    expect!(none.queue, only_blocked)
}

#[test]
fn due_waiting_lane_is_sent_and_becomes_draining() -> TestResult {
    let q = queue(vec![lane(
        "a",
        LaneStatus::Waiting { retry_at_ms: 100 },
        &[1],
    )]);
    let result = begin_send(q, 100);
    expect!(result.request.map(|r| r.message_id), Some("a-1".into()))?;
    expect!(result.queue.lanes[0].status.clone(), LaneStatus::Draining)
}

#[test]
fn expected_revision_is_looked_up_or_zero() -> TestResult {
    let mut q = queue(vec![
        lane("a", LaneStatus::Idle, &[1]),
        lane("b", LaneStatus::Idle, &[2]),
    ]);
    q.revisions = vec![FileRevision {
        file_id: "a".into(),
        revision: 7,
    }];
    let first = begin_send(q, 100);
    expect!(first.request.map(|r| r.expected_revision), Some(7))?;
    let second = begin_send(first.queue, 100);
    expect!(second.request.map(|r| r.expected_revision), Some(0))
}

#[test]
fn next_wake_is_earliest_waiting_or_none() -> TestResult {
    let q = queue(vec![
        lane("a", LaneStatus::Waiting { retry_at_ms: 900 }, &[1]),
        lane("b", LaneStatus::Waiting { retry_at_ms: 400 }, &[2]),
        lane("c", LaneStatus::Idle, &[3]),
    ]);
    expect!(next_wake_ms(&q), Some(400))?;
    expect!(
        next_wake_ms(&queue(vec![lane("c", LaneStatus::Idle, &[3])])),
        None,
    )
}
