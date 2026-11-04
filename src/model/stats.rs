use super::ids::{deserialize_amount, deserialize_counter};
use serde::Deserialize;

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(default)]
pub struct LiveStats {
    #[serde(rename = "AddFans", deserialize_with = "deserialize_counter")]
    pub fans: i64,
    #[serde(rename = "DanmuNum", deserialize_with = "deserialize_counter")]
    pub danmaku: i64,
    /// `HamsterRmb` is RMB in yuan and may be fractional.
    #[serde(rename = "HamsterRmb", deserialize_with = "deserialize_amount")]
    pub revenue: f64,
    #[serde(rename = "LiveTime", deserialize_with = "deserialize_counter")]
    pub seconds: i64,
    #[serde(rename = "MaxOnline", deserialize_with = "deserialize_counter")]
    pub max_online: i64,
    #[serde(rename = "NewFansClub", deserialize_with = "deserialize_counter")]
    pub fans_club: i64,
    #[serde(rename = "WatchedCount", deserialize_with = "deserialize_counter")]
    pub viewers: i64,
}
