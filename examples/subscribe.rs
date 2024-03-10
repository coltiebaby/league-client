use tokio::time::{interval, Duration};
use league_client::client;

#[tokio::main]
async fn main() {
    let builder = client::Client::builder().unwrap();
    let lc = builder.insecure(true).build().unwrap();

    let connected = lc.connect_to_socket().await.unwrap();

    let speaker = league_client::subscribe(connected).await;

    let msg = (5, "OnJsonApiEvent");
    let msg = serde_json::to_string(&msg).unwrap();

    speaker.send(msg).await.expect("should have sent a message");
    let mut ticker = interval(Duration::from_secs(60));

    let mut counter = 0;
    loop {
        tokio::select!{
            Ok(msg) = speaker.reader.recv_async() => {
                println!("{msg:?}");
            }
            _ = ticker.tick() => {
                counter += 1;
            }
        };

        if counter == 2 {
            break;
        }
    }

    println!("finished");
}
