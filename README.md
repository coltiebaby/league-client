# league-client
LCU - The League Client Rust Wrapper

Speaker allows you to communicate with the socket.

## Types
I'm very lazy and I'm not going to hand type these but I do have an idea in the future
to generate all these.

For now you can just use [serde_json::from_value](https://docs.rs/serde_json/latest/serde_json/fn.from_value.html) to do your thing.
This will allow to make only the ones you need going forward.

## Example
Rather run it instead? Just log in and run the [example](examples/connect_to_client.rs) to see it in action.
```rust
let builder = client::Client::builder().unwrap();
let lc = builder.insecure(true).build().unwrap();
let connected = lc.connect_to_socket().await.unwrap();

let speaker = league_client::subscribe(connected).await;

// You must send this to get events.
let msg = (5, "OnJsonApiEvent");
let msg = serde_json::to_string(&msg).unwrap();
speaker.send(msg).await.expect("should have sent a message");

while let Ok(msg) = speaker.reader.recv_async().await {
    println!("{msg:?}");
}
```

