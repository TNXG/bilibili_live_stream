mod auth;
mod client;
mod config;
mod cookies;
mod live;
mod response;

use config::Endpoints;
use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Api {
    client: Client,
    jar: Arc<Jar>,
    endpoints: Endpoints,
    cookie_path: PathBuf,
}

/// Hosts B站 uses for the QR login cross-domain redirect.
#[must_use]
pub fn login_host_allowed(host: &str) -> bool {
    ["bilibili.com", "biligame.com"]
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")))
}
