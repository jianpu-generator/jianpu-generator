//! Encoding and decoding the queue to and from stored records.

use crate::types::{
    DecodeError, FileRevision, Lane, LaneStatus, Queue, StoredRecord, QUEUE_SCHEMA_VERSION,
};
use itertools::Itertools;

const VERSION_KEY: &str = "meta:version";
const REVISIONS_KEY: &str = "meta:revisions";
const LANE_KEY_PREFIX: &str = "lane:";

fn lane_key(file_id: &str) -> String {
    format!("{LANE_KEY_PREFIX}{file_id}")
}

fn corrupt(key: &str) -> DecodeError {
    DecodeError::Corrupt {
        key: key.to_string(),
    }
}

/// Encodes the queue as records sorted by key: the schema version, the
/// recorded revisions, and one record per lane.
pub fn encode_queue(queue: &Queue) -> Vec<StoredRecord> {
    let meta = [
        StoredRecord {
            key: VERSION_KEY.to_string(),
            value: QUEUE_SCHEMA_VERSION.to_string(),
        },
        StoredRecord {
            key: REVISIONS_KEY.to_string(),
            value: serde_json::to_string(&queue.revisions).unwrap_or_else(|_| "[]".to_string()),
        },
    ];
    let lanes = queue.lanes.iter().filter_map(|lane| {
        serde_json::to_string(lane).ok().map(|value| StoredRecord {
            key: lane_key(&lane.file_id),
            value,
        })
    });
    meta.into_iter()
        .chain(lanes)
        .sorted_by(|a, b| a.key.cmp(&b.key))
        .collect()
}

/// Decodes records written by `encode_queue`, ignoring keys it does not own.
/// Interrupted work is recovered: every `in_flight` is cleared and `Waiting`
/// lanes become `Draining`. No owned records at all yields an empty queue.
pub fn decode_queue(records: &[StoredRecord]) -> Result<Queue, DecodeError> {
    let version_record = records.iter().find(|record| record.key == VERSION_KEY);
    let lane_records = records
        .iter()
        .filter(|record| record.key.starts_with(LANE_KEY_PREFIX))
        .collect_vec();
    let revisions_record = records.iter().find(|record| record.key == REVISIONS_KEY);

    let Some(version_record) = version_record else {
        return if lane_records.is_empty() && revisions_record.is_none() {
            Ok(Queue {
                lanes: Vec::new(),
                revisions: Vec::new(),
            })
        } else {
            Err(corrupt(VERSION_KEY))
        };
    };
    let found: u32 = version_record
        .value
        .parse()
        .map_err(|_| corrupt(VERSION_KEY))?;
    if found != QUEUE_SCHEMA_VERSION {
        return Err(DecodeError::UnsupportedVersion { found });
    }

    let revisions: Vec<FileRevision> = match revisions_record {
        Some(record) => serde_json::from_str(&record.value).map_err(|_| corrupt(REVISIONS_KEY))?,
        None => Vec::new(),
    };
    let lanes = lane_records
        .into_iter()
        .map(|record| {
            let lane: Lane =
                serde_json::from_str(&record.value).map_err(|_| corrupt(&record.key))?;
            if lane_key(&lane.file_id) != record.key {
                return Err(corrupt(&record.key));
            }
            Ok(recover_lane(lane))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Queue { lanes, revisions })
}

fn recover_lane(lane: Lane) -> Lane {
    let status = match lane.status {
        LaneStatus::Waiting { .. } => LaneStatus::Draining,
        other => other,
    };
    Lane {
        in_flight: None,
        status,
        ..lane
    }
}
