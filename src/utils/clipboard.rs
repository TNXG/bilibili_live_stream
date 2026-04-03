use crate::error::{BiliLiveError, Result};
use arboard::Clipboard;
use std::cell::RefCell;

thread_local! {
    pub static CLIPBOARD: RefCell<Option<Clipboard>> = const { RefCell::new(None) };
}

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    CLIPBOARD.with(|cell| {
        let mut clipboard_opt = cell.borrow_mut();
        if clipboard_opt.is_none()
            && let Ok(ctx) = Clipboard::new()
        {
            *clipboard_opt = Some(ctx);
        }

        if let Some(ctx) = clipboard_opt.as_mut() {
            ctx.set_text(text.to_owned()).map_err(|e| {
                BiliLiveError::Io(std::io::Error::other(format!("复制到剪贴板失败: {}", e)))
            })
        } else {
            Err(BiliLiveError::Io(std::io::Error::other(
                "无法初始化剪贴板".to_string(),
            )))
        }
    })
}
