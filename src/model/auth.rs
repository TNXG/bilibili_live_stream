use serde::{Deserialize, Serialize};

// Intentionally no Debug on credentials or QR tokens.
#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub room_id: u64,
    pub sessdata: String,
    pub csrf_token: String,
}

/// Logged-in account shown in the UI; no cookies live here.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Account {
    pub mid: u64,
    pub name: String,
}

#[derive(Clone, Deserialize)]
pub struct QrCodeData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Clone, Deserialize)]
pub struct QrPoll {
    pub code: i64,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub message: String,
}
