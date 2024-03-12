pub mod client;
pub mod connector;
pub mod core;

pub use client::Client;
pub use connector::Speaker;

pub use connector::subscribe;

pub type LCResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown")]
    Unknown,
    #[error("league of legends is not running")]
    AppNotRunning,
    #[error("failed to create the client: {0}")]
    HttpClientCreation(String),
    #[error("failed to create the client: {0}")]
    Websocket(String),
    #[error("failed to make a request: {0}")]
    WebsocketRequest(String),
    #[error("failed to connect to stream: {0}")]
    Stream(#[from] std::io::Error),
    #[error("tls stream failure: {0}")]
    Tls(#[from] native_tls::Error),
    #[error("failed to send message")]
    SendErr,
    #[error("websocket connection error: {0}")]
    Tungstenite(#[from] tungstenite::error::Error),
}
