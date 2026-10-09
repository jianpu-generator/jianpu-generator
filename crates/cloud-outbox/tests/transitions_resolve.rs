use cloud_outbox::transitions::{on_result, record_revision, resolve, resolve_signed_in};
use cloud_outbox::types::*;

type TestResult = Result<(), String>;

#[allow(clippy::needless_pass_by_value)] // call sites pass temporaries
fn check<T: PartialEq + std::fmt::Debug>(actual: T, expected: T, label: &str) -> TestResult {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label}: expected {expected:?}, got {actual:?}"))
    }
}

fn message(id: &str, body: Message) -> QueuedMessage {
    QueuedMessage {
        message_id: id.into(),
        created_at_ms: 1,
        message: body,
        attempts: vec![],
    }
}

fn save(id: &str, content: &str) -> QueuedMessage {
    message(
        id,
        Message::SaveContent {
            content: content.into(),
        },
    )
}

fn queue_with(messages: Vec<QueuedMessage>, status: LaneStatus, in_flight: Option<&str>) -> Queue {
    Queue {
        lanes: vec![
            Lane {
                file_id: "f".into(),
                status,
                in_flight: in_flight.map(String::from),
                messages,
            },
            Lane {
                file_id: "other".into(),
                status: LaneStatus::Draining,
                in_flight: None,
                messages: vec![save("o1", "x")],
            },
        ],
        revisions: vec![
            FileRevision {
                file_id: "f".into(),
                revision: 4,
            },
            FileRevision {
                file_id: "other".into(),
                revision: 9,
            },
        ],
    }
}

fn in_flight_queue(messages: Vec<QueuedMessage>) -> Queue {
    queue_with(messages, LaneStatus::Draining, Some("m1"))
}

fn lane(queue: &Queue) -> Result<&Lane, String> {
    queue
        .lanes
        .iter()
        .find(|lane| lane.file_id == "f")
        .ok_or_else(|| "lane f missing".to_string())
}

fn revision_of(queue: &Queue, file_id: &str) -> Option<i64> {
    queue
        .revisions
        .iter()
        .find(|entry| entry.file_id == file_id)
        .map(|entry| entry.revision)
}

fn other_untouched(queue: &Queue) -> TestResult {
    let other = queue
        .lanes
        .iter()
        .find(|lane| lane.file_id == "other")
        .ok_or("other lane missing")?;
    check(other.messages.len(), 1, "other lane messages")?;
    check(revision_of(queue, "other"), Some(9), "other revision")
}

#[test]
fn retry_from_halted_states_drains() -> TestResult {
    for status in [
        LaneStatus::Waiting { retry_at_ms: 9 },
        LaneStatus::NeedsMerge {
            current_revision: 3,
        },
        LaneStatus::Failed { reason: "x".into() },
    ] {
        let queue = resolve(
            queue_with(vec![save("m1", "a")], status.clone(), None),
            "f",
            Resolution::Retry,
        );
        check(
            lane(&queue)?.status.clone(),
            LaneStatus::Draining,
            &format!("{status:?}"),
        )?;
        check(lane(&queue)?.in_flight.clone(), None, "in flight")?;
    }
    Ok(())
}

#[test]
fn retry_ignored_from_other_states() -> TestResult {
    for status in [
        LaneStatus::Idle,
        LaneStatus::NeedsSignIn,
        LaneStatus::Draining,
    ] {
        let start = queue_with(vec![save("m1", "a")], status, None);
        check(
            resolve(start.clone(), "f", Resolution::Retry),
            start,
            "unchanged",
        )?;
    }
    Ok(())
}

#[test]
fn discard_drops_lane_and_revision() -> TestResult {
    let queue = resolve(
        queue_with(
            vec![save("m1", "a"), save("m2", "b")],
            LaneStatus::NeedsMerge {
                current_revision: 3,
            },
            None,
        ),
        "f",
        Resolution::Discard,
    );
    check(queue.lanes.len(), 1, "lanes")?;
    check(revision_of(&queue, "f"), None, "revision")?;
    other_untouched(&queue)
}

