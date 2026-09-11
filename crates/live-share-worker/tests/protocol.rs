//! Wire-shape tests for `protocol`, ported from the old
//! `live-share-worker/test/protocol.test.ts`.
#![allow(clippy::disallowed_macros)]

use live_share_worker::protocol::{SyncedDoc, SyncedWriteRequest};

#[test]
fn deserializes_an_update_request_from_camel_case_wire_fields() -> Result<(), serde_json::Error> {
    let json = r#"{"type":"update","identityToken":"tok-a","filename":"song.jianpu","content":"[M] 1","revision":1}"#;

    let request: SyncedWriteRequest = serde_json::from_str(json)?;

    assert_eq!(request.identity_token(), "tok-a");
    assert!(matches!(request, SyncedWriteRequest::Update { .. }));
    Ok(())
}

#[test]
fn deserializes_a_stop_request_from_camel_case_wire_fields() -> Result<(), serde_json::Error> {
    let json = r#"{"type":"stop","identityToken":"tok-a"}"#;

    let request: SyncedWriteRequest = serde_json::from_str(json)?;

    assert_eq!(request.identity_token(), "tok-a");
    assert!(matches!(request, SyncedWriteRequest::Stop { .. }));
    Ok(())
}

#[test]
fn rejects_a_write_request_with_an_unknown_type_tag() {
    let json = r#"{"type":"delete","identityToken":"tok-a"}"#;

    let result: Result<SyncedWriteRequest, serde_json::Error> = serde_json::from_str(json);

    assert!(result.is_err());
}

#[test]
fn serializes_synced_doc_with_camel_case_fields() -> Result<(), serde_json::Error> {
    let doc = SyncedDoc {
        ended: false,
        filename: "song.jianpu".to_string(),
        content: "[M] 1".to_string(),
        revision: 1,
        owner_login: Some("octocat".to_string()),
    };

    let json = serde_json::to_string(&doc)?;

    assert!(json.contains("\"filename\":\"song.jianpu\""));
    assert!(json.contains("\"revision\":1"));
    assert!(json.contains("\"ownerLogin\":\"octocat\""));
    Ok(())
}

#[test]
fn serializes_synced_doc_owner_login_as_null_when_absent() -> Result<(), serde_json::Error> {
    let doc = SyncedDoc {
        ended: true,
        filename: String::new(),
        content: String::new(),
        revision: 0,
        owner_login: None,
    };

    let json = serde_json::to_string(&doc)?;

    assert!(json.contains("\"ownerLogin\":null"));
    Ok(())
}
