mod areas;
mod chrome;
mod login;
mod protocol_details;
mod protocols;
mod summary;
mod widgets;

use crate::app::{App, Screen};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Color,
};

const ACCENT: Color = Color::Cyan;
const MUTED: Color = Color::Gray;
const WARNING: Color = Color::Yellow;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    app.buttons.clear();
    if size.is_empty() {
        return;
    }
    let compact = size.height < 20;
    let normal_logs = if size.height >= 30 {
        6
    } else if compact {
        3
    } else {
        4
    };
    let login = matches!(app.screen, Screen::Login);
    let log_height = if login {
        // Header 2 + footer 1 + panel border 2 must not starve the QR.
        let qr_height = app.qr.as_ref().map_or(20, |qr| qr.minimum_size().height);
        normal_logs.min(size.height.saturating_sub(qr_height.saturating_add(5)))
    } else {
        normal_logs
    };
    let panes = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(log_height),
        Constraint::Length(if compact || login { 1 } else { 2 }),
    ])
    .split(size);
    chrome::draw_header(frame, app, panes[0]);
    match app.screen {
        Screen::Loading => login::draw_loading(frame, app, panes[1]),
        Screen::Login => login::draw_login(frame, app, panes[1]),
        Screen::Areas => areas::draw(frame, app, panes[1]),
        Screen::Live => protocols::draw(frame, app, panes[1]),
        Screen::Summary => summary::draw(frame, app, panes[1]),
    }
    app.log_area = panes[2];
    chrome::draw_logs(frame, app, panes[2]);
    chrome::draw_footer(frame, app, panes[3]);
    if app.confirm_quit {
        chrome::draw_confirmation(frame, app, size);
    }
}
