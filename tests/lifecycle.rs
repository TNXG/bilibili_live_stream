use bili_live::{
    app::{App, Effect, Screen},
    logging::LogBuffer,
    model::{Area, AreaGroup, LiveSession, PushProtocol},
    runtime::worker::{Command, Event, Operation},
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

fn key(app: &mut App, code: KeyCode) -> Option<Effect> {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn ready() -> App {
    let mut app = App::new(LogBuffer::new(), false);
    app.handle_event(Event::Areas {
        room_id: 12,
        account: Some(bili_live::model::Account {
            mid: 7,
            name: "测试用户".into(),
        }),
        groups: vec![AreaGroup {
            id: 1,
            name: "游戏".into(),
            list: vec![
                Area {
                    id: 42,
                    name: "分区一".into(),
                },
                Area {
                    id: 43,
                    name: "分区二".into(),
                },
            ],
        }],
        recent: Some(Area {
            id: 42,
            name: "分区一".into(),
        }),
    });
    app
}

fn opened() -> App {
    let mut app = ready();
    key(&mut app, KeyCode::Char('l'));
    app.handle_event(Event::Started(LiveSession {
        live_key: "19".into(),
        protocols: vec![PushProtocol {
            protocol: "rtmp".into(),
            addr: "rtmp://live.example/app".into(),
            code: "private-code".into(),
            provider: "线路一".into(),
            backup: false,
        }],
    }));
    app
}

#[test]
fn quit_during_start_waits_for_result_then_requires_successful_stop() {
    let mut app = ready();
    assert!(matches!(
        key(&mut app, KeyCode::Char('l')),
        Some(Effect::Request(Command::Start { area_id: 42, .. }))
    ));
    assert!(key(&mut app, KeyCode::Char('q')).is_none());
    assert!(app.confirm_quit);
    assert!(key(&mut app, KeyCode::Char('y')).is_none());
    assert!(app.exit_after_stop);
    assert!(matches!(
        app.handle_event(Event::Started(LiveSession {
            live_key: String::new(),
            protocols: vec![],
        })),
        Some(Effect::Request(Command::Stop))
    ));
    assert!(app.session.is_some());
    assert!(matches!(
        app.handle_event(Event::Stopped),
        Some(Effect::Exit)
    ));
    assert!(app.session.is_none());
}

#[test]
fn failed_stop_keeps_codes_and_allows_retry_instead_of_exiting() {
    let mut app = opened();
    key(&mut app, KeyCode::Char('q'));
    assert!(matches!(
        key(&mut app, KeyCode::Char('y')),
        Some(Effect::Request(Command::Stop))
    ));
    assert!(
        app.handle_event(Event::Failed {
            operation: Operation::Stop,
            error: "network failure".into()
        })
        .is_none()
    );
    assert!(app.session.is_some());
    assert_eq!(app.screen, Screen::Live);
    assert!(!app.exit_after_stop);
    assert!(matches!(
        key(&mut app, KeyCode::Char('s')),
        Some(Effect::Request(Command::Stop))
    ));
}

#[test]
fn statistics_failure_cannot_resurrect_a_stopped_room() {
    let mut app = opened();
    key(&mut app, KeyCode::Char('s'));
    app.handle_event(Event::Stopped);
    app.handle_event(Event::Failed {
        operation: Operation::Stats,
        error: "unavailable".into(),
    });
    assert!(app.session.is_none());
    assert_eq!(app.screen, Screen::Summary);
    assert!(matches!(
        key(&mut app, KeyCode::Char('q')),
        Some(Effect::Exit)
    ));
}

#[test]
fn quit_during_refresh_still_stops_when_refresh_fails() {
    let mut app = opened();
    assert!(matches!(
        key(&mut app, KeyCode::Char('r')),
        Some(Effect::Request(Command::Refresh { area_id: 42 }))
    ));
    key(&mut app, KeyCode::Char('q'));
    key(&mut app, KeyCode::Char('y'));
    assert!(matches!(
        app.handle_event(Event::Failed {
            operation: Operation::Refresh,
            error: "unavailable".into()
        }),
        Some(Effect::Request(Command::Stop))
    ));
    assert!(app.session.is_some());
}

#[test]
fn clipboard_gets_full_endpoint_not_masked_display_text() {
    let mut app = opened();
    assert!(!app.show_secrets);
    let Some(Effect::Copy(value)) = key(&mut app, KeyCode::Char('c')) else {
        panic!("missing copy");
    };
    assert_eq!(value, "rtmp://live.example/app/private-code");
    let Some(Effect::Copy(value)) = key(&mut app, KeyCode::Char('k')) else {
        panic!("missing code");
    };
    assert_eq!(value, "private-code");
}

#[test]
fn unsupported_record_cannot_be_copied_as_a_network_endpoint() {
    let mut app = opened();
    app.session.as_mut().unwrap().protocols[0].protocol = "usb".into();
    for code in ['a', 'k', 'c'] {
        assert!(key(&mut app, KeyCode::Char(code)).is_none());
    }
}

#[test]
fn changing_parent_category_resets_child_selection_before_start() {
    let mut app = ready();
    app.groups.push(AreaGroup {
        id: 2,
        name: "生活".into(),
        list: vec![Area {
            id: 50,
            name: "日常".into(),
        }],
    });
    key(&mut app, KeyCode::Right);
    key(&mut app, KeyCode::Down);
    key(&mut app, KeyCode::Left);
    key(&mut app, KeyCode::Down);
    key(&mut app, KeyCode::Right);
    assert!(matches!(
        key(&mut app, KeyCode::Enter),
        Some(Effect::Request(Command::Start { area_id: 50, .. }))
    ));
}

#[test]
fn release_and_repeated_activation_cannot_double_submit_start() {
    let mut app = ready();
    app.handle_key(KeyEvent::new_with_kind(
        KeyCode::Char('l'),
        KeyModifiers::NONE,
        KeyEventKind::Release,
    ));
    assert!(!app.busy);
    assert!(matches!(
        key(&mut app, KeyCode::Char('l')),
        Some(Effect::Request(Command::Start { .. }))
    ));
    assert!(key(&mut app, KeyCode::Char('l')).is_none());
    assert!(key(&mut app, KeyCode::Enter).is_none());
}

#[test]
fn late_statistics_failure_cannot_complete_a_newer_start_request() {
    let mut app = opened();
    key(&mut app, KeyCode::Char('s'));
    app.handle_event(Event::Stopped);
    key(&mut app, KeyCode::Enter);
    assert!(matches!(
        key(&mut app, KeyCode::Char('l')),
        Some(Effect::Request(Command::Start { .. }))
    ));
    app.handle_event(Event::Failed {
        operation: Operation::Stats,
        error: "previous-session statistics unavailable".into(),
    });
    assert!(
        app.busy,
        "statistics failure must not complete the pending start"
    );
    assert!(key(&mut app, KeyCode::Char('l')).is_none());
    assert!(key(&mut app, KeyCode::Char('q')).is_none());
    assert!(app.confirm_quit);
    key(&mut app, KeyCode::Char('y'));
    assert!(matches!(
        app.handle_event(Event::Started(LiveSession {
            live_key: "20".into(),
            protocols: vec![],
        })),
        Some(Effect::Request(Command::Stop))
    ));
}

#[test]
fn late_statistics_success_does_not_leak_into_the_next_session_summary() {
    let mut app = opened();
    key(&mut app, KeyCode::Char('s'));
    app.handle_event(Event::Stopped);
    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Char('l'));
    app.handle_event(Event::Stats(bili_live::model::LiveStats {
        fans: 99,
        ..Default::default()
    }));
    app.handle_event(Event::Started(LiveSession {
        live_key: "20".into(),
        protocols: vec![],
    }));
    key(&mut app, KeyCode::Char('s'));
    app.handle_event(Event::Stopped);
    app.handle_event(Event::Failed {
        operation: Operation::Stats,
        error: "new-session statistics unavailable".into(),
    });
    assert!(
        app.stats.is_none(),
        "must not show the previous session's totals"
    );
}

#[test]
fn refresh_keeps_records_from_either_response_without_duplicating() {
    let mut app = opened();
    let before = app.session.as_ref().unwrap().protocols.len();
    app.handle_event(Event::Refreshed(vec![
        PushProtocol {
            protocol: "rtmp".into(),
            addr: "rtmp://live.example/app".into(),
            code: "private-code".into(),
            provider: "线路一".into(),
            backup: false,
        },
        PushProtocol {
            protocol: "srt".into(),
            addr: "rtmpsrt://srt.example/live".into(),
            code: "srt-code".into(),
            provider: String::new(),
            backup: false,
        },
    ]));
    let protocols = &app.session.as_ref().unwrap().protocols;
    assert_eq!(
        protocols.len(),
        before + 1,
        "duplicate must not be re-added"
    );
    assert!(protocols.iter().any(|p| p.protocol == "srt"));
}

#[test]
fn wheel_scrolls_logs_over_the_log_pane_and_moves_lists_elsewhere() {
    let mut app = opened();
    app.log_area = ratatui::layout::Rect::new(0, 20, 80, 6);
    app.handle_scroll(10, 22, true);
    assert_eq!(app.log_scroll, 3);
    app.handle_scroll(10, 22, false);
    assert_eq!(app.log_scroll, 0);
    app.protocol_index = 0;
    app.session.as_mut().unwrap().protocols.push(PushProtocol {
        protocol: "srt".into(),
        addr: "rtmpsrt://srt.example/live".into(),
        code: "code".into(),
        provider: String::new(),
        backup: false,
    });
    app.handle_scroll(10, 5, false);
    assert_eq!(app.protocol_index, 1, "wheel over the list moves selection");
}
