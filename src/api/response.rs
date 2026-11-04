use crate::error::{BiliLiveError, Result};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

#[derive(Deserialize)]
pub(super) struct Envelope {
    pub code: i64,
    #[serde(default)]
    pub data: Value,
}

impl Envelope {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|_| BiliLiveError::Parse("服务器返回数据格式异常".into()))
    }

    pub fn checked(self) -> Result<Self> {
        if self.code != 0 {
            // Server messages may echo credentials; expose only the numeric status.
            return Err(BiliLiveError::Api(format!(
                "请求未通过 (错误码 {})",
                self.code
            )));
        }
        Ok(self)
    }

    pub fn decode<T: DeserializeOwned>(self) -> Result<T> {
        serde_json::from_value(self.checked()?.data)
            .map_err(|_| BiliLiveError::Parse("服务器返回数据不完整".into()))
    }
}
