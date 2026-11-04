// Intentionally no Debug: every address may carry credentials.
#[derive(Clone)]
pub struct PushProtocol {
    pub protocol: String,
    pub addr: String,
    pub code: String,
    pub provider: String,
    pub backup: bool,
}

impl PushProtocol {
    #[must_use]
    pub fn label(&self) -> &str {
        if self.protocol.eq_ignore_ascii_case("rtmp") {
            "RTMP"
        } else if self.protocol.eq_ignore_ascii_case("srt")
            || self.protocol.eq_ignore_ascii_case("rtmpsrt")
        {
            "SRT"
        } else if self.protocol.eq_ignore_ascii_case("whip") {
            "WHIP"
        } else {
            &self.protocol
        }
    }

    #[must_use]
    pub fn endpoint(&self) -> Option<String> {
        let is_rtmp = self.protocol.eq_ignore_ascii_case("rtmp");
        let is_srt = self.protocol.eq_ignore_ascii_case("srt")
            || self.protocol.eq_ignore_ascii_case("rtmpsrt");
        let is_whip = self.protocol.eq_ignore_ascii_case("whip");
        if !(is_rtmp || is_srt || is_whip) {
            return None;
        }
        let valid = |raw: &str| {
            if raw.chars().any(|c| c.is_whitespace() || c.is_control()) {
                return false;
            }
            let Ok(url) = reqwest::Url::parse(raw) else {
                return false;
            };
            if url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.fragment().is_some()
            {
                return false;
            }
            match url.scheme() {
                "https" => is_whip,
                "rtmp" | "rtmps" | "rtmpt" | "rtmpe" => is_rtmp,
                // Preserve server-issued schemes; never convert RTMP into generic SRT.
                "rtmpsrt" | "srt" => is_srt,
                _ => false,
            }
        };
        if valid(&self.addr) && !self.code.is_empty() {
            let joined = if self.addr.ends_with('/') {
                format!("{}{}", self.addr, self.code)
            } else {
                format!("{}/{}", self.addr, self.code)
            };
            return valid(&joined).then_some(joined);
        }
        if is_whip && valid(&self.addr) {
            let url = reqwest::Url::parse(&self.addr).ok()?;
            if !url.path().trim_matches('/').is_empty() {
                return Some(self.addr.clone());
            }
        }
        None
    }

    #[must_use]
    pub fn warning(&self) -> &'static str {
        if self.protocol.eq_ignore_ascii_case("srt")
            || self.protocol.eq_ignore_ascii_case("rtmpsrt")
        {
            "B站专用 SRT 协议，部分推流软件可能不兼容"
        } else if self.protocol.eq_ignore_ascii_case("whip") {
            "需使用支持 WHIP 协议的推流软件"
        } else if self.protocol.eq_ignore_ascii_case("rtmp") {
            ""
        } else {
            "客户端专用线路，暂不支持直接用于推流软件"
        }
    }
}
