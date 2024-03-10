use futures_util::{StreamExt, SinkExt};
use futures_util::stream::{SplitSink, SplitStream};
use tokio_tungstenite::WebSocketStream;
use tokio_native_tls::TlsStream;
use tokio::net::TcpStream;
use tungstenite::Message;

use crate::core;

pub type Connected = WebSocketStream<TlsStream<TcpStream>>;

pub struct Speaker {
    finish: tokio::sync::broadcast::Sender<bool>,

    pub reader: flume::Receiver<core::Incoming>,
    writer: flume::Sender<String>,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl Speaker {
    pub async fn send(&self, msg: String) {
        self.writer.send_async(msg).await;
    }

    fn try_send(&self, msg: String){
        self.writer.try_send(msg);
    }
}

impl Drop for Speaker {
    fn drop(&mut self) {
        let msg = (6, "OnJsonApiEvent");
        if let Ok(msg) = serde_json::to_string(&msg) {
            self.try_send(msg)
        };

        self.finish.send(true);
    }
}

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
        handles: vec![read_handle, write_handle],
    }
}

async fn read_from(mut end: tokio::sync::broadcast::Receiver<bool>, tx: flume::Sender<core::Incoming>, mut read: SplitStream<Connected>) {
    loop {
        tokio::select! {
            Some(Ok(msg)) = read.next() => {
                let msg = msg.to_string();
                if msg.is_empty() {
                    continue;
                }

                let incoming: core::Incoming = serde_json::from_str(&msg).unwrap();
                tx.send_async(incoming).await;
            },
            _ = end.recv() => { break },
        };
    }
}

async fn write_to(mut end: tokio::sync::broadcast::Receiver<bool>, mut tx: SplitSink<Connected, Message>, read: flume::Receiver<String>) {
    loop {
        tokio::select! {
            Ok(msg) = read.recv_async() => {
                tx.send(Message::Text(msg)).await;
            },
            _ = end.recv() => { break },
        };
    }
}

pub struct Connector {
    tls: tokio_native_tls::TlsConnector,
}

impl Connector {
    fn new(tls: tokio_native_tls::TlsConnector) -> Self {
        Self { tls }
    }

    pub fn builder() -> ConnectorBuilder {
        ConnectorBuilder::default()
    }

    pub async fn connect(&self, req: tungstenite::http::Request<()>) -> Connected {
        let uri = req.uri();

        let host = uri.host().unwrap();
        let port = uri.port().unwrap();
        let combo = format!("{host}:{port}");

        let stream = tokio::net::TcpStream::connect(&combo).await.unwrap();
        let stream = self.tls.connect(&combo, stream).await.unwrap();

        let (socket, _) = tokio_tungstenite::client_async(req, stream).await.expect("Failed to connect");

        socket
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
            ..self
        }
    }

    pub fn build(self) -> Connector {
        let mut connector = native_tls::TlsConnector::builder();

        if self.insecure {
            connector.danger_accept_invalid_certs(true);
        } else {
            // Work out cert
            unimplemented!();
        }

        let connector = connector.build().unwrap();
        let tls = tokio_native_tls::TlsConnector::from(connector);

        Connector { tls }
    }
}
