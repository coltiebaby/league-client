use futures_util::{StreamExt, SinkExt};
use futures_util::stream::{SplitSink, SplitStream};
use tokio_tungstenite::WebSocketStream;
use tokio_native_tls::TlsStream;
use tokio::net::TcpStream;
use tungstenite::Message;

use crate::{LCResult as Result, Error, core};

pub type Connected = WebSocketStream<TlsStream<TcpStream>>;

/// Stores information of the subscription.
///
/// Once speaker is dropped, it will unsubscribe from the events and broadcast
/// that it is finished to the read/write tasks.
pub struct Speaker {
    finish: tokio::sync::broadcast::Sender<bool>,
    writer: flume::Sender<String>,
    _handles: Vec<tokio::task::JoinHandle<()>>,

    pub reader: flume::Receiver<core::Incoming>,
}

impl Speaker {
    pub async fn send(&self, msg: String) -> Result<()> {
        self.writer.send_async(msg).await.or(Err(Error::SendErr))
    }

    fn try_send(&self, msg: String) -> Result<()> {
        self.writer.try_send(msg).or(Err(Error::SendErr))
    }
}

impl Drop for Speaker {
    fn drop(&mut self) {
        let msg = (6, "OnJsonApiEvent");
        if let Ok(msg) = serde_json::to_string(&msg) {
            if let Err(e) = self.try_send(msg) {
                tracing::error!("failed to unsubscribe: {e}");
            }
        };

        if let Err(e) = self.finish.send(true) {
            tracing::error!("failed to send broadcast: {e}");
        };
    }
}

/// Start a subscription to the socket.
///
/// Use the speaker to communicate with the socket.
pub async fn subscribe(socket: Connected) -> Speaker {
    let (cleanup_tx, cleanup_rx1) = tokio::sync::broadcast::channel(1);
    let cleanup_rx2 = cleanup_tx.subscribe();

    let (reader_tx, reader_rx) = flume::unbounded();
    let (writer_tx, writer_rx) = flume::unbounded();

    let (write, read) = socket.split();

    let read_handle = tokio::task::spawn(read_from(cleanup_rx1, reader_tx, read));
    let write_handle = tokio::task::spawn(write_to(cleanup_rx2, write, writer_rx));

    Speaker {
        finish: cleanup_tx,
        reader: reader_rx,
        writer: writer_tx,
        _handles: vec![read_handle, write_handle],
    }
}

async fn read_from(mut end: tokio::sync::broadcast::Receiver<bool>, tx: flume::Sender<core::Incoming>, mut read: SplitStream<Connected>) {
    loop {
        tokio::select! {
            Some(msg) = read.next() => {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(_) => {
                        tracing::warn!("channel disconnect");
                        break;
                    }
                };

                let msg = msg.to_string();
                if msg.is_empty() {
                    continue;
                }

                let incoming = serde_json::from_str::<core::Incoming>(&msg);
                let incoming = match incoming {
                    Ok(incoming) => incoming,
                    Err(_) => {
                        tracing::warn!("failed to parse msg into incoming: {msg}");
                        continue;
                    },
                };

                let resp = tx.send_async(incoming).await;

                if resp.is_err() {
                    tracing::warn!("channel disconnect");
                    break;
                }
            },
            _ = end.recv() => { break },
        };
    }
}

async fn write_to(mut end: tokio::sync::broadcast::Receiver<bool>, mut tx: SplitSink<Connected, Message>, read: flume::Receiver<String>) {
    loop {
        tokio::select! {
            msg = read.recv_async() => {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(_) => {
                        tracing::warn!("channel disconnect");
                        break;
                    }
                };


                let resp = tx.send(Message::Text(msg)).await;

                if resp.is_err() {
                    tracing::warn!("channel disconnect");
                    break;
                }
            },
            _ = end.recv() => { break },
        };
    }
}

/// Creates a connection to the wanted websocket. Use this if you want to set up
/// the connection yourself.
pub struct Connector {
    tls: tokio_native_tls::TlsConnector,
}

impl Connector {
    fn new(tls: tokio_native_tls::TlsConnector) -> Self {
        Self { tls }
    }

    /// create a builder to set up the tls connection.
    pub fn builder() -> ConnectorBuilder {
        ConnectorBuilder::default()
    }

    /// creates a stream and wraps it with tls settings. It will then
    /// create asocket from the said stream.
    ///
    /// the request must have a basic auth included or it will not complete.
    pub async fn connect(&self, req: tungstenite::http::Request<()>) -> Result<Connected> {
        let uri = req.uri();

        let host = uri.host().ok_or(Error::Websocket("host is missing".into()))?;
        let port = uri.port().ok_or(Error::Websocket("port is missing".into()))?;
        let combo = format!("{host}:{port}");

        let stream = tokio::net::TcpStream::connect(&combo).await.map_err(Error::Stream)?;
        let stream = self.tls.connect(&combo, stream).await.map_err(Error::Tls)?;

        let (socket, _) = tokio_tungstenite::client_async(req, stream).await.map_err(Error::Tungstenite)?;

        Ok(socket)
    }
}

#[derive(Default)]
pub struct ConnectorBuilder {
    insecure: bool,
}

impl ConnectorBuilder {
    pub fn insecure(self, value: bool) -> Self {
        Self {
            insecure: value,
        }
    }

    pub fn build(self) -> Result<Connector> {
        let mut connector = native_tls::TlsConnector::builder();

        if self.insecure {
            connector.danger_accept_invalid_certs(true);
        } else {
            // Work out cert
            unimplemented!();
        }

        let connector = connector.build().map_err(|e| Error::Websocket(e.to_string()))?;
        let tls = tokio_native_tls::TlsConnector::from(connector);

        Ok(Connector::new(tls))
    }
}
