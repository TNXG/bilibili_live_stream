use bili_live::model::{Area, AreaGroup, LiveSession, PushProtocol};
use serde_json::json;

fn protocol(name: &str, addr: &str, code: &str) -> PushProtocol {
    PushProtocol {
        protocol: name.into(),
        addr: addr.into(),
        code: code.into(),
        provider: String::new(),
        backup: false,
    }
}

#[test]
fn endpoints_join_address_and_code_for_supported_schemes() {
    assert_eq!(
        protocol("SRT", "rtmpsrt://live.example/app/", "secret")
            .endpoint()
            .as_deref(),
        Some("rtmpsrt://live.example/app/secret")
    );
    assert_eq!(
        protocol("rtmpsrt", "rtmpsrt://live.example/app/", "key")
            .endpoint()
            .as_deref(),
        Some("rtmpsrt://live.example/app/key")
    );
    assert_eq!(
        protocol("rtmp", "rtmp://live.example/app", "key")
            .endpoint()
            .as_deref(),
        Some("rtmp://live.example/app/key")
    );
}

#[test]
fn new_link_is_ignored_and_never_used_as_an_endpoint() {
    // The field still arrives from the server but the tool no longer uses it.
    let session: LiveSession = serde_json::from_value(json!({"protocols":[
        {"protocol":"rtmp","addr":"rtmp://live.example/app","code":"key",
         "new_link":"rtmp://fast.example/app/key"},
        {"protocol":"whip","addr":"https://live.example/ingest","code":"token",
         "new_link":"https://fast.example/ingest?token=secret"}
    ]}))
    .unwrap();
    assert_eq!(
        session.protocols[0].endpoint().as_deref(),
        Some("rtmp://live.example/app/key")
    );
    assert_eq!(
        session.protocols[1].endpoint().as_deref(),
        Some("https://live.example/ingest/token")
    );
}

#[test]
fn whip_uses_https_endpoint_not_a_guessed_bearer() {
    assert_eq!(
        protocol("WHIP", "https://live.example/whip", "")
            .endpoint()
            .as_deref(),
        Some("https://live.example/whip")
    );
    assert_eq!(
        protocol("whip", "https://live.example/ingest", "key")
            .endpoint()
            .as_deref(),
        Some("https://live.example/ingest/key")
    );
    assert!(
        protocol("whip", "https://live.example", "")
            .endpoint()
            .is_none()
    );
    assert!(
        protocol("whip", "http://live.example/whip", "")
            .endpoint()
            .is_none()
    );
}

#[test]
fn unknown_app_only_and_malformed_endpoints_are_not_synthesized() {
    for name in ["usb", "osi", "future"] {
        assert!(
            protocol(name, "rtmp://live.example/app", "key")
                .endpoint()
                .is_none()
        );
    }
    for addr in [
        "rtmp://",
        "javascript:alert(1)",
        "rtmp://user:password@live.example/app",
        "rtmp://live.example/app\n",
    ] {
        assert!(protocol("rtmp", addr, "key").endpoint().is_none());
    }
    assert!(
        protocol("rtmp", "rtmp://live.example/app", "")
            .endpoint()
            .is_none()
    );
}

#[test]
fn serialization_boundary_preserves_order_backups_and_raw_protocols() {
    let session: LiveSession = serde_json::from_value(json!({"live_key":18446744073709551615u64,
        "protocols":[null,{"protocol":"RTMP","addr":"rtmp://live.example/app","code":"first"},
            {"protocol":"srt","addr":"rtmpsrt://live.example/app","code":"second","provider":"edge"},
            {"protocol":"whip","addr":"https://live.example/whip/path"},
            {"protocol":"future","addr":"opaque"}],
        "protocols_backup":[{"protocol":"rtmp","addr":"rtmp://backup.example/app","code":"backup"}],
        "rtmp":{"addr":"rtmp://legacy.example/app","code":"unused"}})).unwrap();
    assert_eq!(session.live_key, "18446744073709551615");
    assert_eq!(session.protocols.len(), 5);
    assert_eq!(session.protocols[0].protocol, "RTMP");
    assert_eq!(session.protocols[1].provider, "edge");
    assert_eq!(
        session.protocols[2].endpoint().as_deref(),
        Some("https://live.example/whip/path")
    );
    assert!(session.protocols[3].endpoint().is_none());
    assert!(!session.protocols[3].backup);
    assert!(session.protocols[4].backup);
}

#[test]
fn optional_metadata_never_invalidates_successful_session_data() {
    for value in [
        json!(null),
        json!({}),
        json!({"live_key":{},"protocols":"wrong"}),
        json!({"live_key":"abc","protocols":[false,{"protocol":5},
            {"protocol":"srt","addr":42,"code":null}]}),
    ] {
        let session: LiveSession = serde_json::from_value(value).unwrap();
        assert!(session.protocols.iter().all(|p| p.endpoint().is_none()));
    }
    let session: LiveSession =
        serde_json::from_str(r#"{"live_key":"non-numeric-session"}"#).unwrap();
    assert_eq!(session.live_key, "non-numeric-session");
}

#[test]
fn legacy_fallback_is_separate_for_primary_and_backup() {
    let session: LiveSession = serde_json::from_value(json!({"protocols":[],
        "rtmp":{"addr":"rtmp://live.example/app","code":"primary"},
        "protocols_backup":null,"rtmp_backup":{"addr":"rtmp://backup.example/app","code":"backup"}})).unwrap();
    assert_eq!(session.protocols.len(), 2);
    assert_eq!(
        session.protocols[0].endpoint().as_deref(),
        Some("rtmp://live.example/app/primary")
    );
    assert_eq!(
        session.protocols[1].endpoint().as_deref(),
        Some("rtmp://backup.example/app/backup")
    );
    assert!(session.protocols[1].backup);
}

#[test]
fn ids_accept_decimal_strings_without_float_coercion() {
    let groups: Vec<AreaGroup> = serde_json::from_value(json!([
        {"id":"1","name":"games","list":[{"id":2,"name":"game"}]}]))
    .unwrap();
    assert_eq!(groups[0].list[0].id, 2);
    for id in [
        json!(-1),
        json!(1.5),
        json!("-2"),
        json!("18446744073709551616"),
    ] {
        assert!(serde_json::from_value::<Area>(json!({"id":id,"name":"area"})).is_err());
    }
}
