use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{App, Effect, Screen};
use crate::runtime::worker::{Command, Operation};

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Effect> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        if self.confirm_quit {
            return match key.code {
                KeyCode::Char('y' | 'Y') | KeyCode::Enter => {
                    self.confirm_quit = false;
                    self.exit_after_stop = true;
                    if self.busy {
                        self.status = "等待当前请求完成后关播退出…".into();
                        None
                    } else if self.session.is_some() {
                        Some(self.stop())
                    } else {
                        Some(Effect::Exit)
                    }
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    self.confirm_quit = false;
                    None
                }
                _ => None,
            };
        }
        if key.code == KeyCode::Char('q')
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            if self.session.is_some() || self.pending == Some(Operation::Start) {
                self.confirm_quit = true;
                return None;
            }
            return Some(Effect::Exit);
        }
        match key.code {
            KeyCode::PageUp => {
                self.log_scroll = self.log_scroll.saturating_add(3);
                return None;
            }
            KeyCode::PageDown => {
                self.log_scroll = self.log_scroll.saturating_sub(3);
                return None;
            }
            KeyCode::Char('h') => {
                self.show_secrets = !self.show_secrets;
                return None;
            }
            _ => {}
        }
        match self.screen {
            Screen::Areas if !self.busy => self.area_key(key.code),
            Screen::Live => self.live_key(key.code),
            Screen::Loading | Screen::Login if key.code == KeyCode::Char('r') && !self.busy => {
                Some(self.request(Operation::Bootstrap, "重新检查登录…", Command::Bootstrap))
            }
            Screen::Summary if matches!(key.code, KeyCode::Enter | KeyCode::Esc) => {
                self.screen = Screen::Areas;
                self.status = "请选择分区，按 Enter 开始直播".into();
                None
            }
            _ => None,
        }
    }

    fn area_key(&mut self, code: KeyCode) -> Option<Effect> {
        match code {
            KeyCode::Tab | KeyCode::BackTab => self.area_focus = !self.area_focus,
            KeyCode::Left | KeyCode::Esc => self.area_focus = false,
            KeyCode::Right => self.area_focus = true,
            KeyCode::Up | KeyCode::Down => {
                let down = code == KeyCode::Down;
                if self.area_focus {
                    let count = self
                        .groups
                        .get(self.group_index)
                        .map_or(0, |g| g.list.len());
                    self.area_index = step(self.area_index, count, down);
                } else {
                    self.group_index = step(self.group_index, self.groups.len(), down);
                    self.area_index = 0;
                }
            }
            KeyCode::Enter => {
                if !self.area_focus {
                    self.area_focus = true;
                } else if let Some(area) = self
                    .groups
                    .get(self.group_index)
                    .and_then(|g| g.list.get(self.area_index))
                    .cloned()
                {
                    return Some(self.start(area));
                }
            }
            KeyCode::Char('l') => {
                if let Some(area) = self.recent_area.clone() {
                    return Some(self.start(area));
                }
                self.status = "暂无上次分区记录，请从列表中选择".into();
            }
            KeyCode::Char('r') => {
                return Some(self.request(
                    Operation::Bootstrap,
                    "重新加载分区…",
                    Command::Bootstrap,
                ));
            }
            _ => {}
        }
        None
    }

    fn live_key(&mut self, code: KeyCode) -> Option<Effect> {
        match code {
            KeyCode::Up | KeyCode::Down => {
                let count = self.session.as_ref().map_or(0, |s| s.protocols.len());
                self.protocol_index = step(self.protocol_index, count, code == KeyCode::Down);
            }
            KeyCode::Char('r') if !self.busy => {
                if let Some(area) = &self.selected_area {
                    let area_id = area.id;
                    return Some(self.request(
                        Operation::Refresh,
                        "正在刷新推流信息…",
                        Command::Refresh { area_id },
                    ));
                }
            }
            KeyCode::Char('s') if !self.busy => return Some(self.stop()),
            KeyCode::Char('c' | 'k' | 'a') => {
                let protocol = self
                    .session
                    .as_ref()
                    .and_then(|s| s.protocols.get(self.protocol_index));
                let protocol = protocol?;
                let Some(endpoint) = protocol.endpoint() else {
                    self.status = "当前线路不支持复制".into();
                    return None;
                };
                let value = match code {
                    KeyCode::Char('k') => protocol.code.clone(),
                    KeyCode::Char('a') => protocol.addr.clone(),
                    _ => endpoint,
                };
                if value.is_empty() {
                    self.status = "该项为空，无法复制".into();
                } else {
                    return Some(Effect::Copy(value));
                }
            }
            _ => {}
        }
        None
    }
}

fn step(index: usize, count: usize, down: bool) -> usize {
    if count == 0 {
        return 0;
    }
    if down {
        index.saturating_add(1).min(count - 1)
    } else {
        index.saturating_sub(1).min(count - 1)
    }
}
