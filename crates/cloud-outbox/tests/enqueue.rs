use cloud_outbox::enqueue::enqueue;
use cloud_outbox::types::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn empty() -> Queue {
    Queue {
        lanes: vec![],
        revisions: vec![],
    }
}

fn save(content: &str) -> Message {
    Message::SaveContent {
        content: content.into(),
    }
}

fn lane<'a>(queue: &'a Queue, file_id: &str) -> Result<&'a Lane, String> {
    queue
        .lanes
        .iter()
        .find(|lane| lane.file_id == file_id)
        .ok_or_else(|| format!("no lane {file_id}"))
}

fn ids(lane: &Lane) -> Vec<&str> {
    lane.messages
        .iter()
        .map(|m| m.message_id.as_str())
        .collect()
}

fn expect<T: PartialEq + std::fmt::Debug>(actual: &T, expected: &T) -> TestResult {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {expected:?}, got {actual:?}").into())
    }
}

#[test]
fn enqueue_creates_lane_on_first_use() -> TestResult {
    let queue = enqueue(empty(), "f1", save("a"), "m1", 10);
    let l = lane(&queue, "f1")?;
    expect(&ids(l), &vec!["m1"])?;
    expect(&l.messages[0].created_at_ms, &10)?;
    expect(&l.messages[0].attempts, &vec![])?;
    expect(&l.in_flight, &None)?;
    expect(&l.status, &LaneStatus::Draining)
}

#[test]
fn enqueue_appends_in_order() -> TestResult {
    let queue = enqueue(empty(), "f1", Message::TrashFile, "m1", 1);
    let queue = enqueue(queue, "f1", Message::RestoreFile, "m2", 2);
    let queue = enqueue(queue, "f1", Message::RenameFile { to: "x".into() }, "m3", 3);
    expect(&ids(lane(&queue, "f1")?), &vec!["m1", "m2", "m3"])
}

#[test]
fn enqueue_coalesces_save_into_pending_save() -> TestResult {
    let queue = enqueue(empty(), "f1", save("a"), "m1", 1);
    let queue = enqueue(queue, "f1", save("b"), "m2", 2);
    let l = lane(&queue, "f1")?;
    expect(&ids(l), &vec!["m2"])?;
    expect(&l.messages[0].message, &save("b"))
}

#[test]
fn enqueue_does_not_coalesce_into_in_flight_head() -> TestResult {
    let mut queue = enqueue(empty(), "f1", save("a"), "m1", 1);
    queue.lanes[0].in_flight = Some("m1".into());
    let queue = enqueue(queue, "f1", save("b"), "m2", 2);
    expect(&ids(lane(&queue, "f1")?), &vec!["m1", "m2"])
}

#[test]
fn enqueue_does_not_coalesce_across_other_kinds() -> TestResult {
    let queue = enqueue(empty(), "f1", save("a"), "m1", 1);
    let queue = enqueue(queue, "f1", Message::RenameFile { to: "n".into() }, "m2", 2);
    let queue = enqueue(queue, "f1", save("b"), "m3", 3);
    expect(&ids(lane(&queue, "f1")?), &vec!["m1", "m2", "m3"])
}

#[test]
fn enqueue_lanes_are_independent() -> TestResult {
    let queue = enqueue(empty(), "f1", save("a"), "m1", 1);
    let queue = enqueue(queue, "f2", save("b"), "m2", 2);
    let queue = enqueue(queue, "f1", save("c"), "m3", 3);
    expect(&queue.lanes.len(), &2)?;
    expect(&ids(lane(&queue, "f1")?), &vec!["m3"])?;
    expect(&ids(lane(&queue, "f2")?), &vec!["m2"])
}
