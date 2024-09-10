use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use anyhow::bail;
use futures::{stream::SplitStream, SinkExt, StreamExt, TryStreamExt};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    sync::{Mutex, OnceCell},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, instrument, warn};

use crate::{
    rpc::{
        RpcInboundSubscriptionPayloadResponse, RpcInboundSubscriptionRequest,
        RpcInboundSuccessResponse, SubscriptionKind,
    },
    settings::TargetEndpointsForChain,
    target_endpoint::WebSocketTargetEndpoint,
};

type NodeSubscriptionWebSocketSplitStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

trait RpcSubscriptionWebSocketStream {
    async fn get_first_inbound_success_response(
        &mut self,
    ) -> anyhow::Result<RpcInboundSuccessResponse>;
}

impl RpcSubscriptionWebSocketStream for NodeSubscriptionWebSocketSplitStream {
    async fn get_first_inbound_success_response(
        &mut self,
    ) -> anyhow::Result<RpcInboundSuccessResponse> {
        loop {
            let msg = match self.try_next().await? {
                Some(msg) => msg,
                None => bail!("Node websocket closed unexpectedly"),
            };

            info!(?msg, "received message during sub init phase");

            let response = match msg {
                Message::Text(msg) => serde_json::from_str::<RpcInboundSuccessResponse>(&msg),
                Message::Binary(msg) => serde_json::from_slice::<RpcInboundSuccessResponse>(&msg),
                Message::Ping(_) => {
                    continue;
                } // TODO:
                Message::Pong(_) => {
                    continue;
                } // TODO:
                Message::Close(_) => {
                    continue;
                } // TODO:
                Message::Frame(_) => {
                    continue;
                } // TODO:
            }?;

            return Ok(response);
        }
    }
}

impl WebSocketTargetEndpoint {
    pub async fn init_subscription(
        &self,
        inbound: &RpcInboundSubscriptionRequest,
    ) -> anyhow::Result<NodeSubscriptionWebSocketSplitStream> {
        let (ws_stream, _) = connect_async(self.url.as_str()).await?;

        let (mut sink, mut stream) = ws_stream.split();
        let outbound_request = serde_json::to_string(&inbound.request)?;
        sink.send(Message::Text(outbound_request)).await?;
        // TODO: add a tokio::select + timeout here
        // TODO: consider preserving the raw input for this

        let init_success_response = stream.get_first_inbound_success_response().await?; // TODO: do a timeout here
        if init_success_response.id != inbound.request.id {
            bail!("inbound success response was for a different outbound request")
        }
        if inbound.request.method != "eth_subscribe" {
            // TODO: add parsing logic for subscription init requests, init responses and payload responses
            bail!("inbound success response was not for a subscription")
        }

        Ok(stream)
    }
}

/////////////
// NodeSubscriptionsActor

type RpcSubscriptionPayload = Value; // TODO: consider ARC, and consider a struct;
type RpcSubscriptionBroadcastSender = tokio::sync::broadcast::Sender<RpcSubscriptionPayload>;
type RpcSubscriptionBroadcastReceiver = tokio::sync::broadcast::Receiver<RpcSubscriptionPayload>;
type RpcSubscriptionBroadcastSenderOneShotSender =
    tokio::sync::oneshot::Sender<RpcSubscriptionBroadcastReceiver>;

type ActiveSubscriptionsMap =
    HashMap<SubscriptionKind, Arc<OnceCell<RpcSubscriptionBroadcastSender>>>;

pub struct NodeSubscriptionsActor {
    active_subscriptions: Arc<Mutex<ActiveSubscriptionsMap>>,
    inbox: tokio::sync::mpsc::Receiver<NodeSubscriptionsActorMessage>,
    websocket_endpoints: Vec<WebSocketTargetEndpoint>,
}

pub struct NewInboundSubscriptionMessage {
    pub request: RpcInboundSubscriptionRequest,
    pub respond_to: RpcSubscriptionBroadcastSenderOneShotSender,
}

pub enum NodeSubscriptionsActorMessage {
    NewInboundSubscription(NewInboundSubscriptionMessage),
}

impl TargetEndpointsForChain {}

impl NodeSubscriptionsActor {
    pub async fn init_subscription_loop(
        endpoints: Vec<WebSocketTargetEndpoint>,
        request: RpcInboundSubscriptionRequest,
    ) -> anyhow::Result<NodeSubscriptionWebSocketSplitStream> {
        for endpoint in endpoints.iter() {
            // TODO: add timeouts and loop bounds
            // TODO: enable tls
            match endpoint.init_subscription(&request).await {
                Ok(s) => return Ok(s),
                Err(err) => {
                    warn!(?err, "Could not initialize outbound subscription");
                    continue;
                }
            };
        }

        error!(?request, "ran out of websocket endpoints");

        bail!("Ran out of websocket endpoints")
    }

