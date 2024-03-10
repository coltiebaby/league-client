pub mod client;
pub mod core;
pub mod connector;

pub use client::Client;

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
    WebsocketCreation(String),
    #[error("failed to send message")]
    SendErr,
}
