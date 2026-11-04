use bili_live::{
    app::{App, Screen},
    logging::LogBuffer,
    model::{LiveSession, PushProtocol},
    ui,
};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

pub const SIZES: [(u16, u16); 3] = [(100, 32), (80, 24), (40, 12)];

pub fn render(app: &mut App, width: u16, height: u16) -> String {
    text(&render_buffer(app, width, height))
}

/// Cell symbols joined without padding spaces, so wide CJK glyphs match as
/// contiguous text (their continuation cells are blank).
pub fn text(buffer: &Buffer) -> String {
    buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>()
        .replace(' ', "")
}

pub fn render_buffer(app: &mut App, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| ui::draw(frame, app)).unwrap();
    terminal.backend().buffer().clone()
}

pub fn live_app() -> App {
    let mut app = App::new(LogBuffer::new(), false);
    app.screen = Screen::Live;
    app.session = Some(LiveSession {
        live_key: "PRIVATE_LIVE_KEY".into(),
        protocols: vec![PushProtocol {
            protocol: "WHIP".into(),
            addr: "https://PRIVATE_HOST/PRIVATE_PATH?token=PRIVATE_QUERY".into(),
            code: "PRIVATE_CODE".into(),
            provider: "PRIVATE_PROVIDER".into(),
            backup: true,
        }],
    });
    app
}
