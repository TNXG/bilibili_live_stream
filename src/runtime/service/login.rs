use std::time::{Duration, Instant};

use super::Service;
use crate::api::Api;
use crate::runtime::worker::{Event, Operation};

impl Service {
    pub(super) fn bootstrap(&mut self) {
        if self.live_key.is_some() {
            return;
        }
        self.qr = None;
        let api = match Api::new() {
            Ok(api) => api,
            Err(error) => {
                self.fail(Operation::Bootstrap, error);
                return;
            }
        };
        match api.check_session() {
            Ok(session) => {
                if let Some((credentials, account)) = session {
                    self.credentials = Some(credentials);
                    self.account = Some(account);
                }
            }
            Err(error) => {
                self.fail(Operation::Bootstrap, error);
                return;
            }
        }
        self.api = Some(api);
        if self.credentials.is_some() {
            self.send(Event::LoginStatus("已登录，正在加载分区…".into()));
            self.load_areas();
        } else {
            self.send(Event::LoginStatus("正在获取登录二维码…".into()));
            self.generate_qr();
        }
    }

    fn generate_qr(&mut self) {
        match self.api.as_ref().unwrap().generate_qr() {
            Ok(qr) => {
                self.send(Event::Qr(qr.url.clone()));
                self.qr = Some(qr);
                self.next_poll = Instant::now() + Duration::from_secs(2);
            }
            Err(error) => {
                self.qr = None;
                self.fail(Operation::Login, error);
            }
        }
    }

    pub(super) fn poll_login(&mut self) {
        let Some(qr) = self.qr.as_ref() else {
            return;
        };
        let api = self.api.as_ref().unwrap();
        match api.poll_qr(&qr.qrcode_key) {
            Ok(poll) => match poll.code {
                86101 => {}
                86090 => self.send(Event::LoginStatus("已扫码，请在手机上确认登录".into())),
                86038 => {
                    self.qr = None;
                    self.generate_qr();
                }
                0 => {
                    self.qr = None;
                    match api.finish_login(&poll.url) {
                        Ok((credentials, account)) => {
                            self.credentials = Some(credentials);
                            self.account = Some(account);
                            self.send(Event::LoginStatus("登录成功，正在加载分区…".into()));
                            self.load_areas();
                        }
                        Err(error) => self.fail(Operation::Login, error),
                    }
                }
                code => {
                    self.qr = None;
                    self.fail(Operation::Login, format!("扫码状态异常（{code}）"));
                }
            },
            Err(error) => {
                self.qr = None;
                self.fail(Operation::Login, error);
            }
        }
        self.next_poll = Instant::now() + Duration::from_secs(2);
    }

    fn load_areas(&self) {
        let api = self.api.as_ref().unwrap();
        let credentials = self.credentials.as_ref().unwrap();
        match api.areas() {
            Ok(groups) => {
                let recent = match api.recent_area(credentials) {
                    Ok(recent) => recent,
                    Err(error) => {
                        log::warn!("读取上次分区失败：{error}");
                        None
                    }
                };
                self.send(Event::Areas {
                    room_id: credentials.room_id,
                    account: self.account.clone(),
                    groups,
                    recent,
                });
            }
            Err(error) => self.fail(Operation::Areas, error),
        }
    }
}
