use super::support::{SIZES, live_app, render, render_buffer};
use bili_live::{
    app::{App, Screen},
    logging::LogBuffer,
    model::{Area, AreaGroup, PushProtocol},
    runtime::worker::Event,
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

#[test]
fn protocol_selection_scrolls_to_last_backup_without_exposing_its_endpoint() {
    let mut app = live_app();
    let protocols = &mut app.session.as_mut().unwrap().protocols;
    protocols.extend((0..40).map(|_| PushProtocol {
        protocol: "RTMP".into(),
        addr: "rtmp://PRIVATE_HOST/live".into(),
        code: "PRIVATE_CODE".into(),
        provider: "PRIVATE_PROVIDER".into(),
        backup: false,
    }));
    protocols.push(PushProtocol {
        protocol: "WHIP".into(),
        addr: "https://PRIVATE_HOST/ingest".into(),
        code: String::new(),
        provider: "PRIVATE_LAST_PROVIDER".into(),
        backup: true,
    });
    app.protocol_index = protocols.len() - 1;
    for (width, height) in SIZES {
        let output = render(&mut app, width, height);
        assert!(output.contains("WHIP"));
        assert!(output.contains("42"));
        assert!(!output.contains("PRIVATE"));
    }
    app.show_secrets = true;
    assert!(render(&mut app, 180, 40).contains("PRIVATE_LAST_PROVIDER"));
}

#[test]
fn selected_area_remains_visible_after_scrolling_both_panes() {
    let mut app = App::new(LogBuffer::new(), false);
    app.screen = Screen::Areas;
    app.groups = (0..30)
        .map(|id| AreaGroup {
            id,
            name: format!("group-{id}"),
            list: (0..60)
                .map(|id| Area {
                    id,
                    name: format!("area-{id}"),
                })
                .collect(),
        })
        .collect();
    app.group_index = 29;
    app.area_index = 59;
    app.recent_area = Some(Area {
        id: 7,
        name: "RECENT_SENTINEL".into(),
    });
    for (width, height) in SIZES {
        let output = render(&mut app, width, height);
        assert!(output.contains("group-29"));
        assert!(output.contains("area-59"));
        assert!(output.contains("RECENT_SENTINEL"));
    }
}

#[test]
fn log_scroll_reveals_older_entries() {
    let mut app = App::new(LogBuffer::new(), false);
    for index in 0..20 {
        app.logs
            .push(log::Level::Info, format!("LOG_SENTINEL_{index:02}"));
    }
    let newest = render(&mut app, 100, 32);
    assert!(newest.contains("LOG_SENTINEL_19"));
    assert!(!newest.contains("LOG_SENTINEL_00"));
    app.log_scroll = 19;
    let oldest = render(&mut app, 100, 32);
    assert!(oldest.contains("LOG_SENTINEL_00"));
    assert!(!oldest.contains("LOG_SENTINEL_19"));
}

fn qr_app() -> App {
    let mut app = App::new(LogBuffer::new(), false);
    app.handle_event(Event::Qr("https://example.invalid/login".into()));
    app
}

fn qr_bounds(buffer: &Buffer) -> Option<Rect> {
    let mut bounds: Option<Rect> = None;
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            if matches!(cell.fg, Color::Black | Color::White)
                && matches!(cell.bg, Color::Black | Color::White)
            {
                bounds = Some(match bounds {
                    None => Rect::new(x, y, 1, 1),
                    Some(previous) => Rect::new(
                        previous.x.min(x),
                        previous.y.min(y),
                        previous.right().max(x + 1) - previous.x.min(x),
                        previous.bottom().max(y + 1) - previous.y.min(y),
                    ),
                });
            }
        }
    }
    bounds
}

#[test]
fn generated_qr_expands_and_shrinks_without_losing_square_geometry() {
    let mut app = qr_app();
    let small = render_buffer(&mut app, 80, 30);
    let small_bounds = qr_bounds(&small).expect("a complete QR fits");
    let large = render_buffer(&mut app, 160, 60);
    let large_bounds = qr_bounds(&large).expect("a larger complete QR fits");
    assert!(large_bounds.width > small_bounds.width);
    assert!(large_bounds.height > small_bounds.height);
    for bounds in [small_bounds, large_bounds] {
        assert_eq!(bounds.width, 2 * bounds.height);
    }
    assert_eq!(render_buffer(&mut app, 80, 30), small);
}

#[test]
fn login_reclaims_logs_and_border_before_sacrificing_the_complete_qr() {
    let mut app = qr_app();
    app.logs
        .push(log::Level::Info, "LOG_SHOULD_NOT_DISPLACE_QR".into());
    let minimum = app.qr.as_ref().unwrap().minimum_size();
    let buffer = render_buffer(&mut app, minimum.width + 2, minimum.height + 5);
    let bounds = qr_bounds(&buffer).expect("minimum QR plus status and chrome fits");
    assert_eq!(bounds.width, minimum.width);
    assert_eq!(bounds.height, minimum.height);
    let output = super::support::text(&buffer);
    assert!(output.contains("手机端确认"));
    assert!(!output.contains("LOG_SHOULD_NOT_DISPLACE_QR"));
    assert!(bounds.y >= 2);
    assert!(bounds.bottom() <= buffer.area.height - 2);
}

