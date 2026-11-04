use thiserror::Error;

#[derive(Debug, Error)]
pub enum BiliLiveError {
    #[error("网络请求失败: {0}")]
    Network(String),
    #[error("API 请求失败: {0}")]
    Api(String),
    #[error("数据解析失败: {0}")]
    Parse(String),
    #[error("文件或终端操作失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("二维码生成失败: {0}")]
    QrCode(#[from] qrcode::types::QrError),
    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, BiliLiveError>;
