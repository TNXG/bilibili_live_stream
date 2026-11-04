use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use super::service::Service;
use crate::model::{Account, Area, AreaGroup, LiveSession, LiveStats, PushProtocol};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Bootstrap,
    Login,
    Areas,
    Start,
    Refresh,
    Stop,
    Stats,
}

pub enum Command {
    Bootstrap,
    Start { area_id: u64, request_srt: bool },
    Refresh { area_id: u64 },
    Stop,
    Shutdown,
}

pub enum Event {
    Qr(String),
    LoginStatus(String),
    Areas {
        room_id: u64,
        account: Option<Account>,
        groups: Vec<AreaGroup>,
        recent: Option<Area>,
    },
    Started(LiveSession),
    Refreshed(Vec<PushProtocol>),
    Stopped,
    Stats(LiveStats),
    Failed {
        operation: Operation,
        error: String,
    },
}

pub struct Worker {
    pub commands: Sender<Command>,
    pub events: Receiver<Event>,
    join: JoinHandle<Result<(), String>>,
}

impl Worker {
    #[must_use]
    pub fn spawn() -> Self {
        let (commands, receiver) = mpsc::channel();
        let (sender, events) = mpsc::channel();
        let join = thread::spawn(move || Service::new(sender).run(receiver));
        Self {
            commands,
            events,
            join,
        }
    }

    pub fn detach(self) {
        let _ = self.commands.send(Command::Shutdown);
    }

    pub fn shutdown(self) -> Result<(), String> {
        let _ = self.commands.send(Command::Shutdown);
        self.join
            .join()
            .unwrap_or_else(|_| Err("网络工作线程异常终止，请检查直播间状态".into()))
    }
}