#[test]
fn no_terminal_too_small_message_at_any_size() {
    let mut app = qr_app();
    for (width, height) in [(40, 12), (24, 30), (80, 8), (1, 1), (0, 0), (100, 32)] {
        let output = render(&mut app, width, height);
        assert!(
            !output.contains("终端过小"),
            "size {width}x{height}:\n{output}"
        );
        assert!(!output.contains("放大"), "size {width}x{height}");
    }
    // A roomy terminal renders the QR at a larger size instead of a hint.
    let small = qr_bounds(&render_buffer(&mut app, 80, 26)).expect("QR at 80x26");
    let large = qr_bounds(&render_buffer(&mut app, 100, 40)).expect("QR at 100x40");
    assert!(large.width >= small.width);
    assert_eq!(large.width, 2 * large.height);
}

#[test]
fn footer_buttons_dispatch_the_same_action_as_their_shortcut() {
    use bili_live::app::Effect;
    let mut app = App::new(LogBuffer::new(), false);
    app.handle_event(Event::Areas {
        room_id: 7,
        account: Some(bili_live::model::Account {
            mid: 9,
            name: "测试用户".into(),
        }),
        groups: vec![AreaGroup {
            id: 1,
            name: "游戏".into(),
            list: vec![Area {
                id: 42,
                name: "分区".into(),
            }],
        }],
        recent: Some(Area {
            id: 42,
            name: "分区".into(),
        }),
    });
    render(&mut app, 100, 32);
    let button = app
        .buttons
        .iter()
        .find(|button| button.key == ratatui::crossterm::event::KeyCode::Char('l'))
        .cloned()
        .expect("上次分区 button drawn");
    let click = app.handle_click(
        button.area.x + button.area.width / 2,
        button.area.y + button.area.height / 2,
    );
    assert!(matches!(
        click,
        Some(Effect::Request(
            bili_live::runtime::worker::Command::Start { area_id: 42, .. }
        ))
    ));
}

#[test]
fn login_status_stays_visible_in_the_log_panel() {
    let mut app = qr_app();
    app.handle_event(Event::LoginStatus("已扫码，请在手机上确认登录".into()));
    let output = render(&mut app, 100, 32);
    assert!(output.contains("已扫码"));
    assert!(output.contains("扫码登录"));
}

#[test]
fn quit_confirmation_buttons_are_clickable() {
    let mut app = qr_app();
    app.confirm_quit = true;
    render(&mut app, 100, 32);
    let cancel = app
        .buttons
        .iter()
        .find(|button| button.key == ratatui::crossterm::event::KeyCode::Char('n'))
        .cloned()
        .expect("cancel button drawn");
    assert!(app.handle_click(cancel.area.x, cancel.area.y).is_none());
    assert!(!app.confirm_quit);
}

#[test]
fn closed_room_summary_shows_units_without_duplicating_footer_hints() {
    let mut app = App::new(LogBuffer::new(), false);
    app.screen = Screen::Summary;
    app.status = "直播间已关闭".into();
    app.stats = Some(bili_live::model::LiveStats {
        seconds: 3661,
        viewers: 12,
        max_online: 3,
        danmaku: 45,
        fans: 6,
        fans_club: 1,
        revenue: 1.5,
    });
    let output = render(&mut app, 80, 24);
    assert!(output.contains("01:01:01"), "{output}");
    assert!(output.contains("本场收益1.50元"), "{output}");
    assert!(output.contains("最高在线3人"), "{output}");
    assert!(output.contains("弹幕45条"), "{output}");
    assert_eq!(output.matches("Enter返回分区").count(), 1);
    assert_eq!(output.matches("q退出").count(), 1);
}

#[test]
fn long_login_url_fits_the_login_panel_as_a_square_qr() {
    let mut app = App::new(LogBuffer::new(), false);
    // Real B站 QR URLs are long; the QR must still fit and stay square.
    app.handle_event(Event::Qr(
        "https://passport.bilibili.com/qrcode/h5/login?oauthKey=0123456789abcdef0123456789abcdef"
            .into(),
    ));
    let minimum = app.qr.as_ref().unwrap().minimum_size();
    let buffer = render_buffer(&mut app, 100, 32);
    let bounds = qr_bounds(&buffer).expect("long-payload QR fits 100x32");
    assert_eq!(bounds.width, 2 * bounds.height);
    assert!(bounds.height >= minimum.height);
}