    #[instrument]
    async fn drive_subscription_broadcast(
        mut stream: NodeSubscriptionWebSocketSplitStream,
        broadcast_tx: RpcSubscriptionBroadcastSender,
    ) {
        while let Some(next) = stream.next().await {
            // TODO: how can i make it so that this loop doesn't even run as soon as the receiver count is zero?
            info!(
                receiver_count = broadcast_tx.receiver_count(),
                buffer_size = broadcast_tx.len(),
                "Node subscription message received"
            );
            let msg = match next {
                Ok(msg) => msg,
                Err(err) => {
                    error!(?err, "Node subscription socket errored");
                    break;
                }
            };

            info!(?msg, "node sub received payload");

            let response = match msg {
                Message::Text(msg) => {
                    serde_json::from_str::<RpcInboundSubscriptionPayloadResponse>(&msg)
                }
                Message::Binary(msg) => {
                    serde_json::from_slice::<RpcInboundSubscriptionPayloadResponse>(&msg)
                }
                Message::Ping(_) => {
                    continue;
                } // TODO:
                Message::Pong(_) => {
                    continue;
                } // TODO:
                Message::Close(_) => {
                    continue;
                } // TODO:
                Message::Frame(_) => {
                    continue;
                } // TODO:
            };

            let send_result = match response {
                Ok(payload) => broadcast_tx.send(payload.params.result),
                Err(err) => {
                    error!(?err, "Node subscription socket did not return subscription payload in expected format.");
                    break;
                }
            };

            if let Err(err) = send_result {
                warn!(
                    ?err,
                    "Error while broadcasting node subscription"
                );
                break;
            } else {
                info!("successfully sent node subscription broadcast")
            }
        }
    }

    async fn respond_with_bcast_rx(
        cell: Arc<OnceCell<RpcSubscriptionBroadcastSender>>,
        endpoints: Vec<WebSocketTargetEndpoint>,
        msg: NewInboundSubscriptionMessage,
        task_tracker: TaskTracker,
        active_subscriptions: Arc<Mutex<ActiveSubscriptionsMap>>,
    ) {
        // TODO: the init function should be allowed to fail. How should everyone recover from the failure?
        let bcast_sender = cell
            .get_or_init(|| async move {
                info!("node subscrpition did not exist before. creating for the first time.");
                let stream =
                    NodeSubscriptionsActor::init_subscription_loop(endpoints, msg.request.clone())
                        .await
                        .expect(
                            "expected to successfully establish a websocket connection", // TODO: remove the expect()
                        );

                let (broadcast_tx, _) =
                    tokio::sync::broadcast::channel::<RpcSubscriptionPayload>(1024); // TODO: make this configurable

                let loop_tx_clone = broadcast_tx.clone();

                // TODO: how do we make sure that this does not even begin execution before there's an active listener?
                // maybe the driving should be handled outside the cell initializer? i.e the spawn should happen
                // after msg.respond_to.send.
                
                task_tracker.spawn(async move {
                    NodeSubscriptionsActor::drive_subscription_broadcast(stream, loop_tx_clone)
                        .await;

                    warn!("Exiting from node subscription loop. Removing the broadcast channel from the table.");
                    let mut guard = active_subscriptions.lock().await;
                    guard.remove(&msg.request.kind);
                    drop(guard);
                });
                broadcast_tx
            })
            .await;
        let bcast_receiver = bcast_sender.subscribe();
        let result = msg.respond_to.send(bcast_receiver);
        if result.is_err() {
            warn!("Could not transmit the subscription broadcast rx to the sender.");
        } else {
            info!("Successfully transmitted the subscription broadcast rx to the sender.")
        }
    }

    pub async fn run(mut self) {
        // TODO: consider recv_many instead
        let task_tracker = TaskTracker::new();
        while let Some(next) = self.inbox.recv().await {
            match next {
                NodeSubscriptionsActorMessage::NewInboundSubscription(msg) => {
                    let cloned_subscriptions = Arc::clone(&self.active_subscriptions);
                    let mut active_subscriptions_guard = self.active_subscriptions.lock().await;
                    warn!(?msg.request.kind, "outbound request received!");

                    let active_subscription = active_subscriptions_guard
                        .entry(msg.request.kind.clone())
                        .or_insert_with(|| Arc::new(OnceCell::new()));
                    let cloned_subscription_tx_cell = Arc::clone(&active_subscription);
                    drop(active_subscriptions_guard);

                    let cloned_tracker = task_tracker.clone();
                    let cloned_endpoints = self.websocket_endpoints.clone();

                    task_tracker.spawn(async move {
                        NodeSubscriptionsActor::respond_with_bcast_rx(
                            cloned_subscription_tx_cell,
                            cloned_endpoints,
                            msg,
                            cloned_tracker,
                            cloned_subscriptions,
                        )
                        .await;
                    });
                }
            }
        }
        warn!("Node subscriptions actor has stopped received new messages. Waiting until ongoing subscriptions terminate.");
        task_tracker.wait().await;
    }
}

#[derive(Clone)]
pub struct NodeSubscriptionsActorHandle {
    sender: tokio::sync::mpsc::Sender<NodeSubscriptionsActorMessage>,
}

impl Deref for NodeSubscriptionsActorHandle {
    type Target = tokio::sync::mpsc::Sender<NodeSubscriptionsActorMessage>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl DerefMut for NodeSubscriptionsActorHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sender
    }
}

pub fn actor(
    websocket_endpoints: Vec<WebSocketTargetEndpoint>, // TODO: should this also be a static reference?
) -> (NodeSubscriptionsActor, NodeSubscriptionsActorHandle) {
    let (tx, rx) = tokio::sync::mpsc::channel(100); // TODO: make this configurable
    let actor = NodeSubscriptionsActor {
        active_subscriptions: Arc::new(Mutex::new(HashMap::new())),
        inbox: rx,
        websocket_endpoints,
    };
    let handle = NodeSubscriptionsActorHandle { sender: tx };
    (actor, handle)
}
