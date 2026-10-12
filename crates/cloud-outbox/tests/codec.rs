use cloud_outbox::codec::{decode_queue, encode_queue};
use cloud_outbox::types::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn lane(file_id: &str, status: LaneStatus, in_flight: Option<&str>) -> Lane {
    Lane {
        file_id: file_id.into(),
        status,
        in_flight: in_flight.map(String::from),
        messages: vec![QueuedMessage {
            message_id: "m1".into(),
            created_at_ms: 1,
            message: Message::SaveContent {
                content: "x".into(),
            },
            attempts: vec![Attempt {
                at_ms: 2,
                outcome: AttemptOutcome::Unauthorized,
            }],
        }],
    }
}

fn queue(lanes: Vec<Lane>) -> Queue {
    Queue {
        lanes,
        revisions: vec![FileRevision {
            file_id: "a".into(),
            revision: 3,
        }],
    }
}

macro_rules! expect_eq {
    ($left:expr, $right:expr $(,)?) => {{
        let (left, right) = (&$left, &$right);
        if left == right {
            Ok::<(), Box<dyn std::error::Error>>(())
        } else {
            Err(Box::<dyn std::error::Error>::from(format!(
                "mismatch: {left:?} != {right:?}"
            )))
        }
    }};
}

fn record(key: &str, value: &str) -> StoredRecord {
    StoredRecord {
        key: key.into(),
        value: value.into(),
    }
}

#[test]
fn roundtrip() -> TestResult {
    let original = queue(vec![
        lane("a", LaneStatus::Draining, None),
        lane("b", LaneStatus::NeedsSignIn, None),
        lane("c", LaneStatus::Failed { reason: "r".into() }, None),
    ]);
    expect_eq!(decode(&encode_queue(&original))?, original)
}

#[test]
fn records_are_sorted_by_key() -> TestResult {
    let encoded = encode_queue(&queue(vec![
        lane("b", LaneStatus::Idle, None),
        lane("a", LaneStatus::Idle, None),
    ]));
    let keys: Vec<&str> = encoded.iter().map(|r| r.key.as_str()).collect();
    expect_eq!(
        keys,
        vec!["lane:a", "lane:b", "meta:revisions", "meta:version"],
    )?;
    expect_eq!(encode_queue(&queue(vec![])), encode_queue(&queue(vec![])))
}

#[test]
fn unsupported_version_is_rejected() -> TestResult {
    expect_eq!(
        decode_queue(&[record("meta:version", "99")]),
        Err(DecodeError::UnsupportedVersion { found: 99 }),
    )
}

#[test]
fn corrupt_records_are_rejected() -> TestResult {
    let mut records = encode_queue(&queue(vec![lane("a", LaneStatus::Idle, None)]));
    records.push(record("lane:z", "not json"));
    expect_eq!(
        decode_queue(&records),
        Err(DecodeError::Corrupt {
            key: "lane:z".into(),
        }),
    )?;
    expect_eq!(
        decode_queue(&[record("meta:version", "1"), record("meta:revisions", "{")]),
        Err(DecodeError::Corrupt {
            key: "meta:revisions".into(),
        }),
    )
}

#[test]
fn in_flight_is_cleared_and_waiting_becomes_draining() -> TestResult {
    let original = queue(vec![
        lane("a", LaneStatus::Draining, Some("m1")),
        lane("b", LaneStatus::Waiting { retry_at_ms: 50 }, None),
    ]);
    let decoded = decode(&encode_queue(&original))?;
    expect_eq!(
        decoded.lanes,
        vec![
            lane("a", LaneStatus::Draining, None),
            lane("b", LaneStatus::Draining, None),
        ],
    )
}

#[test]
fn empty_queue_roundtrips_and_no_records_decode_empty() -> TestResult {
    let empty = Queue {
        lanes: vec![],
        revisions: vec![],
    };
    expect_eq!(decode(&encode_queue(&empty))?, empty.clone())?;
    expect_eq!(decode(&[])?, empty)
}

#[test]
fn foreign_keys_are_ignored() -> TestResult {
    let original = queue(vec![lane("a", LaneStatus::Idle, None)]);
    let mut records = encode_queue(&original);
    records.push(record("base:a", "whatever"));
    expect_eq!(decode(&records)?, original)
}

fn decode(records: &[StoredRecord]) -> Result<Queue, String> {
    decode_queue(records).map_err(|error| format!("{error:?}"))
}
