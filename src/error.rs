use thiserror::Error;

#[derive(Error, Debug)]
pub enum BiliLiveError {
    #[error("网络请求失败: {0}")]
    NetworkError(#[from] minreq::Error),

    #[error("JSON解析失败: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("文件操作失败: {0}")]
    IoError(#[from] std::io::Error),

    #[error("数字解析失败: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("二维码生成失败: {0}")]
    QrCodeError(#[from] qrcode::types::QrError),

    #[error("图像处理失败: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Cookie文件不存在或无效")]
    _CookieError,

    #[error("登录状态异常: {0}")]
    _LoginError(String),

    #[error("登录状态检查失败: {0}")]
    _LoginStatusError(String),

    #[error("API返回错误: {0}")]
    ApiError(String),

    #[error("分区选择失败: {0}")]
    _AreaSelectionError(String),

    #[error("用户输入错误: {0}")]
    InputError(String),

    #[error("直播操作失败: {0}")]
    _LiveError(String),

    #[error("数据解析失败: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, BiliLiveError>;
