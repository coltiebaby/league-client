# league-client
LCU - The League Client Rust Wrapper

Just wanted to share a way to connect to the service through rust.
Currently very rough and only works on osx right now.

View the [subscription example](examples/subscription.rs) to see how to get started.

Speaker allows you to communicate with the socket.

## Types
I'm very lazy and I'm not going to hand type these but I do have an idea in the future
to generate all these.

For now you can just use `[serde_json::from_value](https://docs.rs/serde_json/latest/serde_json/fn.from_value.html)` to do your thing.
This will allow to make only the ones you need going forward.

```rust
let speaker = league_client::subscribe(connected).await;
while let Ok(msg) = speaker.reader.recv_async().await {
    println!("{msg:?}");
}
```
