//! Wire-shape tests for `protocol`, ported from the old
//! `live-share-worker/test/protocol.test.ts`.
#![allow(clippy::disallowed_macros)]

use live_share_worker::protocol::{
    FileShareRequest, FileShareResponse, ShareStatus, ShareStatusResponse, SyncedDoc,
};

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

#[test]
fn deserializes_a_file_share_request_from_camel_case_wire_fields() -> Result<(), serde_json::Error>
{
    let json = r#"{"identityToken":"tok-a"}"#;

    let request: FileShareRequest = serde_json::from_str(json)?;

    assert_eq!(request.identity_token, "tok-a");
    Ok(())
}

#[test]
fn serializes_a_file_share_response_with_camel_case_fields() -> Result<(), serde_json::Error> {
    let response = FileShareResponse {
        share_id: "abcdefghijk".to_string(),
    };

    assert_eq!(
        serde_json::to_string(&response)?,
        r#"{"shareId":"abcdefghijk"}"#
    );
    Ok(())
}

#[test]
fn serializes_a_share_status_response_with_a_share() -> Result<(), serde_json::Error> {
    let response = ShareStatusResponse {
        share: Some(ShareStatus {
            share_id: "abcdefghijk".to_string(),
            ended: true,
        }),
    };

    assert_eq!(
        serde_json::to_string(&response)?,
        r#"{"share":{"shareId":"abcdefghijk","ended":true}}"#
    );
    Ok(())
}

#[test]
fn serializes_a_share_status_response_without_a_share_as_null() -> Result<(), serde_json::Error> {
    let response = ShareStatusResponse { share: None };

    assert_eq!(serde_json::to_string(&response)?, r#"{"share":null}"#);
    Ok(())
}