#[test]
fn merged_and_save_replaces_head() -> TestResult {
    let mut head = save("m1", "old");
    head.attempts = vec![Attempt {
        at_ms: 1,
        outcome: AttemptOutcome::RevisionConflict {
            current_revision: 6,
        },
    }];
    let queue = resolve(
        queue_with(
            vec![head, save("m2", "next")],
            LaneStatus::NeedsMerge {
                current_revision: 6,
            },
            None,
        ),
        "f",
        Resolution::MergedAndSave {
            content: "merged".into(),
            server_revision: 6,
        },
    );
    let lane = lane(&queue)?;
    check(lane.status.clone(), LaneStatus::Draining, "status")?;
    check(lane.messages.len(), 2, "count")?;
    check(
        lane.messages[0].message.clone(),
        Message::SaveContent {
            content: "merged".into(),
        },
        "head",
    )?;
    check(lane.messages[0].attempts.clone(), vec![], "fresh attempts")?;
    check(
        lane.messages[0].message_id.clone(),
        "m1".to_string(),
        "id kept",
    )?;
    check(revision_of(&queue, "f"), Some(6), "revision")
}

#[test]
fn merged_and_save_ignored_outside_needs_merge() -> TestResult {
    let start = queue_with(vec![save("m1", "a")], LaneStatus::NeedsSignIn, None);
    check(
        resolve(
            start.clone(),
            "f",
            Resolution::MergedAndSave {
                content: "x".into(),
                server_revision: 1,
            },
        ),
        start,
        "unchanged",
    )
}

#[test]
fn resolve_signed_in_resumes_only_sign_in_lanes() -> TestResult {
    let queue = resolve_signed_in(queue_with(
        vec![save("m1", "a")],
        LaneStatus::NeedsSignIn,
        None,
    ));
    check(lane(&queue)?.status.clone(), LaneStatus::Draining, "status")?;
    let failed = queue_with(
        vec![save("m1", "a")],
        LaneStatus::Failed { reason: "x".into() },
        None,
    );
    check(
        resolve_signed_in(failed.clone()),
        failed,
        "failed untouched",
    )
}

#[test]
fn record_revision_upserts() -> TestResult {
    let queue = record_revision(in_flight_queue(vec![save("m1", "a")]), "f", 20);
    check(revision_of(&queue, "f"), Some(20), "replaced")?;
    let queue = record_revision(queue, "new", 1);
    check(revision_of(&queue, "new"), Some(1), "added")?;
    check(queue.revisions.len(), 3, "count")
}

#[test]
fn permanent_fails_the_lane() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![save("m1", "a")]),
        "f",
        "m1",
        SendResult::Permanent {
            status: 400,
            reason: "bad".into(),
        },
        5,
        0.0,
    );
    check(
        lane(&queue)?.status.clone(),
        LaneStatus::Failed {
            reason: "bad".into(),
        },
        "status",
    )?;
    check(lane(&queue)?.in_flight.clone(), None, "in flight")?;
    check(lane(&queue)?.messages.len(), 1, "message kept")
}

#[test]
fn stale_result_is_ignored() -> TestResult {
    let start = in_flight_queue(vec![save("m1", "a")]);
    let wrong_message = on_result(
        start.clone(),
        "f",
        "m9",
        SendResult::Ok { revision: 1 },
        1,
        0.0,
    );
    check(wrong_message, start.clone(), "wrong message id")?;
    let wrong_file = on_result(
        start.clone(),
        "nope",
        "m1",
        SendResult::Ok { revision: 1 },
        1,
        0.0,
    );
    check(wrong_file, start, "unknown file")?;
    let idle = queue_with(vec![save("m1", "a")], LaneStatus::Draining, None);
    check(
        on_result(idle.clone(), "f", "m1", SendResult::Unauthorized, 1, 0.0),
        idle,
        "not in flight",
    )
}
