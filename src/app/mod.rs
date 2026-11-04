mod events;
mod input;

use ratatui_image::picker::Picker;

use crate::logging::LogBuffer;
use crate::model::{Account, Area, AreaGroup, LiveSession, LiveStats};
use crate::qr::QrDisplay;
use crate::runtime::worker::{Command, Operation};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Loading,
    Login,
    Areas,
    Live,
    Summary,
}

pub enum Effect {
    Request(Command),
    Copy(String),
    Exit,
}

/// A footer button: the key it mirrors, its label, and its drawn cell area.
#[derive(Clone, Debug)]
pub struct Button {
    pub area: Rect,
    pub key: KeyCode,
    pub label: &'static str,
}

impl Button {
    #[must_use]
    pub fn new(area: Rect, key: KeyCode, label: &'static str) -> Self {
        Self { area, key, label }
    }
}

/// UI state contains no account cookies. Network operations run in the worker.
pub struct App {
    pub screen: Screen,
    pub status: String,
    pub busy: bool,
    pub qr: Option<QrDisplay>,
    pub room_id: Option<u64>,
    pub account: Option<Account>,
    pub groups: Vec<AreaGroup>,
    pub group_index: usize,
    pub area_index: usize,
    pub area_focus: bool,
    pub selected_area: Option<Area>,
    pub recent_area: Option<Area>,
    pub session: Option<LiveSession>,
    pub protocol_index: usize,
    pub show_secrets: bool,
    pub stats: Option<LiveStats>,
    pub confirm_quit: bool,
    pub exit_after_stop: bool,
    pub logs: LogBuffer,
    pub log_scroll: u16,
    /// Buttons drawn in the last frame; cleared on every draw.
    pub buttons: Vec<Button>,
    /// Log pane drawn in the last frame, for wheel scrolling.
    pub log_area: Rect,
    /// Ask the server for an SRT line at start (it may still be omitted).
    pub request_srt: bool,
    pending: Option<Operation>,
    qr_picker: Picker,
}

impl App {
    #[must_use]
    pub fn new(logs: LogBuffer, show_secrets: bool) -> Self {
        Self {
            screen: Screen::Loading,
            status: "正在检查登录状态…".into(),
            busy: true,
            qr: None,
            room_id: None,
            account: None,
            groups: Vec::new(),
            group_index: 0,
            area_index: 0,
            area_focus: false,
            selected_area: None,
            recent_area: None,
            session: None,
            protocol_index: 0,
            show_secrets,
            stats: None,
            confirm_quit: false,
            exit_after_stop: false,
            logs,
            log_scroll: 0,
            buttons: Vec::new(),
            log_area: Rect::ZERO,
            request_srt: false,
            pending: Some(Operation::Bootstrap),
            qr_picker: Picker::halfblocks(),
        }
    }

    pub fn configure_qr(&mut self, picker: Picker) {
        self.qr_picker = picker;
        self.qr = None;
    }

    /// Translate a mouse click into the same action as its keyboard shortcut.
    pub fn handle_click(&mut self, column: u16, row: u16) -> Option<Effect> {
        let key = self
            .buttons
            .iter()
            .find(|button| {
                column >= button.area.x
                    && column < button.area.right()
                    && row >= button.area.y
                    && row < button.area.bottom()
            })
            .map(|button| button.key);
        let Some(key) = key else {
            log::debug!("点击未命中按钮: {column},{row}");
            return None;
        };
        self.handle_key(ratatui::crossterm::event::KeyEvent::new(
            key,
            ratatui::crossterm::event::KeyModifiers::NONE,
        ))
    }

    /// Mouse wheel: scroll the log pane under the pointer, otherwise move the
    /// focused list (same as the arrow keys). Display-only screens scroll logs.
    pub fn handle_scroll(&mut self, column: u16, row: u16, up: bool) -> Option<Effect> {
        let over_logs = column >= self.log_area.x
            && column < self.log_area.right()
            && row >= self.log_area.y
            && row < self.log_area.bottom();
        if over_logs {
            let step = 3;
            self.log_scroll = if up {
                self.log_scroll.saturating_add(step)
            } else {
                self.log_scroll.saturating_sub(step)
            };
            return None;
        }
        let key = match self.screen {
            Screen::Areas | Screen::Live => {
                if up {
                    KeyCode::Up
                } else {
                    KeyCode::Down
                }
            }
            _ => {
                let step = 3;
                self.log_scroll = if up {
                    self.log_scroll.saturating_add(step)
                } else {
                    self.log_scroll.saturating_sub(step)
                };
                return None;
            }
        };
        self.handle_key(ratatui::crossterm::event::KeyEvent::new(
            key,
            ratatui::crossterm::event::KeyModifiers::NONE,
        ))
    }

    fn request(&mut self, operation: Operation, status: &str, command: Command) -> Effect {
        self.busy = true;
        self.pending = Some(operation);
        self.status = status.into();
        self.logs.push(log::Level::Info, status.into());
        Effect::Request(command)
    }

    fn stop(&mut self) -> Effect {
        self.request(Operation::Stop, "正在关闭直播间…", Command::Stop)
    }

    fn start(&mut self, area: Area) -> Effect {
        let id = area.id;
        self.selected_area = Some(area);
        self.stats = None;
        self.request(
            Operation::Start,
            "正在开启直播间…",
            Command::Start {
                area_id: id,
                request_srt: self.request_srt,
            },
        )
    }
}
