pub mod client;
pub mod core;
pub mod connector;

use tungstenite::http;

pub type LCUResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown")]
    Unknown,
    #[error("league of legends is not running")]
    AppNotRunning,
    #[error("failed to make client: {0}")]
    HttpClientError(String),
    #[error("request error: {0}")]
    Request(#[from] http::Error),
    #[error("websocket: {0}")]
    Websocket(String),
    #[error("request is missing host or port")]
    Uri,
}
