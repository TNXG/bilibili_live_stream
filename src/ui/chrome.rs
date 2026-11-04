use super::{ACCENT, MUTED, WARNING, widgets::panel};
use crate::app::{App, Button, Screen};
use ratatui::crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Clear, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

pub(super) fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let room = match (&app.account, app.room_id) {
        (Some(account), Some(id)) => {
            format!("  {} (UID {}) · 房间 {id}", account.name, account.mid)
        }
        (Some(account), None) => format!("  {} (UID {})", account.name, account.mid),
        (None, Some(id)) => format!("  房间 {id}"),
        (None, None) => String::new(),
    };
    let stage = match app.screen {
        Screen::Loading => "连接",
        Screen::Login => "扫码登录",
        Screen::Areas => "选择分区",
        Screen::Live => "推流信息",
        Screen::Summary => "关播结果",
    };
    let title = Line::from(vec![
        Span::styled(
            " B站 · 推流码工具 ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{stage}{room}")),
    ]);
    let status = Line::from(vec![
        Span::styled(
            if app.busy { " 处理中 · " } else { " " },
            Style::default().fg(WARNING),
        ),
        Span::raw(app.status.as_str()),
    ]);
    frame.render_widget(Paragraph::new(vec![title, status]), area);
}

pub(super) fn draw_logs(frame: &mut Frame, app: &App, area: Rect) {
    // Hold the bounded log buffer only while borrowing the visible entries.
    let entries = app.logs.entries();
    let count = usize::from(area.height.saturating_sub(2));
    let end = entries.len().saturating_sub(usize::from(app.log_scroll));
    let start = end.saturating_sub(count);
    let lines: Vec<_> = entries
        .iter()
        .skip(start)
        .take(end - start)
        .map(|entry| {
            let color = match entry.level {
                log::Level::Error => Color::Red,
                log::Level::Warn => WARNING,
                _ => MUTED,
            };
            Line::from(vec![
                Span::styled(
                    format!("{} {:5} ", entry.time, entry.level),
                    Style::default().fg(color),
                ),
                Span::raw(entry.message.as_str()),
            ])
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).block(panel(" 日志 · PgUp/PgDn 滚动 ", false)),
        area,
    );
}

/// Footer buttons per screen: (shortcut, label). The button mirrors the key.
fn footer_actions(screen: Screen) -> &'static [(KeyCode, &'static str)] {
    match screen {
        Screen::Loading => &[(KeyCode::Char('r'), "重试"), (KeyCode::Char('q'), "退出")],
        Screen::Login => &[
            (KeyCode::Char('r'), "刷新二维码"),
            (KeyCode::Char('q'), "退出"),
        ],
        Screen::Areas => &[
            (KeyCode::Char('l'), "上次分区"),
            (KeyCode::Enter, "开始直播"),
            (KeyCode::Char('r'), "刷新"),
            (KeyCode::Char('q'), "退出"),
        ],
        Screen::Live => &[
            (KeyCode::Char('h'), "显示/隐藏"),
            (KeyCode::Char('c'), "复制链接"),
            (KeyCode::Char('k'), "复制码"),
            (KeyCode::Char('a'), "复制地址"),
            (KeyCode::Char('r'), "刷新"),
            (KeyCode::Char('s'), "关播"),
            (KeyCode::Char('q'), "退出"),
        ],
        Screen::Summary => &[(KeyCode::Enter, "返回分区"), (KeyCode::Char('q'), "退出")],
    }
}

pub(super) fn draw_footer(frame: &mut Frame, app: &mut App, area: Rect) {
    let hint = match app.screen {
        Screen::Areas => "↑↓ 选择  ←→/Tab 切换列表",
        Screen::Live => "↑↓ 切换线路",
        Screen::Login => "请使用哔哩哔哩 App 扫码登录",
        Screen::Loading | Screen::Summary => "",
    };
    let buttons = footer_actions(app.screen);
    let mut x = area.x;
    let mut line = Vec::new();
    for (key, label) in buttons {
        let text = format!(" {} {} ", key_label(*key), label);
        // CJK labels are double-width: hit areas must use display columns.
        let width = u16::try_from(UnicodeWidthStr::width(text.as_str())).unwrap_or(0);
        if x + width > area.right() {
            break;
        }
        line.push(Span::styled(
            text,
            Style::default()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        ));
        line.push(Span::raw(" "));
        app.buttons
            .push(Button::new(Rect::new(x, area.y, width, 1), *key, label));
        x += width + 1;
    }
    let mut lines = vec![Line::from(line)];
    if area.height > 1 && !hint.is_empty() {
        lines.push(Line::from(Span::styled(hint, Style::default().fg(MUTED))));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn key_label(key: KeyCode) -> String {
    match key {
        KeyCode::Enter => "Enter".to_owned(),
        KeyCode::Char(character) => character.to_string(),
        _ => "?".to_owned(),
    }
}

pub(super) fn draw_confirmation(frame: &mut Frame, app: &mut App, area: Rect) {
    let width = area.width.min(64);
    let height = area.height.min(9);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);
    let title = if app.exit_after_stop {
        " 关播并退出？ "
    } else {
        " 确认关闭房间？ "
    };
    let (text, actions): (Text<'static>, &[(KeyCode, &str)]) = if app.busy {
        (
            Text::from(vec![
                Line::from("正在处理，请稍候…"),
                Line::from(app.status.clone()),
            ]),
            &[],
        )
    } else {
        (
            Text::from(vec![Line::from(if app.exit_after_stop {
                "当前正在直播，确定关播并退出吗？"
            } else {
                "确定要关闭当前直播间吗？"
            })]),
            &[
                (KeyCode::Char('y'), "确认关播"),
                (KeyCode::Char('n'), "取消"),
            ],
        )
    };
    let inner = Rect::new(
        popup.x + 1,
        popup.y + 1,
        popup.width.saturating_sub(2),
        popup.height.saturating_sub(2),
    );
    let text_height = u16::try_from(text.height()).unwrap_or(2);
    frame.render_widget(
        Paragraph::new(text)
            .block(panel(title, true))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left),
        popup,
    );
    let mut x = inner.x;
    let row = inner.y + text_height.min(inner.height.saturating_sub(1));
    for (key, label) in actions {
        let text = format!(" {} {} ", key_label(*key), label);
        let width = u16::try_from(UnicodeWidthStr::width(text.as_str())).unwrap_or(0);
        if x + width > inner.right() || row >= popup.bottom() {
            break;
        }
        frame.render_widget(
            Paragraph::new(Span::styled(
                text,
                Style::default().fg(Color::Black).bg(ACCENT),
            )),
            Rect::new(x, row, width, 1),
        );
        app.buttons
            .push(Button::new(Rect::new(x, row, width, 1), *key, label));
        x += width + 1;
    }
}
