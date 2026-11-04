use super::{ACCENT, widgets::panel};
use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

pub(super) fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let lines = if let Some(stats) = app.stats.as_ref() {
        vec![
            Line::styled(
                "房间已关播",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Line::from(format!(
                "开播时长 {}   观看人数 {}",
                duration(stats.seconds),
                stats.viewers
            )),
            Line::from(format!(
                "最高在线 {} 人   弹幕 {} 条",
                stats.max_online, stats.danmaku
            )),
            Line::from(format!(
                "新增粉丝 {} 人   粉丝团 {} 人",
                stats.fans, stats.fans_club
            )),
            Line::from(format!("本场收益 {:.2} 元", stats.revenue)),
        ]
    } else {
        vec![
            Line::styled("房间已关播", Style::default().fg(ACCENT)),
            Line::from("暂无本场统计数据。"),
        ]
    };
    frame.render_widget(
        Paragraph::new(lines).block(panel(" 关播统计 ", false)),
        area,
    );
}

fn duration(seconds: i64) -> String {
    let seconds = seconds.max(0);
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}
