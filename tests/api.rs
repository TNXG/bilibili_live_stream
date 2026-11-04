#[path = "support/auth.rs"]
mod auth;
mod support;

use bili_live::api::Api;
use support::{Fixture, Response, credentials};

#[test]
fn successful_start_without_urls_still_allows_real_stop_request() {
    let fixture = Fixture::new(&[
        r#"{"code":0,"data":{"protocols":"malformed"}}"#,
        r#"{"code":0}"#,
    ]);
    let session = fixture.api.start(&credentials(), 456, false).unwrap();
    assert_eq!(session.live_key, "");
    assert!(session.protocols.is_empty());
    fixture.api.stop(&credentials()).unwrap();
    let requests = fixture.requests();
    assert!(requests[0].starts_with("POST /room/v1/Room/startLive HTTP/1.1\r\n"));
    assert!(requests[0].ends_with("room_id=123&area_v2=456&csrf=csrf%2Bvalue&platform=pc_link"));
    assert!(!requests[0].contains("recommend_protocol"));
    assert!(
        requests[0]
            .to_ascii_lowercase()
            .contains("cookie: sessdata=session-safe; bili_jct=csrf+value")
    );
    assert!(requests[1].starts_with("POST /room/v1/Room/stopLive HTTP/1.1\r\n"));
}

#[test]
fn poll_encodes_key_and_api_errors_never_echo_server_secrets() {
    let fixture = Fixture::new(&[
        r#"{"code":0,"data":{"code":86101}}"#,
        r#"{"code":-403,"message":"SESSDATA=secret; push=rtmp://secret","data":null}"#,
    ]);
    assert_eq!(
        fixture.api.poll_qr("key&injected=value").unwrap().code,
        86101
    );
    let error = fixture.api.generate_qr().err().unwrap();
    assert!(!format!("{error:?} {error}").contains("secret"));
    assert!(fixture.requests()[0].starts_with(
        "GET /x/passport-login/web/qrcode/poll?qrcode_key=key%26injected%3Dvalue HTTP/1.1\r\n"
    ));
}

#[test]
fn refresh_fetches_all_protocols_and_stats_encode_live_key() {
    let fixture = Fixture::new(&[
        r#"{"code":0,"data":{"protocols":[{"protocol":"rtmp","addr":"rtmp://host/app","code":"key"},{"protocol":"srt","addr":"rtmpsrt://host/app","code":"key"},{"protocol":"whip","addr":"https://host/whip"}],"protocols_backup":[{"protocol":"rtmp","addr":"rtmp://backup/app","code":"backup"}]}}"#,
        r#"{"code":0,"data":{"AddFans":"2","DanmuNum":3,"LiveTime":60,"WatchedCount":100}}"#,
    ]);
    let protocols = fixture.api.refresh(&credentials(), 456).unwrap();
    assert_eq!(protocols.len(), 4);
    assert!(protocols[3].backup);
    assert_eq!(
        protocols[2].endpoint().as_deref(),
        Some("https://host/whip")
    );
    let stats = fixture.api.stats(&credentials(), "key&next=value").unwrap();
    assert_eq!(
        (stats.fans, stats.danmaku, stats.seconds, stats.viewers),
        (2, 3, 60, 100)
    );
    assert_eq!(stats.revenue, 0.0);
    let requests = fixture.requests();
    assert!(requests[0].starts_with(
        "GET /xlive/app-blink/v1/live/GetUpStreamRtmp?room_id=123&area_v2=456 HTTP/1.1\r\n"
    ));
    assert!(requests[1].starts_with(
        "GET /xlive/app-blink/v1/live/StopLiveData?live_key=key%26next%3Dvalue HTTP/1.1\r\n"
    ));
}

#[test]
fn malformed_envelopes_and_counters_hide_response_text() {
    for body in [
        r#"{"data":{"token":"secret"}}"#,
        r#"{"code":"0","data":{}}"#,
        "not-json secret",
        r#"{"code":-403,"message":"SESSDATA=secret"}"#,
        r#"{"code":0,"data":{"LiveTime":"bad secret"}}"#,
    ] {
        let fixture = Fixture::new(&[body]);
        let error = fixture.api.stats(&credentials(), "key").err().unwrap();
        assert!(!format!("{error:?} {error}").contains("secret"));
        assert_eq!(fixture.requests().len(), 1);
    }
}

#[test]
fn http_failure_and_invalid_credentials_never_expose_secrets() {
    let fixture = Fixture::responses(vec![Response::json("secret").status(503)]);
    let error = fixture.api.poll_qr("secret").err().unwrap();
    assert!(!format!("{error:?} {error}").contains("secret"));
    assert_eq!(fixture.requests().len(), 1);
    let fixture = Fixture::new(&[]);
    for (room_id, sessdata, csrf_token) in [
        (0, "valid", "csrf"),
        (1, "valid", ""),
        (1, "a; injected=1", "csrf"),
        (1, "valid", "csrf\r\nheader"),
    ] {
        let mut c = credentials();
        c.room_id = room_id;
        c.sessdata = sessdata.into();
        c.csrf_token = csrf_token.into();
        assert!(fixture.api.stop(&c).is_err());
    }
    assert!(fixture.api.start(&credentials(), 0, false).is_err());
    assert!(fixture.api.stats(&credentials(), "").is_err());
    assert_eq!(fixture.requests(), [] as [std::string::String; 0]);
}

