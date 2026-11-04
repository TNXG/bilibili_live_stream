use super::{
    ACCENT, MUTED, WARNING, protocol_details,
    widgets::{draw_selector, panel},
};
use crate::{app::App, model::PushProtocol};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Paragraph, Wrap},
};

pub(super) fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let protocols = app
        .session
        .as_ref()
        .map(|session| session.protocols.as_slice())
        .unwrap_or(&[]);
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);
    frame.render_widget(
        Paragraph::new(availability(protocols)).style(Style::default().fg(MUTED)),
        rows[0],
    );
    if protocols.is_empty() {
        frame.render_widget(
            Paragraph::new("直播间已开播，暂未获取到推流信息。\n按 r 重新获取，按 s 下播。")
                .block(panel(" 暂无推流信息 ", true))
                .wrap(Wrap { trim: true }),
            rows[1],
        );
        return;
    }
    let selected_index = app.protocol_index.min(protocols.len() - 1);
    let selected = &protocols[selected_index];
    if rows[1].height < 10 {
        draw_compact(
            frame,
            app,
            rows[1],
            selected_index,
            selected,
            protocols.len(),
        );
        return;
    }
    let wide = area.width >= 76;
    let panes = if wide {
        Layout::horizontal([Constraint::Length(29), Constraint::Min(0)]).split(rows[1])
    } else {
        Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).split(rows[1])
    };
    let labels = protocols
        .iter()
        .enumerate()
        .map(|(i, protocol)| protocol_row(i, protocol));
    let list_area = if wide && panes[0].height >= 12 {
        let stacked = Layout::vertical([Constraint::Min(6), Constraint::Length(5)]).split(panes[0]);
        draw_account(frame, app, stacked[1]);
        stacked[0]
    } else {
        panes[0]
    };
    draw_selector(
        frame,
        list_area,
        format!(" 推流线路 {}/{} ", selected_index + 1, protocols.len()),
        labels,
        protocols.len(),
        selected_index,
        true,
    );
    protocol_details::draw(frame, app, selected, panes[1]);
}

fn availability(protocols: &[PushProtocol]) -> String {
    ["RTMP", "SRT", "WHIP"]
        .iter()
        .map(|name| {
            let returned = protocols.iter().any(|p| {
                p.protocol.eq_ignore_ascii_case(name)
                    || (*name == "SRT" && p.protocol.eq_ignore_ascii_case("rtmpsrt"))
            });
            let state = if returned {
                "可用"
            } else if *name == "SRT" {
                "暂无（可按 r 刷新）"
            } else {
                "暂无"
            };
            format!("{name}:{state}")
        })
        .collect::<Vec<_>>()
        .join("  ")
}

/// Account card: who is streaming, where, and the chosen category.
fn draw_account(frame: &mut Frame, app: &App, area: Rect) {
    let account = app.account.as_ref();
    let name = account.map_or("未获取".into(), |a| a.name.clone());
    let mid = account.map_or_else(|| "—".into(), |a| a.mid.to_string());
    let area_name = app
        .selected_area
        .as_ref()
        .map_or("—".into(), |a| a.name.clone());
    let room = app.room_id.map_or_else(|| "—".into(), |id| id.to_string());
    let lines = vec![
        protocol_details::field("昵称", &name),
        protocol_details::field("UID", &mid),
        protocol_details::field("房间", &room),
        protocol_details::field("分区", &area_name),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(panel(" 账号信息 ", false)),
        area,
    );
}

pub(super) fn role(protocol: &PushProtocol) -> &'static str {
    if protocol.backup {
        "备用线路"
    } else {
        "主线路"
    }
}

fn protocol_row(index: usize, protocol: &PushProtocol) -> String {
    let name = if protocol.protocol.eq_ignore_ascii_case("rtmpsrt") {
        "SRT"
    } else {
        metadata(&protocol.protocol)
    };
    format!("{}. {} · {}", index + 1, name, role(protocol))
}

pub(super) fn metadata(value: &str) -> &str {
    if value
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
    {
        value
    } else {
        "（含特殊字符）"
    }
}

fn draw_compact(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    index: usize,
    protocol: &PushProtocol,
    count: usize,
) {
    // endpoint() allocates a sensitive derived URL; never compute it while masked.
    let endpoint = app.show_secrets.then(|| protocol.endpoint()).flatten();
    let mut lines = vec![
        Line::styled(
            format!(
                "[{}/{}] {} · {}",
                index + 1,
                count,
                protocol.label(),
                role(protocol)
            ),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        protocol_details::field(
            "完整链接",
            protocol_details::endpoint_text(endpoint.as_deref(), app.show_secrets),
        ),
    ];
    let warning = protocol.warning();
    if !warning.is_empty() {
        lines.push(Line::styled(warning, Style::default().fg(WARNING)));
    }
    lines.push(Line::styled("↑↓ 切换线路", Style::default().fg(MUTED)));
    frame.render_widget(Paragraph::new(lines), area);
}
