mod graphics;
mod service;
pub mod worker;

use std::io::{self, IsTerminal};
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use arboard::Clipboard;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as TerminalEvent, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;

use crate::app::{App, Effect};
use crate::logging::LogBuffer;
use crate::ui;
use worker::{Command, Worker};

struct TerminalRestore;
impl Drop for TerminalRestore {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableMouseCapture);
        ratatui::restore();
    }
}

pub fn run(logs: LogBuffer, show_secrets: bool, request_srt: bool) -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other("请在交互式终端中运行 bili_live"));
    }
    let restore = TerminalRestore;
    let mut terminal = ratatui::try_init()?;
    // Buttons work where the terminal reports mouse events; capture is harmless elsewhere.
    let _ = execute!(io::stdout(), EnableMouseCapture);
    let mut app = App::new(logs, show_secrets);
    app.request_srt = request_srt;
    app.status = "正在初始化界面…".into();
    terminal.draw(|frame| ui::draw(frame, &mut app))?;
    app.configure_qr(graphics::detect());
    app.status = "正在检查登录状态…".into();
    let worker = Worker::spawn();
    let mut clipboard: Option<Clipboard> = None;
    worker
        .commands
        .send(Command::Bootstrap)
        .map_err(|_| io::Error::other("网络线程不可用"))?;
    let result = event_loop(&mut terminal, &mut app, &worker, &mut clipboard);
    drop(restore);
    match result {
        Ok(()) => {
            // Normal exit is only allowed after successful stop (or before opening a room).
            // Do not wait for harmless pending login/statistics HTTP requests.
            worker.detach();
            Ok(())
        }
        Err(error) => {
            if let Err(cleanup) = worker.shutdown() {
                return Err(io::Error::other(format!("{error}；{cleanup}")));
            }
            Err(error)
        }
    }
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    worker: &Worker,
    clipboard: &mut Option<Clipboard>,
) -> io::Result<()> {
    loop {
        loop {
            match worker.events.try_recv() {
                Ok(event) => {
                    let effect = app.handle_event(event);
                    if apply(effect, app, worker, clipboard)? {
                        return Ok(());
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(io::Error::other("网络线程已终止，请检查直播间状态"));
                }
            }
        }
        terminal.draw(|frame| ui::draw(frame, app))?;
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            match event {
                TerminalEvent::Key(key) => {
                    let effect = app.handle_key(key);
                    if apply(effect, app, worker, clipboard)? {
                        return Ok(());
                    }
                }
                TerminalEvent::Mouse(mouse)
                    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
                {
                    log::debug!("鼠标点击 {},{}", mouse.column, mouse.row);
                    let effect = app.handle_click(mouse.column, mouse.row);
                    if apply(effect, app, worker, clipboard)? {
                        return Ok(());
                    }
                }
                TerminalEvent::Mouse(mouse)
                    if matches!(
                        mouse.kind,
                        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                    ) =>
                {
                    let effect = app.handle_scroll(
                        mouse.column,
                        mouse.row,
                        matches!(mouse.kind, MouseEventKind::ScrollUp),
                    );
                    if apply(effect, app, worker, clipboard)? {
                        return Ok(());
                    }
                }
                TerminalEvent::Resize(_, _) => {}
                other => log::debug!("终端事件 {other:?}"),
            }
        }
    }
}

fn apply(
    effect: Option<Effect>,
    app: &mut App,
    worker: &Worker,
    clipboard: &mut Option<Clipboard>,
) -> io::Result<bool> {
    match effect {
        Some(Effect::Exit) => return Ok(true),
        Some(Effect::Request(command)) => worker
            .commands
            .send(command)
            .map_err(|_| io::Error::other("网络线程已终止，请检查直播间状态"))?,
        Some(Effect::Copy(text)) => {
            if clipboard.is_none() {
                if let Ok(owner) = Clipboard::new() {
                    *clipboard = Some(owner)
                } else {
                    app.status = "剪贴板不可用 · 请按 h 显示后手动复制".into();
                    return Ok(false);
                }
            }
            // Retain the owner for X11 clipboard lifetime; never log copied contents.
            match clipboard.as_mut().unwrap().set_text(text) {
                Ok(()) => app.status = "已复制到剪贴板".into(),
                Err(_) => app.status = "复制失败 · 请按 h 显示后手动复制".into(),
            }
        }
        None => {}
    }
    Ok(false)
}
