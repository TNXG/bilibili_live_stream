use ratatui_image::picker::Picker;

/// Query once before the event loop: two simultaneous stdin readers lose key events.
pub(super) fn detect() -> Picker {
    match Picker::from_query_stdio() {
        Ok(picker) => picker,
        Err(_) => {
            log::warn!("终端图片能力检测未完成，使用黑白块二维码");
            Picker::halfblocks()
        }
    }
}
