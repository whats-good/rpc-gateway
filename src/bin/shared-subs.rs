use std::{collections::HashMap, fmt::Debug, num::ParseIntError, ops::Deref, sync::Arc};

use futures::{SinkExt, StreamExt, TryStreamExt};
use rpc_gateway::{
    chain::ChainId,
    node_subscriptions::{
        self, NewInboundSubscriptionMessage, NodeSubscriptionsActorHandle,
        NodeSubscriptionsActorMessage,
    },
    rpc::{
        RpcInboundRequest, RpcInboundSubscriptionRequest, RpcOutboundSubscriptionPayloadResponse,
        RpcOutboundSubscriptionPayloadResponseParams, RpcOutboundSuccessResponse, SubscriptionId,
        SubscriptionKind, JSON_RPC_VERSION,
    },
    settings::{RawSettings, Settings},
    tracing::init_tracing,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, event, info, instrument, warn, Level};

#[instrument]
fn get_settings() -> &'static Settings {
    let settings: Settings = RawSettings::from_config_file("config.toml")
        .unwrap()
        .try_into()
        .unwrap();

    event!(Level::INFO, ?settings, "Booting");
    settings.leak()
}

async fn accept_connection(stream: TcpStream, sub_handle: NodeSubscriptionsActorHandle) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (sink, mut stream) = ws_stream.split();
    let sink = Arc::new(Mutex::new(sink));

    let mut subscriptions: HashMap<SubscriptionId, CancellationToken> = HashMap::new();

    let task_tracker = TaskTracker::new();
    let entire_ws_loop_cancellation_token = CancellationToken::new();
    let inbound_messages_loop = async {
        while let Ok(msg) = stream.try_next().await {
            let entire_ws_loop_cancellation_token = entire_ws_loop_cancellation_token.clone();
            let msg = match msg {
                Some(msg) => msg,
                None => {
                    break;
                }
            };
            let req = match msg {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    serde_json::from_str::<RpcInboundRequest>(&text)
                }
                tokio_tungstenite::tungstenite::Message::Binary(binary) => {
                    serde_json::from_slice::<RpcInboundRequest>(&binary)
                }
                tokio_tungstenite::tungstenite::Message::Ping(_)
                | tokio_tungstenite::tungstenite::Message::Pong(_)
                | tokio_tungstenite::tungstenite::Message::Close(_)
                | tokio_tungstenite::tungstenite::Message::Frame(_) => {
                    continue;
                    // TODO:
                }
            }
            .unwrap(); // TODO: respond with parse error.

            let req_id = req.id.clone();
            let sub_kind = match req
                .params
                .as_ref()
                .map(|v| v.first().map(|s| s.to_string()))
            {
                Some(Some(s)) if &s == "newHeads" => SubscriptionKind::NewHeads,
                Some(Some(s)) if &s == "newPendingTransactions" => {
                    SubscriptionKind::NewPendingTransactions
                }
                _ => panic!(), // TODO: fix
            };

            warn!(?sub_kind, "inbound received");

            let subscription_request = RpcInboundSubscriptionRequest {
                kind: sub_kind, // TODO: actually parse
                request: req,
            };

            let (tx, rx) = tokio::sync::oneshot::channel();
            let msg = NewInboundSubscriptionMessage {
                request: subscription_request,
                respond_to: tx,
            };
            let subscription_id = SubscriptionId::rand();
            let local_cancellation_token = CancellationToken::new();

            subscriptions.insert(subscription_id.clone(), local_cancellation_token.clone());

            let inner_sink = Arc::clone(&sink);
            task_tracker.spawn(async move {
                // TODO: add a timeout here
                let mut bcast_rx = rx
                    .await
                    .expect("expected to receive a bcast channel for subscriptions"); // TODO: maybe we can respond with an error message here?

                let bcast_loop = async {
                    let ack_response = RpcOutboundSuccessResponse {
                        id: req_id,
                        jsonrpc: JSON_RPC_VERSION,
                        result: String::from(subscription_id.clone()),
                    };
                    // TODO: is it better to deserialize into bytes instead?
                    let ack_response_string = serde_json::to_string(&ack_response)
                        .expect("expected the ack response to serialize correctly");
                    let mut guard = inner_sink.lock().await;

                    guard
                        .send(Message::text(ack_response_string))
                        .await
                        .expect("expected to send ack through the socket");
                    drop(guard);

                    while let Ok(next) = bcast_rx.recv().await {
                        // TODO: the bcast tx sometimes can't keep up with the volume of incoming messages. what's a good way to recover when this happens?
                        let response_payload = RpcOutboundSubscriptionPayloadResponse {
                            jsonrpc: JSON_RPC_VERSION,
                            method: "eth_subscribe".to_string(), // TODO: this should be a static string
                            params: RpcOutboundSubscriptionPayloadResponseParams {
                                result: next,
                                subscription: String::from(subscription_id.clone()),
                            },
                        };
                        let response_str = serde_json::to_string(&response_payload)
                            .expect("expected subscription response payload to serialize properly");
                        let mut guard = inner_sink.lock().await;
                        guard
                            .send(Message::Text(response_str))
                            .await
                            .expect("expected to send the subscription through the socket"); // TODO: handle failed message
                        drop(guard);
                    }
                };

                tokio::select! {
                    _ = local_cancellation_token.cancelled() => {
                        info!("subscription was cancelled");
                    }
                    _ = entire_ws_loop_cancellation_token.cancelled() => {
                        info!("websocket was closed, no need to keep listening to outbounds.");// TODO: see if this was a proper close
                    }
                    _ = bcast_loop => {
                        error!("broadcast loop ended unexpectedly."); // TODO: need to abort the client socket here.
                        entire_ws_loop_cancellation_token.cancel();
                    },
                };
            });

            sub_handle
                .send(NodeSubscriptionsActorMessage::NewInboundSubscription(msg))
                .await
                .expect("Expected to successfully send the inbound sub message to the subs actor");
            // TODO: maybe we can respond with an error message here?
        }

        // TODO: see if this was a proper close
        warn!("inbound message loop ended unexpectedly. cancelling internal drivers.");
        entire_ws_loop_cancellation_token.cancel();
    };

    tokio::select! {
        _ = inbound_messages_loop => {
            error!("inbound messages loop ended unexpectedly");
        }
        _ = entire_ws_loop_cancellation_token.cancelled() => {
            warn!("inbound messages loop ended via token")
        }
    }
}

#[tokio::main]
async fn main() {
    init_tracing();
    let settings = get_settings();

    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    let chain_id: ChainId = 1.into();
    let ws_endpoints = settings
        .chains_to_targets
        .get(&chain_id)
        .unwrap()
        .ws
        .clone();

    let (sub_actor, sub_handle) = node_subscriptions::actor(ws_endpoints);

    tokio::spawn(sub_actor.run());

    let listen_handler = tokio::spawn(async move {
        let internal_tracker = TaskTracker::new();
        while let Ok((stream, _)) = listener.accept().await {
            internal_tracker.spawn(accept_connection(stream, sub_handle.clone()));
        }
    });

    listen_handler.await;
    info!("Listening on: {}", addr);
}
