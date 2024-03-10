use std::process;

use base64::prelude::*;
use tungstenite::http;
use tungstenite::client::IntoClientRequest;

use super::{LCUResult, Error};

#[derive(Debug)]
pub struct Client {
    pub token: String,
    pub port: String,
}

impl Client {
    pub fn new() -> LCUResult<Self> {
        let processes = from_process("LeagueClientUx").ok_or(super::Error::AppNotRunning)?;
        let process = processes.get(0).ok_or(super::Error::AppNotRunning)?;

        Self::from_str(process)
    }

    fn from_str(value: &str) -> super::LCUResult<Client> {
        let re = regex::Regex::new(r"--remoting-auth-token=([\w-]*) --app-port=([0-9]*)").unwrap();
        let caps = re.captures(value);
        let caps = caps.unwrap();
        let token: String = caps.get(1).unwrap().as_str().to_string();
        let port: String = caps.get(2).unwrap().as_str().to_string();

        Ok(Client { token, port })
    }

    fn reqwest_client(&self) -> LCUResult<reqwest::Client> {
        let mut headers = reqwest::header::HeaderMap::new();
        let mut auth = reqwest::header::HeaderValue::from_str(&self.auth()).map_err(|_| Error::Unknown)?;
        auth.set_sensitive(true);

        headers.insert("authorization", auth);

        reqwest::Client::builder()
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|_| Error::Unknown)
    }

    fn auth(&self) -> String {
        let auth = format!("riot:{}", self.token);
        format!("basic {}", BASE64_STANDARD.encode(auth))
    }

    pub fn uri(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    pub async fn patch(&self, path: &str, data: String) -> LCUResult<reqwest::Response> {
        let u = format!("https://{}/{}", self.uri(), path);

        let client = self.reqwest_client()?;
        client.patch(u)
            .body(data)
            .send()
            .await
            .map_err(|_| Error::Unknown)
    }

    pub fn wss(&self) -> LCUResult<http::Request<()>> {
        let auth = self.auth();
        let mut req = format!("wss://127.0.0.1:{}", self.port)
            .into_client_request().map_err(|_| Error::Unknown)?;

        req.headers_mut().insert("authorization", auth.parse().map_err(|_| Error::Unknown)?);

        Ok(req)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_from_string() {
        let example = r#"/Applications/League of Legends.app/Contents/LoL/League of Legends.app/Contents/MacOS/LeagueClientUx --riotclient-auth-token=token --riotclient-app-port=12345 --no-rads --disable-self-update --region=NA --locale=en_US --client-config-url=https://clientconfig.rpg.riotgames.com --riotgamesapi-standalone --riotgamesapi-settings=token --rga-lite --remoting-auth-token=token --app-port=12345 --install-directory=/Applications/League of Legends.app/Contents/LoL --app-name=LeagueClient --ux-name=LeagueClientUx --ux-helper-name=LeagueClientUxHelper --log-dir=LeagueClient Logs --crash-reporting=crashpad --crash-environment=NA1 --app-log-file-path=/Applications/League of Legends.app/Contents/LoL/Logs/LeagueClient Logs/2024-03-09T14-52-20_5736_LeagueClient.log --app-pid=5736 --output-base-dir=/Applications/League of Legends.app/Contents/LoL --no-proxy-server --ignore-certificate-errors"#;

        let client = Client::from_str(example).expect("usable client");
        assert_eq!(client.port, "12345".to_string())
    }
}
