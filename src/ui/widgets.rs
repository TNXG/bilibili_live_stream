use super::ACCENT;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub(super) fn panel(title: impl Into<Line<'static>>, focused: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(if focused { ACCENT } else { Color::DarkGray }))
}

pub(super) fn draw_selector<'a, T: Into<Text<'a>>>(
    frame: &mut Frame,
    area: Rect,
    title: String,
    items: impl Iterator<Item = T>,
    count: usize,
    selected: usize,
    focused: bool,
) {
    let capacity = usize::from(area.height.saturating_sub(2));
    let selected = selected.min(count.saturating_sub(1));
    let offset = (selected + 1).saturating_sub(capacity.max(1));
    let items: Vec<_> = items
        .skip(offset)
        .take(capacity)
        .map(ListItem::new)
        .collect();
    let mut state = ListState::default()
        .with_selected((count > 0 && capacity > 0).then_some(selected.saturating_sub(offset)));
    frame.render_stateful_widget(
        List::new(items)
            .block(panel(title, focused))
            .highlight_symbol("› ")
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
        area,
        &mut state,
    );
}
