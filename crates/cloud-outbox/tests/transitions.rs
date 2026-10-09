use cloud_outbox::backoff::backoff_ms;
use cloud_outbox::transitions::{on_result, resolve};
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

fn head_attempts(queue: &Queue) -> Result<Vec<Attempt>, String> {
    Ok(lane(queue)?
        .messages
        .first()
        .ok_or("no head")?
        .attempts
        .clone())
}

#[test]
fn ok_with_more_messages_drains() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![save("m1", "a"), save("m2", "b")]),
        "f",
        "m1",
        SendResult::Ok { revision: 7 },
        100,
        0.0,
    );
    let lane = lane(&queue)?;
    check(lane.messages.len(), 1, "remaining")?;
    check(lane.status.clone(), LaneStatus::Draining, "status")?;
    check(lane.in_flight.clone(), None, "in flight")?;
    check(revision_of(&queue, "f"), Some(7), "revision")?;
    other_untouched(&queue)
}

#[test]
fn ok_with_last_message_goes_idle() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![save("m1", "a")]),
        "f",
        "m1",
        SendResult::Ok { revision: 7 },
        100,
        0.0,
    );
    check(lane(&queue)?.status.clone(), LaneStatus::Idle, "status")?;
    check(lane(&queue)?.messages.len(), 0, "messages")
}

#[test]
fn ok_adds_revision_when_none_recorded() -> TestResult {
    let mut start = in_flight_queue(vec![save("m1", "a")]);
    start.revisions.retain(|entry| entry.file_id != "f");
    let queue = on_result(start, "f", "m1", SendResult::Ok { revision: 2 }, 1, 0.0);
    check(revision_of(&queue, "f"), Some(2), "revision")
}

#[test]
fn transient_waits_with_backoff() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![save("m1", "a")]),
        "f",
        "m1",
        SendResult::Transient {
            reason: "net".into(),
        },
        1_000,
        0.5,
    );
    check(
        lane(&queue)?.status.clone(),
        LaneStatus::Waiting {
            retry_at_ms: 1_000 + backoff_ms(1, 0.5),
        },
        "status",
    )?;
    check(lane(&queue)?.in_flight.clone(), None, "in flight")?;
    check(
        head_attempts(&queue)?,
        vec![Attempt {
            at_ms: 1_000,
            outcome: AttemptOutcome::Transient {
                reason: "net".into(),
            },
        }],
        "attempts",
    )?;
    check(revision_of(&queue, "f"), Some(4), "revision kept")
}

#[test]
fn consecutive_transients_increase_backoff() -> TestResult {
    let mut queue = in_flight_queue(vec![save("m1", "a")]);
    for round in 1..=3u32 {
        queue = on_result(
            queue,
            "f",
            "m1",
            SendResult::Transient {
                reason: "net".into(),
            },
            10,
            0.0,
        );
        check(
            lane(&queue)?.status.clone(),
            LaneStatus::Waiting {
                retry_at_ms: 10 + backoff_ms(round, 0.0),
            },
            &format!("round {round}"),
        )?;
        queue = resolve(queue, "f", Resolution::Retry);
        queue.lanes[0].in_flight = Some("m1".into());
    }
    Ok(())
}

#[test]
fn non_transient_attempt_resets_the_count() -> TestResult {
    let mut head = save("m1", "a");
    head.attempts = vec![
        Attempt {
            at_ms: 1,
            outcome: AttemptOutcome::Transient { reason: "a".into() },
        },
        Attempt {
            at_ms: 2,
            outcome: AttemptOutcome::Unauthorized,
        },
    ];
    let queue = on_result(
        in_flight_queue(vec![head]),
        "f",
        "m1",
        SendResult::Transient { reason: "b".into() },
        0,
        0.0,
    );
    check(
        lane(&queue)?.status.clone(),
        LaneStatus::Waiting {
            retry_at_ms: backoff_ms(1, 0.0),
        },
        "status",
    )
}

#[test]
fn unauthorized_needs_sign_in() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![save("m1", "a")]),
        "f",
        "m1",
        SendResult::Unauthorized,
        5,
        0.0,
    );
    check(
        lane(&queue)?.status.clone(),
        LaneStatus::NeedsSignIn,
        "status",
    )?;
    check(lane(&queue)?.in_flight.clone(), None, "in flight")?;
    check(
        head_attempts(&queue)?,
        vec![Attempt {
            at_ms: 5,
            outcome: AttemptOutcome::Unauthorized,
        }],
        "attempts",
    )
}

#[test]
fn conflict_needs_merge() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![save("m1", "a")]),
        "f",
        "m1",
        SendResult::Conflict {
            current_revision: 12,
        },
        5,
        0.0,
    );
    check(
        lane(&queue)?.status.clone(),
        LaneStatus::NeedsMerge {
            current_revision: 12,
        },
        "status",
    )?;
    check(
        head_attempts(&queue)?,
        vec![Attempt {
            at_ms: 5,
            outcome: AttemptOutcome::RevisionConflict {
                current_revision: 12,
            },
        }],
        "attempts",
    )
}

#[test]
fn name_taken_rewrites_create_file() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![message(
            "m1",
            Message::CreateFile {
                name: "a".into(),
                content: "c".into(),
            },
        )]),
        "f",
        "m1",
        SendResult::NameTaken {
            suggested_name: "a (2)".into(),
        },
        5,
        0.0,
    );
    check(lane(&queue)?.status.clone(), LaneStatus::Draining, "status")?;
    check(lane(&queue)?.in_flight.clone(), None, "in flight")?;
    check(
        lane(&queue)?.messages[0].message.clone(),
        Message::CreateFile {
            name: "a (2)".into(),
            content: "c".into(),
        },
        "message",
    )?;
    check(
        head_attempts(&queue)?,
        vec![Attempt {
            at_ms: 5,
            outcome: AttemptOutcome::NameTaken {
                suggested_name: "a (2)".into(),
            },
        }],
        "attempts",
    )
}

#[test]
fn name_taken_rewrites_rename_file() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![message("m1", Message::RenameFile { to: "a".into() })]),
        "f",
        "m1",
        SendResult::NameTaken {
            suggested_name: "a (2)".into(),
        },
        5,
        0.0,
    );
    check(lane(&queue)?.status.clone(), LaneStatus::Draining, "status")?;
    check(
        lane(&queue)?.messages[0].message.clone(),
        Message::RenameFile { to: "a (2)".into() },
        "message",
    )
}

#[test]
fn name_taken_on_other_message_is_permanent() -> TestResult {
    let queue = on_result(
        in_flight_queue(vec![save("m1", "a")]),
        "f",
        "m1",
        SendResult::NameTaken {
            suggested_name: "z".into(),
        },
        5,
        0.0,
    );
    check(
        lane(&queue)?.status.clone(),
        LaneStatus::Failed {
            reason: "name_taken".into(),
        },
        "status",
    )?;
    check(
        head_attempts(&queue)?,
        vec![Attempt {
            at_ms: 5,
            outcome: AttemptOutcome::Permanent {
                status: 409,
                reason: "name_taken".into(),
            },
        }],
        "attempts",
    )
}
