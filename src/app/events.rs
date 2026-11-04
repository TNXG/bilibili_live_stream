use crate::qr::QrDisplay;

use super::{App, Effect, Screen};
use crate::runtime::worker::{Event, Operation};

impl App {
    pub fn handle_event(&mut self, event: Event) -> Option<Effect> {
        match event {
            Event::Qr(url) => {
                self.busy = false;
                self.pending = None;
                self.screen = Screen::Login;
                self.status = "请使用哔哩哔哩 App 扫码登录 · q 退出".into();
                self.logs
                    .push(log::Level::Info, "已生成登录二维码，等待扫码".into());
                if self.qr.as_ref().is_none_or(|qr| !qr.matches(&url)) {
                    match QrDisplay::new(&url, self.qr_picker.clone()) {
                        Ok(qr) => self.qr = Some(qr),
                        Err(_) => {
                            self.qr = None;
                            self.status = "二维码加载失败，按 r 重新获取".into();
                        }
                    }
                }
            }
            Event::LoginStatus(status) => {
                self.logs.push(log::Level::Info, status.clone());
                self.status = status;
            }
            Event::Areas {
                room_id,
                account,
                groups,
                recent,
            } => {
                self.room_id = Some(room_id);
                if account.is_some() {
                    self.account = account;
                }
                self.groups = groups;
                self.recent_area = recent;
                self.group_index = 0;
                self.area_index = 0;
                self.area_focus = false;
                self.busy = false;
                self.pending = None;
                self.qr = None;
                self.screen = Screen::Areas;
                self.status = "请选择分区，按 Enter 开始直播".into();
                self.logs.push(log::Level::Info, "已加载直播分区".into());
            }
            Event::Started(session) => {
                self.session = Some(session);
                self.protocol_index = 0;
                self.busy = false;
                self.pending = None;
                self.screen = Screen::Live;
                self.status =
                    "直播间已开播 · 按 c 复制完整链接，或按 a/k 分别复制地址与推流码".into();
                self.logs
                    .push(log::Level::Info, "直播间已开播，已获取推流信息".into());
                if self.exit_after_stop {
                    return Some(self.stop());
                }
            }
            Event::Refreshed(protocols) => {
                if let Some(session) = &mut self.session {
                    // A refresh may omit lines the first response had (and the
                    // reverse): keep every unique server-issued record.
                    for protocol in protocols {
                        let duplicate = session.protocols.iter().any(|known| {
                            known.protocol.eq_ignore_ascii_case(&protocol.protocol)
                                && known.addr == protocol.addr
                                && known.code == protocol.code
                        });
                        if !duplicate {
                            session.protocols.push(protocol);
                        }
                    }
                    self.protocol_index = self
                        .protocol_index
                        .min(session.protocols.len().saturating_sub(1));
                }
                self.busy = false;
                self.pending = None;
                self.status = "推流信息已刷新".into();
                if self.exit_after_stop {
                    return Some(self.stop());
                }
            }
            Event::Stopped => {
                self.session = None;
                self.busy = false;
                self.pending = None;
                self.confirm_quit = false;
                self.screen = Screen::Summary;
                self.status = "直播间已关闭 · 正在获取本场统计…".into();
                self.logs.push(log::Level::Info, "直播间已关闭".into());
                if self.exit_after_stop {
                    return Some(Effect::Exit);
                }
            }
            Event::Stats(stats) => {
                // Statistics belong to the closed room, not a newer start request.
                if self.screen == Screen::Summary && self.session.is_none() {
                    self.stats = Some(stats);
                    self.status = "直播间已关闭".into();
                }
            }
            Event::Failed { operation, error } => {
                if operation == Operation::Stats {
                    self.logs.push(log::Level::Warn, error.clone());
                    if self.screen == Screen::Summary {
                        self.status = format!("直播间已关闭；统计获取失败：{error}");
                    }
                    return None;
                }
                self.busy = false;
                self.pending = None;
                self.logs.push(log::Level::Error, error.clone());
                self.status = error;
                match operation {
                    Operation::Bootstrap | Operation::Login => {
                        self.status.push_str(" · 按 r 重试，按 q 退出");
                    }
                    Operation::Areas => {
                        self.screen = Screen::Loading;
                        self.status.push_str(" · 按 r 重新加载");
                    }
                    Operation::Start => {
                        self.exit_after_stop = false;
                        self.confirm_quit = false;
                        self.status.push_str(" · 请检查直播间状态或重试");
                    }
                    Operation::Stop => {
                        // Keep session and its codes: a failed stop is not permission to exit.
                        self.exit_after_stop = false;
                        self.confirm_quit = false;
                        self.status.push_str(" · 关播失败，请按 s 重试");
                    }
                    Operation::Refresh if self.exit_after_stop => return Some(self.stop()),
                    Operation::Stats => unreachable!("statistics handled independently"),
                    Operation::Refresh => {}
                }
            }
        }
        None
    }
}
