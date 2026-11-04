use super::{
    WARNING,
    widgets::{draw_selector, panel},
};
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::Paragraph,
};

pub(super) fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);
    let recent = app.recent_area.as_ref().map_or_else(
        || "l 上次分区：暂无记录".into(),
        |last| format!("l 上次分区：{}", last.name),
    );
    frame.render_widget(
        Paragraph::new(recent).style(Style::default().fg(WARNING)),
        rows[0],
    );
    let columns =
        Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)]).split(rows[1]);
    let groups = app.groups.iter().map(|group| group.name.as_str());
    draw_selector(
        frame,
        columns[0],
        " 大分区 ".into(),
        groups,
        app.groups.len(),
        app.group_index,
        !app.area_focus,
    );
    let selected_group = app.groups.get(app.group_index);
    let count = selected_group.map_or(0, |group| group.list.len());
    let areas = selected_group
        .into_iter()
        .flat_map(|group| group.list.iter())
        .map(|area| area.name.as_str());
    draw_selector(
        frame,
        columns[1],
        format!(" 子分区 {}/{} ", position(app.area_index, count), count),
        areas,
        count,
        app.area_index,
        app.area_focus,
    );
    if app.groups.is_empty() {
        frame.render_widget(
            Paragraph::new("暂无分区 · r 重试").style(Style::default().fg(WARNING)),
            panel("", false).inner(columns[1]),
        );
    }
}

fn position(selected: usize, count: usize) -> usize {
    if count == 0 {
        0
    } else {
        selected.min(count - 1) + 1
    }
}
