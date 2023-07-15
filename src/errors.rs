use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ServerError {
    #[error("无法成功连接webdriver")]
    WebdriverConnectionError,

    #[error("无法成功连接redis")]
    RedisConnectionError,

    #[error("环境变量:{key:?}错误")]
    InvalidConfig { key: String },
}

#[derive(Error, Debug)]
pub(crate) enum ClientError {
    #[error("无法成功连接redis")]
    RedisConnectionError,

    #[error("无法成功连接webdriver")]
    WebdriverConnectionError,

    #[error("登录失败")]
    LoginFailed,

    #[error("cookie有错误")]
    CookieError,
}

// Api返回的错误信息
#[derive(Error, Debug)]
pub enum DmApiError {
    #[error("B-00203-200-034::您选购的商品信息已过期，请重新查询")]
    ProductEpired,

    #[error("RGV587_ERROR::SM::哎哟喂,被挤爆啦,请稍后重试")]
    SystemBusy,

    #[error("B-00203-200-008::对不起，您选购的商品库存不足，请重新选购")]
    SoldOut,

    #[error("F-10001-10-16-103::对不起，系统繁忙，请稍候再试")]
    BuildOrderSystemBusy,
}