#[test]
fn area_endpoints_accept_string_ids_and_include_authenticated_room() {
    let fixture = Fixture::new(&[
        r#"{"code":0,"data":[{"id":"1","name":"games","list":[{"id":2,"name":"game"}]}]}"#,
        r#"{"code":0,"data":[{"id":0,"name":"none"},{"id":"2","name":"game"}]}"#,
    ]);
    assert_eq!(fixture.api.areas().unwrap()[0].list[0].id, 2);
    assert_eq!(
        fixture.api.recent_area(&credentials()).unwrap().unwrap().id,
        2
    );
    let requests = fixture.requests();
    assert!(requests[0].starts_with("GET /room/v1/Area/getList HTTP/1.1\r\n"));
    assert!(requests[1].starts_with("GET /room/v1/Area/getMyChooseArea?roomid=123 HTTP/1.1\r\n"));
}

#[test]
fn explicit_origins_reject_nonliteral_or_unsafe_loopback_configuration() {
    for origin in [
        "http://localhost:8080",
        "https://example.test",
        "http://127.0.0.1:9/path",
        "http://u:p@127.0.0.1:9",
        "http://127.0.0.1:9?secret=x",
        "http://127.1:9",
        "http://2130706433:9",
        "http://127.0.0.1:9/#fragment",
    ] {
        let error = Api::with_origins(
            origin,
            "http://127.0.0.1:9",
            "http://[::1]:9",
            "unused.json",
        )
        .err()
        .unwrap();
        assert!(!format!("{error:?} {error}").contains("secret"));
    }
    assert!(
        Api::with_origins(
            "http://127.0.0.1:9",
            "http://[::1]:9",
            "http://127.0.0.1:9",
            "unused.json"
        )
        .is_ok()
    );
}

#[test]
fn login_cross_domain_hosts_cover_bilibili_and_biligame_only() {
    for host in [
        "passport.bilibili.com",
        "bilibili.com",
        "passport.biligame.com",
    ] {
        assert!(bili_live::api::login_host_allowed(host), "{host}");
    }
    for host in [
        "evilbilibili.com",
        "bilibili.com.evil.test",
        "biligame.com.evil.test",
        "example.com",
        "",
    ] {
        assert!(!bili_live::api::login_host_allowed(host), "{host}");
    }
}

#[test]
fn refresh_reports_app_only_endpoint_without_leaking_secrets() {
    let fixture = Fixture::new(&[r#"{"code":-400,"message":"请求错误 SESSDATA=secret"}"#]);
    let error = fixture.api.refresh(&credentials(), 456).err().unwrap();
    let text = format!("{error}");
    assert!(text.contains("-400"), "{text}");
    assert!(!text.contains("secret"));
}

#[test]
fn srt_hint_uses_speed_test_node_ip() {
    let fixture = Fixture::new(&[
        r#"{"code":0,"data":{"nodes":[
            {"protocol":"rtmp","addr":"rtmp://node.example/live","ip":"203.0.113.7"},
            {"protocol":"srt","addr":"rtmpsrt://node.example/live","ip":"203.0.113.9"}]}}"#,
        r#"{"code":0,"data":{"live_key":"77","protocols":[
            {"protocol":"rtmpsrt","addr":"rtmpsrt://srt.example/live","code":"srt-secret"}]}}"#,
    ]);
    let session = fixture.api.start(&credentials(), 456, true).unwrap();
    let requests = fixture.requests();
    assert!(
        requests[0].starts_with(
            "GET /xlive/app-blink/v1/upStreamConfig/GetUpStreamSpeedTestInfo?room_id=123&platform=pc_link&build=7120300&area_v2_id=456&free_tab=0 HTTP/1.1\r\n"
        ),
        "{}",
        requests[0]
    );
    assert!(requests[1].starts_with("POST /room/v1/Room/startLive HTTP/1.1\r\n"));
    assert!(requests[1].contains("recommend_protocol=srt"));
    assert!(requests[1].contains("recommend_ip=203.0.113.9"));
    assert_eq!(session.protocols[0].protocol, "rtmpsrt");
    assert_eq!(
        session.protocols[0].endpoint().as_deref(),
        Some("rtmpsrt://srt.example/live/srt-secret")
    );
}

#[test]
fn start_without_srt_nodes_still_opens_room() {
    let fixture = Fixture::new(&[
        r#"{"code":0,"data":{"nodes":[{"protocol":"rtmp","addr":"rtmp://node.example/live","ip":"203.0.113.7"}]}}"#,
        r#"{"code":0,"data":{"protocols":[{"protocol":"rtmp","addr":"rtmp://host/app","code":"key"}]}}"#,
    ]);
    let session = fixture.api.start(&credentials(), 456, true).unwrap();
    let requests = fixture.requests();
    assert!(requests[1].starts_with("POST /room/v1/Room/startLive HTTP/1.1\r\n"));
    assert!(!requests[1].contains("recommend_protocol"));
    assert!(!requests[1].contains("recommend_ip"));
    assert_eq!(session.protocols.len(), 1);
}

#[test]
fn speed_test_failure_does_not_block_start() {
    let fixture = Fixture::new(&[
        r#"{"code":-400,"message":"请求错误"}"#,
        r#"{"code":0,"data":{"protocols":[{"protocol":"rtmp","addr":"rtmp://host/app","code":"key"}]}}"#,
    ]);
    fixture.api.start(&credentials(), 456, true).unwrap();
    let requests = fixture.requests();
    assert!(requests[0].contains("GetUpStreamSpeedTestInfo"));
    assert!(!requests[1].contains("recommend_protocol"));
}
