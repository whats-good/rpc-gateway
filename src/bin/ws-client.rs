//! A simple example of hooking up stdin/stdout to a WebSocket stream.
//!
//! This example will connect to a server specified in the argument list and
//! then forward all data read on stdin to the server, printing out all data
//! received on stdout.
//!
//! Note that this is not currently optimized for performance, especially around
//! buffer management. Rather it's intended to show an example of working with a
//! client.
//!
//! You can use this example together with the `server` example.

use std::env;

use futures::{SinkExt, TryStreamExt};
use futures_util::{future, pin_mut, StreamExt};
use rpc_gateway::rpc::{RpcInboundRequest, RpcOutboundRequest};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:9002".to_string();

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut sink, mut stream) = ws_stream.split();

    // let stdin_to_ws = stdin_rx.map(Ok).forward(write);

    let rpc_request = RpcInboundRequest {
        id: Some(1234),
        method: "eth_subscribe".to_string(), // TODO: accept `AsRef<String>`
        params: Some(vec!["newHeads".to_string()]),
    };

    let payload = serde_json::to_string(&rpc_request).unwrap();

    sink.send(Message::Text(payload)).await.unwrap();

    while let Some(x) = stream.try_next().await.unwrap() {
        println!("{x:?}");
    }
}
