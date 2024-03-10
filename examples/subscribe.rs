use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let lc = league_client::client::Client::new().unwrap();
    let req = lc.wss().unwrap();

    let builder = league_client::connector::Connector::builder();
    let c = builder.insecure(true).build();

    let connected = c.connect(req).await;

    let speaker = league_client::connector::subscribe(connected).await;

    let msg = (5, "OnJsonApiEvent");
    let msg = serde_json::to_string(&msg).unwrap();

    speaker.send(msg).await;

    loop {
        tokio::select!{
            Ok(msg) = speaker.reader.recv_async() => {
                println!("{msg:?}");
            }
            _ = sleep(Duration::from_secs(60)) => {
                break;
            }
        };
    }

    println!("finished");
}
