use super::support::{Fixture, Response, credentials};

#[test]
fn saved_session_distinguishes_expiry_from_transient_failure() {
    let fixture = Fixture::new(&[r#"{"code":-101}"#, r#"{"code":-500,"message":"secret"}"#]);
    fixture.save_credentials();
    assert!(fixture.api.check_session().unwrap().is_none());
    assert!(fixture.api.check_session().is_err());
    assert_eq!(fixture.requests().len(), 2);
}

#[test]
fn saved_session_resolves_current_room_from_string_ids() {
    let fixture = Fixture::new(&[
        r#"{"code":0,"data":{"isLogin":true,"mid":"42","uname":"测试用户"}}"#,
        r#"{"code":0,"data":{"roomid":"456"}}"#,
    ]);
    fixture.save_credentials();
    let (c, account) = fixture.api.check_session().unwrap().unwrap();
    assert_eq!(account.mid, 42);
    assert_eq!(account.name, "测试用户");
    assert_eq!(c.room_id, 456);
    assert_eq!(c.sessdata, "session-safe");
    let requests = fixture.requests();
    assert!(requests[0].starts_with("GET /x/web-interface/nav HTTP/1.1\r\n"));
    assert!(requests[1].starts_with("GET /room/v1/Room/getRoomInfoOld?mid=42 HTTP/1.1\r\n"));
}

#[test]
fn missing_corrupt_and_unreadable_cookie_stores_remain_distinct() {
    let fixture = Fixture::new(&[]);
    assert!(fixture.api.check_session().unwrap().is_none());
    std::fs::write(&fixture.cookie_path, "not-json secret").unwrap();
    let error = fixture.api.check_session().err().unwrap();
    assert!(matches!(error, bili_live::error::BiliLiveError::Parse(_)));
    assert!(!format!("{error:?} {error}").contains("secret"));
    std::fs::remove_file(&fixture.cookie_path).unwrap();
    std::fs::create_dir(&fixture.cookie_path).unwrap();
    assert!(matches!(
        fixture.api.check_session(),
        Err(bili_live::error::BiliLiveError::Io(_))
    ));
    assert_eq!(fixture.requests(), [] as [std::string::String; 0]);
}

#[test]
fn login_uses_issued_cookies_and_persists_existing_shape_securely() {
    let fixture = Fixture::responses(vec![
        Response::json("")
            .header("Set-Cookie", "SESSDATA=issued-session; Path=/; HttpOnly")
            .header("Set-Cookie", "bili_jct=issued-csrf; Path=/; HttpOnly"),
        Response::json(r#"{"code":0,"data":{"isLogin":true,"mid":42}}"#),
        Response::json(r#"{"code":0,"data":{"roomid":"123"}}"#),
    ]);
    let (c, _account) = fixture
        .api
        .finish_login(&format!("{}/crossDomain?token=secret", fixture.origin))
        .unwrap();
    assert_eq!(c.room_id, 123);
    assert_eq!(c.sessdata, "issued-session");
    assert_eq!(c.csrf_token, "issued-csrf");
    let stored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&fixture.cookie_path).unwrap()).unwrap();
    assert_eq!(
        stored,
        serde_json::json!({"room_id":123,"sessdata":"issued-session","csrf_token":"issued-csrf"})
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&fixture.cookie_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let requests = fixture.requests();
    assert!(
        requests[1]
            .to_ascii_lowercase()
            .contains("cookie: sessdata=issued-session; bili_jct=issued-csrf")
    );
}

#[test]
fn crossdomain_rejects_external_insecure_and_credential_bearing_urls() {
    let fixture = Fixture::new(&[]);
    for url in [
        "http://passport.bilibili.com/login",
        "https://bilibili.com.evil.test/login",
        "https://evil.test/?bilibili.com",
        "https://user:pass@bilibili.com/login",
        "https://bilibili.com:444/login",
    ] {
        assert!(fixture.api.finish_login(url).is_err());
    }
    assert_eq!(fixture.requests(), [] as [std::string::String; 0]);
}

#[test]
fn login_validates_each_redirect_before_following() {
    let fixture = Fixture::responses(vec![
        Response::json("")
            .status(302)
            .header("Location", "https://evil.test/?token=secret"),
    ]);
    let error = fixture
        .api
        .finish_login(&format!("{}/crossDomain", fixture.origin))
        .err()
        .unwrap();
    assert!(!format!("{error:?} {error}").contains("secret"));
    assert_eq!(fixture.requests().len(), 1);
}

#[test]
fn qr_generation_rejects_empty_login_metadata() {
    let fixture = Fixture::new(&[
        r#"{"code":0,"data":{"url":"","qrcode_key":"key"}}"#,
        r#"{"code":0,"data":{"url":"https://passport.bilibili.com/qr","qrcode_key":"key"}}"#,
    ]);
    assert!(fixture.api.generate_qr().is_err());
    assert_eq!(fixture.api.generate_qr().unwrap().qrcode_key, "key");
    assert_eq!(fixture.requests().len(), 2);
}

#[test]
fn logged_out_saved_session_does_not_resolve_a_room() {
    let fixture = Fixture::new(&[r#"{"code":0,"data":{"isLogin":false}}"#]);
    fixture.save_credentials();
    assert!(fixture.api.check_session().unwrap().is_none());
    assert_eq!(fixture.requests().len(), 1);
}

#[test]
fn malformed_saved_credentials_do_not_make_network_requests() {
    let fixture = Fixture::new(&[]);
    let mut c = credentials();
    c.sessdata = "invalid;cookie".into();
    std::fs::write(&fixture.cookie_path, serde_json::to_vec(&c).unwrap()).unwrap();
    assert!(fixture.api.check_session().is_err());
    assert_eq!(fixture.requests(), [] as [std::string::String; 0]);
}
