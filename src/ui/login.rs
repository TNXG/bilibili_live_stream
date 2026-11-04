use super::widgets::panel;
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    widgets::{Paragraph, Wrap},
};

pub(super) fn draw_loading(frame: &mut Frame, app: &App, area: Rect) {
    let message = if app.busy {
        "正在连接 B站，请稍候…"
    } else {
        "连接未完成，按 r 重试，按 q 退出。"
    };
    frame.render_widget(
        Paragraph::new(message)
            .block(panel(" 连接 ", false))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub(super) fn draw_login(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(qr) = app.qr.as_mut() else {
        frame.render_widget(
            Paragraph::new("正在获取登录二维码…\n若长时间未显示，可按 r 刷新。")
                .wrap(Wrap { trim: true }),
            area,
        );
        return;
    };
    let minimum = qr.minimum_size();
    // Border only when it does not cost the QR its minimum rows.
    let bordered = area.width >= minimum.width.saturating_add(2)
        && area.height >= minimum.height.saturating_add(2);
    let inner = if bordered {
        let block = panel(" 扫码登录 · 使用哔哩哔哩 App ", true);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        inner
    } else {
        area
    };
    // QrDisplay caches the encoded protocol; re-render is cheap and never crops.
    qr.render(frame, inner);
    if inner.height > 2 {
        frame.render_widget(
            Paragraph::new("扫描后请在手机端确认 · r 刷新").alignment(Alignment::Center),
            Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1),
        );
    }
}
