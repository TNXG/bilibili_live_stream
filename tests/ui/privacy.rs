use super::support::{SIZES, live_app, render};
use bili_live::runtime::worker::Event;

#[test]
fn hides_every_sensitive_protocol_component_at_all_supported_sizes() {
    let mut app = live_app();
    for (width, height) in SIZES {
        let output = render(&mut app, width, height);
        assert!(!output.contains("PRIVATE"));
        assert!(output.contains("WHIP"));
    }
}

#[test]
fn secret_visibility_is_explicit_and_reversible() {
    let mut app = live_app();
    app.show_secrets = true;
    let visible = render(&mut app, 180, 40);
    for sentinel in ["PRIVATE_HOST", "PRIVATE_CODE", "PRIVATE_PROVIDER"] {
        assert!(
            visible.contains(sentinel),
            "missing revealed field: {sentinel}"
        );
    }
    assert!(!visible.contains("PRIVATE_LIVE_KEY"));
    app.show_secrets = false;
    assert!(!render(&mut app, 180, 40).contains("PRIVATE"));
}

#[test]
fn qr_login_never_displays_the_token_as_text() {
    let mut app = live_app();
    app.handle_event(Event::Qr(
        "https://example.invalid/PRIVATE_LOGIN_TOKEN".into(),
    ));
    for (width, height) in SIZES {
        let output = render(&mut app, width, height);
        assert!(!output.contains("PRIVATE_LOGIN_TOKEN"));
    }
}
