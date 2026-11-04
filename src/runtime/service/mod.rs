mod login;

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use super::worker::{Command, Event, Operation};
use crate::api::Api;
use crate::model::{Account, Credentials, QrCodeData};

pub(super) struct Service {
    sender: Sender<Event>,
    api: Option<Api>,
    credentials: Option<Credentials>,
    account: Option<Account>,
    qr: Option<QrCodeData>,
    next_poll: Instant,
    live_key: Option<String>,
}

impl Service {
    pub(super) fn new(sender: Sender<Event>) -> Self {
        Self {
            sender,
            api: None,
            credentials: None,
            account: None,
            qr: None,
            next_poll: Instant::now(),
            live_key: None,
        }
    }

    fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }
    fn fail(&self, operation: Operation, error: impl std::fmt::Display) {
        self.send(Event::Failed {
            operation,
            error: error.to_string(),
        });
    }

    pub(super) fn run(mut self, receiver: Receiver<Command>) -> Result<(), String> {
        loop {
            let wait = if self.qr.is_some() {
                self.next_poll.saturating_duration_since(Instant::now())
            } else {
                Duration::from_secs(3600)
            };
            match receiver.recv_timeout(wait) {
                Ok(Command::Shutdown) | Err(RecvTimeoutError::Disconnected) => {
                    // Also clean up an active room if terminal I/O fails.
                    if self.live_key.is_some() {
                        self.api
                            .as_ref()
                            .unwrap()
                            .stop(self.credentials.as_ref().unwrap())
                            .map_err(|e| format!("退出时关播失败：{e}；请在 B 站手动关播"))?;
                    }
                    return Ok(());
                }
                Ok(Command::Bootstrap) => self.bootstrap(),
                Ok(Command::Start {
                    area_id,
                    request_srt,
                }) => self.start(area_id, request_srt),
                Ok(Command::Refresh { area_id }) => self.refresh(area_id),
                Ok(Command::Stop) => self.stop(),
                Err(RecvTimeoutError::Timeout) => self.poll_login(),
            }
        }
    }

    fn start(&mut self, area_id: u64, request_srt: bool) {
        if self.live_key.is_some() {
            self.fail(Operation::Start, "直播间已开播，请先关播");
            return;
        }
        let (Some(api), Some(credentials)) = (&self.api, &self.credentials) else {
            self.fail(Operation::Start, "尚未登录");
            return;
        };
        match api.start(credentials, area_id, request_srt) {
            Ok(session) => {
                self.live_key = Some(session.live_key.clone());
                self.send(Event::Started(session));
            }
            Err(error) => self.fail(Operation::Start, error),
        }
    }

    fn refresh(&self, area_id: u64) {
        let (Some(api), Some(credentials)) = (&self.api, &self.credentials) else {
            return;
        };
        match api.refresh(credentials, area_id) {
            Ok(protocols) => self.send(Event::Refreshed(protocols)),
            Err(error) => self.fail(Operation::Refresh, error),
        }
    }

    fn stop(&mut self) {
        let (Some(api), Some(credentials)) = (&self.api, &self.credentials) else {
            return;
        };
        match api.stop(credentials) {
            Ok(()) => {
                let live_key = self.live_key.take();
                self.send(Event::Stopped);
                // Failed statistics must never be interpreted as failed stop.
                if let Some(key) = live_key.filter(|key| !key.is_empty()) {
                    match api.stats(credentials, &key) {
                        Ok(stats) => self.send(Event::Stats(stats)),
                        Err(error) => self.fail(Operation::Stats, error),
                    }
                } else {
                    self.fail(Operation::Stats, "暂无本场统计数据");
                }
            }
            Err(error) => self.fail(Operation::Stop, error),
        }
    }
}
