use super::*;

#[test]
fn share_payload_round_trips_through_brotli() {
    let fixtures = [
        include_str!("../../../reference.jianpu"),
        include_str!("../../../simple.jianpu"),
        include_str!("../../../fixtures/follow_and_key.jianpu"),
        include_str!("../../../彌勒淨土鄉.jianpu"),
    ];
    for fixture in fixtures {
        let compressed = compress_share_payload(fixture);
        let decompressed = decompress_share_payload(&compressed);
        assert_eq!(decompressed.as_deref(), Some(fixture));
    }
}

#[test]
fn decompress_share_payload_rejects_garbage() {
    assert_eq!(decompress_share_payload(b"not brotli"), None);
}
