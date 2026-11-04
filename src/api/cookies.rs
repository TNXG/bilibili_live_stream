use crate::error::{BiliLiveError, Result};
use crate::model::Credentials;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn cookie_value(raw: &str, name: &str) -> Option<String> {
    raw.split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(key, value)| *key == name && !value.is_empty())
        .map(|(_, value)| value.to_owned())
}

pub(super) fn validate_credentials(c: &Credentials) -> Result<()> {
    let cookie_safe = |value: &str| {
        !value.is_empty()
            && value
                .bytes()
                .all(|b| b.is_ascii_graphic() && !matches!(b, b';' | b',' | b'"' | b'\\'))
    };
    if c.room_id == 0 || !cookie_safe(&c.sessdata) || !cookie_safe(&c.csrf_token) {
        return Err(BiliLiveError::Parse(
            "登录凭据缺少有效房间号、SESSDATA 或 CSRF".into(),
        ));
    }
    Ok(())
}

pub(super) fn persist_credentials(c: &Credentials, path: &Path) -> Result<()> {
    validate_credentials(c)?;
    let bytes = serde_json::to_vec_pretty(c)
        .map_err(|_| BiliLiveError::Parse("无法编码登录凭据".into()))?;
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let temp = path.with_file_name(format!(
        ".cookies.json.{}.{}.tmp",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp)?;
    let result = (|| -> std::io::Result<()> {
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(BiliLiveError::Io)
}
