//! Client creates a http_client and a socket connection to the LCU server.
//!
//! ```rust
//! use league_client;
//!
//! async fn create_connection() -> Result<league_client::Client, league_client::Error> {
//!     let client = league_client::Client::builder()?.insecure(true).build()?;
//!     Ok(client)
//! }
//! ```

use std::process;

use base64::prelude::*;
use tungstenite::client::IntoClientRequest;

use super::{Error, LCResult as Result};

#[derive(Default, Debug)]
pub struct ClientBuilder {
    token: String,
    port: String,
    insecure: bool,
}

impl ClientBuilder {
    /// Attempts to look for the LeagueClientUx process.
    ///
    /// - Uses ps and grep if you're in the linux family.
    /// - Uses wmic if on the windows family.
    ///
    /// If it finds it, it will grab the token and port from the args.
    /// Set insecure to true to avoid having to pass in the riot key.
    pub fn from_process() -> Result<Self> {
        let processes = from_process("LeagueClientUx").ok_or(Error::AppNotRunning)?;
        let process = processes.get(0).ok_or(Error::AppNotRunning)?;
        let (token, port) = parse_process(process)?;

        Ok(Self {
            token,
            port,
            ..Default::default()
        })
    }

    /// Skip cert check.
    pub fn insecure(mut self, value: bool) -> Self {
        self.insecure = value;
        self
    }

    /// Consumes the builder and returns a [Client]
    pub fn build(self) -> Result<Client> {
        let basic = self.auth();
        let http_client = self.reqwest_client()?;
        let connector = crate::connector::Connector::builder()
            .insecure(self.insecure)
            .build()?;

        let addr = format!("127.0.0.1:{}", self.port);

        Ok(Client {
            basic,
            connector,
            addr,
            http: http_client,
        })
    }

    fn auth(&self) -> String {
        let auth = format!("riot:{}", self.token);
        format!("Basic {}", BASE64_STANDARD.encode(auth))
    }

    fn reqwest_client(&self) -> Result<reqwest::Client> {
        let mut headers = reqwest::header::HeaderMap::new();
        let mut auth = reqwest::header::HeaderValue::from_str(&self.auth())
            .map_err(|e| Error::HttpClientCreation(e.to_string()))?;
        auth.set_sensitive(true);

        headers.insert(reqwest::header::AUTHORIZATION, auth);

        let mut client_builder = reqwest::Client::builder().default_headers(headers);

        if self.insecure {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        client_builder
            .build()
            .map_err(|e| Error::HttpClientCreation(e.to_string()))
    }
}

pub struct Client {
    basic: String,
    connector: crate::connector::Connector,
    http: reqwest::Client,

    pub addr: String,
}

impl Client {
    pub fn builder() -> Result<ClientBuilder> {
        ClientBuilder::from_process()
    }

    /// Connect to the LCU client. Returns a socket connection aliased as [Connected](`crate::connector::Connected`).
    pub async fn connect_to_socket(&self) -> Result<crate::connector::Connected> {
        let mut req = format!("wss://{}", &self.addr)
            .into_client_request()
            .map_err(|e| Error::WebsocketRequest(e.to_string()))?;

        let auth = self.basic.clone();
        let headers = req.headers_mut();

        headers.insert(
            "authorization",
            auth.parse()
                .map_err(|_| Error::WebsocketRequest("failed to createa an auth header".into()))?,
        );

        self.connector.connect(req).await
    }

    /// Gives back a copy of the reqwest client. [Read More](https://docs.rs/reqwest/latest/reqwest/)
    pub fn http_client(&self) -> reqwest::Client {
        self.http.clone()
    }
}

#[cfg(target_family = "unix")]
fn from_process(process: &str) -> Option<Vec<String>> {
    let ps = process::Command::new("ps")
        .args(["x", "-A", "-o args"])
        .stdout(process::Stdio::piped())
        .spawn()
        .ok()?;

    let mut grep = process::Command::new("grep");
    grep.arg(process).stdin(ps.stdout?);

    let output = String::from_utf8(grep.output().ok()?.stdout).ok()?;
    let lines = output.lines();

    let lines: Vec<String> = lines
        .filter(|x| x.contains("--app-port") && x.contains("--remoting-auth-token"))
        .map(String::from)
        .collect();

    Some(lines)
}

#[cfg(target_family = "windows")]
fn from_process(process: &str) -> Option<Vec<String>> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let wmic = process::Command::new("WMIC")
        .args(["path", "win32_process", "get", "Caption,Commandline"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    let stdout = String::from_utf8(wmic.stdout).ok()?;

    let process_exe = format!("{}.exe", process);

    let lines = stdout.lines()
        .filter(|x| x.contains(&process_exe))
        .filter(|x| x.contains("--app-port") && x.contains("--remoting-auth-token"))
        .map(String::from)
        .collect();

    Some(lines)
}

fn parse_process(value: &str) -> Result<(String, String)> {
    let args: Vec<&str> = value.split("\"").collect();

    let mut token = Err(Error::AppNotRunning);
    let mut port = Err(Error::AppNotRunning);

    for arg in args {
        if arg.starts_with("--remoting-auth-token") {
            let val = arg.split("=").last();
            token = val.ok_or(Error::AppNotRunning);
        }
        if arg.starts_with("--app-port") {
            let val = arg.split("=").last();
            port = val.ok_or(Error::AppNotRunning);
        }
    }

    Ok((token?.to_string(), port?.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_from_string() {
        let example = r#"/Applications/League of Legends.app/Contents/LoL/League of Legends.app/Contents/MacOS/LeagueClientUx --riotclient-auth-token=token --riotclient-app-port=12345 --no-rads --disable-self-update --region=NA --locale=en_US --client-config-url=https://clientconfig.rpg.riotgames.com --riotgamesapi-standalone --riotgamesapi-settings=token --rga-lite --remoting-auth-token=token --app-port=12345 --install-directory=/Applications/League of Legends.app/Contents/LoL --app-name=LeagueClient --ux-name=LeagueClientUx --ux-helper-name=LeagueClientUxHelper --log-dir=LeagueClient Logs --crash-reporting=crashpad --crash-environment=NA1 --app-log-file-path=/Applications/League of Legends.app/Contents/LoL/Logs/LeagueClient Logs/2024-03-09T14-52-20_5736_LeagueClient.log --app-pid=5736 --output-base-dir=/Applications/League of Legends.app/Contents/LoL --no-proxy-server --ignore-certificate-errors"#;

        let (token, port) = parse_process(example).expect("usable client");
        assert_eq!(port, "12345".to_string());
        assert_eq!(token, "token".to_string())
    }
}
