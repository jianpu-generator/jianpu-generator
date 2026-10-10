use cloud_outbox::summary::summarize;
use cloud_outbox::types::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn transient(at_ms: u64) -> Attempt {
    Attempt {
        at_ms,
        outcome: AttemptOutcome::Transient {
            reason: "offline".into(),
        },
    }
}

fn lane(file_id: &str, status: LaneStatus, attempts: Vec<Attempt>) -> Lane {
    Lane {
        file_id: file_id.into(),
        status,
        in_flight: None,
        messages: vec![QueuedMessage {
            message_id: "m1".into(),
            created_at_ms: 0,
            message: Message::SaveContent {
                content: "x".into(),
            },
            attempts,
        }],
    }
}

fn queue_of(lanes: Vec<Lane>) -> Queue {
    Queue {
        lanes,
        revisions: vec![],
    }
}

macro_rules! expect {
    ($actual:expr, $expected:expr $(,)?) => {{
        let actual = $actual;
        let expected = $expected;
        if actual == expected {
            Ok::<(), Box<dyn std::error::Error>>(())
        } else {
            Err(format!("expected {expected:?}, got {actual:?}").into())
        }
    }};
}

fn single_reason(lane: Lane) -> Result<LaneReason, Box<dyn std::error::Error>> {
    let summary = summarize(&queue_of(vec![lane]), 100);
    let first = summary.lanes.into_iter().next().ok_or("no lane summary")?;
    Ok(first.reason)
}

#[test]
fn summary_empty_queue() -> TestResult {
    let summary = summarize(&queue_of(vec![]), 0);
    expect!(summary.lanes, vec![])?;
    expect!(summary.total_pending, 0)?;
    expect!(summary.lanes_needing_attention, 0)?;
    expect!(summary.has_unsynced, false)
}

#[test]
fn summary_lane_without_messages_is_omitted() -> TestResult {
    let mut empty = lane("a", LaneStatus::Idle, vec![]);
    empty.messages.clear();
    let summary = summarize(&queue_of(vec![empty]), 0);
    expect!(summary.lanes.len(), 0)?;
    expect!(summary.has_unsynced, false)
}

#[test]
fn summary_reason_syncing() -> TestResult {
    expect!(
        single_reason(lane("a", LaneStatus::Draining, vec![]))?,
        LaneReason::Syncing,
    )?;
    expect!(
        single_reason(lane("a", LaneStatus::Idle, vec![]))?,
        LaneReason::Syncing,
    )
}

#[test]
fn summary_reason_waiting_to_retry_with_countdown() -> TestResult {
    let queue = queue_of(vec![lane(
        "a",
        LaneStatus::Waiting { retry_at_ms: 150 },
        vec![transient(10)],
    )]);
    let summary = summarize(&queue, 100);
    let first = summary.lanes.first().ok_or("no lane")?;
    expect!(first.reason.clone(), LaneReason::WaitingToRetry)?;
    expect!(first.retry_in_ms, Some(50))?;
    expect!(first.attempts.len(), 1)?;
    expect!(summary.lanes_needing_attention, 0)
}

#[test]
fn summary_retry_in_saturates_at_zero() -> TestResult {
    let queue = queue_of(vec![lane(
        "a",
        LaneStatus::Waiting { retry_at_ms: 50 },
        vec![],
    )]);
    let summary = summarize(&queue, 100);
    expect!(summary.lanes.first().ok_or("no lane")?.retry_in_ms, Some(0))
}

#[test]
fn summary_reason_needs_sign_in() -> TestResult {
    expect!(
        single_reason(lane("a", LaneStatus::NeedsSignIn, vec![]))?,
        LaneReason::NeedsSignIn,
    )
}

#[test]
fn summary_reason_needs_merge() -> TestResult {
    expect!(
        single_reason(lane(
            "a",
            LaneStatus::NeedsMerge {
                current_revision: 3,
            },
            vec![],
        ))?,
        LaneReason::NeedsMerge,
    )
}

#[test]
fn summary_reason_failed_carries_reason() -> TestResult {
    let queue = queue_of(vec![lane(
        "a",
        LaneStatus::Failed {
            reason: "bad".into(),
        },
        vec![],
    )]);
    let summary = summarize(&queue, 0);
    let first = summary.lanes.first().ok_or("no lane")?;
    expect!(first.reason.clone(), LaneReason::Failed)?;
    expect!(first.failure_reason.clone(), Some("bad".to_string()))?;
    expect!(summary.lanes_needing_attention, 1)
}

#[test]
fn summary_stuck_threshold_at_nine_attempts() -> TestResult {
    let attempts = (0..9).map(transient).collect();
    expect!(
        single_reason(lane(
            "a",
            LaneStatus::Waiting { retry_at_ms: 200 },
            attempts,
        ))?,
        LaneReason::WaitingToRetry,
    )
}

#[test]
fn summary_stuck_threshold_at_ten_attempts() -> TestResult {
    let attempts = (0..10).map(transient).collect();
    let queue = queue_of(vec![lane(
        "a",
        LaneStatus::Waiting { retry_at_ms: 200 },
        attempts,
    )]);
    let summary = summarize(&queue, 100);
    expect!(
        summary.lanes.first().ok_or("no lane")?.reason.clone(),
        LaneReason::PossiblyStuck,
    )?;
    expect!(summary.lanes_needing_attention, 1)
}

#[test]
fn summary_non_transient_attempt_in_window_is_not_stuck() -> TestResult {
    let mut attempts: Vec<Attempt> = (0..10).map(transient).collect();
    attempts[5] = Attempt {
        at_ms: 5,
        outcome: AttemptOutcome::Unauthorized,
    };
    expect!(
        single_reason(lane(
            "a",
            LaneStatus::Waiting { retry_at_ms: 200 },
            attempts,
        ))?,
        LaneReason::WaitingToRetry,
    )
}

#[test]
fn summary_has_unsynced_with_only_halted_lane() -> TestResult {
    let summary = summarize(
        &queue_of(vec![lane("a", LaneStatus::NeedsSignIn, vec![])]),
        0,
    );
    expect!(summary.has_unsynced, true)?;
    expect!(summary.total_pending, 1)
}

#[test]
fn summary_totals_across_lanes() -> TestResult {
    let mut two = lane("b", LaneStatus::Draining, vec![]);
    two.messages.push(two.messages[0].clone());
    let queue = queue_of(vec![lane("a", LaneStatus::NeedsSignIn, vec![]), two]);
    let summary = summarize(&queue, 0);
    expect!(summary.total_pending, 3)?;
    expect!(summary.lanes_needing_attention, 1)?;
    expect!(summary.lanes.len(), 2)
}
