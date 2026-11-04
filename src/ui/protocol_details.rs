use super::{ACCENT, MUTED, WARNING, widgets::panel};
use crate::{app::App, model::PushProtocol};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

/// Uniform mask: one glyph per field, so the eye scans labels not hint text.
const MASK: &str = "▮▮▮▮▮▮";
const LABEL_COLUMNS: usize = 12;

pub(super) fn draw(frame: &mut Frame, app: &App, protocol: &PushProtocol, area: Rect) {
    let block = panel(" 推流信息详情 ", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    // Do not materialize a secret endpoint on every hidden frame.
    let endpoint = app.show_secrets.then(|| protocol.endpoint()).flatten();
    let title = Line::from(vec![
        Span::styled(
            format!("{} · {}", name(protocol), super::protocols::role(protocol)),
            Style::default()
                .fg(if app.show_secrets { WARNING } else { ACCENT })
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(
            if app.show_secrets {
                "  已显示"
            } else {
                "  已隐藏"
            },
            Style::default().fg(if app.show_secrets { WARNING } else { MUTED }),
        ),
    ]);
    let rows = Layout::vertical([Constraint::Length(6), Constraint::Min(0)]).split(inner);
    let lines = vec![
        title,
        field(
            "完整链接",
            endpoint_text(endpoint.as_deref(), app.show_secrets),
        ),
        field("推流地址", secret(&protocol.addr, app.show_secrets)),
        field("推流码", secret(&protocol.code, app.show_secrets)),
        field("线路名称", secret(&protocol.provider, app.show_secrets)),
    ];
    frame.render_widget(Paragraph::new(lines), rows[0]);
    let warning = protocol.warning();
    if !warning.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::styled(warning, Style::default().fg(WARNING)))
                .wrap(Wrap { trim: true }),
            rows[1],
        );
    }
}

fn name(protocol: &PushProtocol) -> String {
    if protocol.protocol.eq_ignore_ascii_case("rtmp")
        || protocol.protocol.eq_ignore_ascii_case("srt")
        || protocol.protocol.eq_ignore_ascii_case("rtmpsrt")
        || protocol.protocol.eq_ignore_ascii_case("whip")
    {
        protocol.label().to_owned()
    } else {
        super::protocols::metadata(&protocol.protocol).to_owned()
    }
}

fn secret(value: &str, show: bool) -> &str {
    if value.is_empty() {
        "暂无"
    } else if show {
        value
    } else {
        MASK
    }
}

pub(super) fn endpoint_text(endpoint: Option<&str>, show: bool) -> &str {
    if show {
        endpoint.unwrap_or("暂无")
    } else {
        MASK
    }
}

/// Label padded to a fixed display-column width, so CJK and ASCII align.
pub(super) fn field(label: &'static str, value: &str) -> Line<'static> {
    let padding = LABEL_COLUMNS.saturating_sub(UnicodeWidthStr::width(label));
    Line::from(vec![
        Span::styled(
            format!("{label}{}", " ".repeat(padding)),
            Style::default().fg(MUTED),
        ),
        Span::styled(value.to_owned(), Style::default().fg(Color::Reset)),
    ])
}
