use futures::SinkExt;
use futures_util::StreamExt;
use rpc_gateway::tracing::init_tracing;
use std::{
    collections::HashMap, fmt::Debug, num::ParseIntError, ops::Deref, sync::Arc, time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, warn};

#[derive(PartialEq, Eq, Hash, Clone)]
struct SubscriptionId(u128);

impl Deref for SubscriptionId {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.debug_tuple("SubscriptionId").field(&self.0).finish()
        f.pad(&format!("{:#034X?}", self.0))
    }
}

impl From<SubscriptionId> for String {
    fn from(value: SubscriptionId) -> Self {
        format!("{:#034X?}", value) // TODO: is this good?
    }
}

impl TryFrom<&str> for SubscriptionId {
    type Error = ParseIntError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let hex_str = value.strip_prefix("0x").unwrap_or(value);
        let id = u128::from_str_radix(hex_str, 16)?;
        Ok(Self(id))
    }
}

impl SubscriptionId {
    pub fn rand() -> Self {
        Self(fastrand::u128(0..u128::MAX))
    }
}

struct InboundSubscriptionMessage {
    inbound_id: SubscriptionId,
    outbound_id: i32,
    iteration: i32,
    kind: SubscriptionKind,
}

impl Debug for InboundSubscriptionMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inbound_id = self.inbound_id.clone();
        let inbound_id: String = inbound_id.into();

        f.debug_struct("InboundSubscriptionMessage")
            .field("inbound_id", &inbound_id)
            .field("outbound_id", &self.outbound_id)
            .field("iteration", &self.iteration)
            .field("kind", &self.kind)
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum SubscriptionKind {
    NewHeads,
    PendingTransactions,
}

struct WebSocketContext {
    inbound_subscriptions: HashMap<SubscriptionId, CancellationToken>,
    inbound_subscriptions_tx_sender:
        tokio::sync::mpsc::UnboundedSender<InboundSubscriptionSenderSelfPayload>,
}

type InboundSubscriptionReceiver = tokio::sync::mpsc::Receiver<InboundSubscriptionMessage>;

struct InboundSubscriptionReceiverSelfPayload {
    rx: InboundSubscriptionReceiver,
    cancellation_token: CancellationToken,
}

struct InboundSubscriptionSenderSelfPayload {
    tx: InboundSubscriptionSender,
    kind: SubscriptionKind,
    id: SubscriptionId,
}

type InboundSubscriptionSender = tokio::sync::mpsc::Sender<InboundSubscriptionMessage>;

impl WebSocketContext {
    fn get_unique_subscription_id(&self) -> SubscriptionId {
        loop {
            let candidate = SubscriptionId::rand();
            match self.inbound_subscriptions.contains_key(&candidate) {
                true => continue,
                false => break candidate,
            }
        }
    }

    pub fn unsubscribe(&mut self, id: &SubscriptionId) -> bool {
        match self.inbound_subscriptions.remove(id) {
            Some(token) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    pub fn subscribe(&mut self, kind: SubscriptionKind) -> InboundSubscriptionReceiverSelfPayload {
        // TODO: this should be a Result object
        let (tx, rx) = tokio::sync::mpsc::channel::<InboundSubscriptionMessage>(10); // TODO: this should come from the outbound factory

        let id = self.get_unique_subscription_id();
        self.inbound_subscriptions_tx_sender
            .send(InboundSubscriptionSenderSelfPayload {
                kind,
                tx,
                id: id.clone(),
            })
            .unwrap();

        let cancellation_token = CancellationToken::new();

        self.inbound_subscriptions
            .insert(id, cancellation_token.clone());
        InboundSubscriptionReceiverSelfPayload {
            rx,
            cancellation_token: cancellation_token,
        }
    }
}

async fn accept_connection(
    stream: TcpStream,
    tx: tokio::sync::mpsc::UnboundedSender<InboundSubscriptionSenderSelfPayload>,
) {
    let peer = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", peer);
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (sink, mut stream) = ws_stream.split();

    let sink = Arc::new(tokio::sync::Mutex::new(sink));

    let mut ws_context = WebSocketContext {
        inbound_subscriptions: HashMap::new(),
        inbound_subscriptions_tx_sender: tx,
    };

    let (sub_channels_tx, mut sub_channels_rx) =
        tokio::sync::mpsc::unbounded_channel::<InboundSubscriptionReceiverSelfPayload>();

    let inbound_subs_tracker = TaskTracker::new();
    let inbound_subscriptions_loop = async {
        // TODO: handle the "never initialized" case here.
        while let Some(inbound_rx_channel_payload) = sub_channels_rx.recv().await {
            let sink = Arc::clone(&sink);

            inbound_subs_tracker.spawn(async move {
                let mut rx = inbound_rx_channel_payload.rx;
                let cancellation_token = inbound_rx_channel_payload.cancellation_token;
                let inner_loop = async {
                    while let Some(inbound) = rx.recv().await {
                        let mut lock = sink.lock().await;
                        lock.send(format!("Received: {inbound:?}").into())
                            .await
                            .unwrap();
                        drop(lock);
                    }
                    info!("Done with subscription");
                };
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        info!("Subscription cancelled.");
                    }
                    _ = inner_loop => {
                        error!("Inbound inner loop stopped unexpectedly");
                    }
                }
            });
        }
    };

    let ws_inbound_msg_loop = async {
        while let Some(msg) = stream.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(err) => {
                    error!(?err);
                    break;
                }
            };
            match msg {
                Message::Text(m) => {
                    let m = m.trim();
                    if m.eq("new_heads") {
                        info!("new heads");
                        // TODO: if the same client subscribes to the same topic twice, send 2 messages on one channel instead of sending one message on 2 different channels.
                        let rx = ws_context.subscribe(SubscriptionKind::NewHeads);
                        sub_channels_tx.send(rx).unwrap();
                    } else if m.eq("pending_transactions") {
                        info!("pending tx");
                        let rx = ws_context.subscribe(SubscriptionKind::PendingTransactions);
                        sub_channels_tx.send(rx).unwrap();
                    } else if m.contains("unsub") {
                        info!("unsub received");
                        let parts = m.split(" ").nth(1);
                        if let Some(id) = parts {
                            info!("can split: {id:?}");
                            if let Ok(id) = SubscriptionId::try_from(id) {
                                info!(?id, "unsub attempt");
                                let mut lock = sink.lock().await;
                                if ws_context.unsubscribe(&id) {
                                    lock.send(Message::Text(format!(
                                        "Successfully removed: {:?}",
                                        id
                                    )))
                                    .await
                                    .unwrap();
                                } else {
                                    lock.send(Message::Text(format!("Not Found: {:?}", id)))
                                        .await
                                        .unwrap();
                                }
                                drop(lock);
                            } else {
                                error!("failed to parse subscription id");
                            }
                        } else {
                            error!("cant split");
                        }
                    } else {
                        warn!(?m, "unknown message received");
                    }
                }
                Message::Binary(_) => todo!(),
                Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => {
                    info!("hi");
                }
            }
        }
    };

    tokio::select! {
        _ = inbound_subscriptions_loop => {
            error!("subscriptions closed unexpectedly")
        },
        _ = ws_inbound_msg_loop => {
            error!("ws inbound closed unexpectedly")
        }
    }
}

async fn listen_ws_connections(
    listener: TcpListener,
    tx: tokio::sync::mpsc::UnboundedSender<InboundSubscriptionSenderSelfPayload>,
) {
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, tx.clone()));
    }
}

#[derive(Debug)]
struct OutboundSubscriptionMessage {
    outbound_id: i32,
    iteration: i32,
}

// TODO: do we have to use RC even on the broadcast channel? is there a way to make this a single-producer multi-consumer?
// and place the writing end on a fixed thread?
type OutboundSubscriptionSender = tokio::sync::broadcast::Sender<Arc<OutboundSubscriptionMessage>>;

struct OutboundSubscriptions {
    outbounds: HashMap<SubscriptionKind, OutboundSubscriptionSender>,
}

impl OutboundSubscriptions {
    // TODO: this should handle the pull loop too
    pub fn add(&mut self, kind: SubscriptionKind) -> (OutboundSubscriptionSender, bool) {
        match self.outbounds.get(&kind) {
            Some(sender) => (sender.clone(), false),
            None => {
                // TODO: broadcast channels require the message to implement clone. will this be super expensive unless we use rc?
                let (tx, _) =
                    tokio::sync::broadcast::channel::<Arc<OutboundSubscriptionMessage>>(10);
                let sender_clone = tx.clone();
                self.outbounds.insert(kind, sender_clone);
                (tx, true)
            }
        }
    }

    pub fn remove(&mut self, kind: SubscriptionKind) -> bool {
        match self.outbounds.remove(&kind) {
            Some(_) => true,
            None => false,
        }
    }
}

struct OutboundChannelSelfPayload {
    tx: OutboundSubscriptionSender,
    kind: SubscriptionKind,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    let (tx, mut rx) =
        tokio::sync::mpsc::unbounded_channel::<InboundSubscriptionSenderSelfPayload>();
    let inbound_subs_task_tracker = TaskTracker::new();
    let outbound_subs_task_tracker = TaskTracker::new();

    let (outbound_bcast_channels_tx, mut outbound_channels_rx) =
        tokio::sync::mpsc::unbounded_channel::<OutboundChannelSelfPayload>();

    let outbound_subscriptions = Arc::new(Mutex::new(OutboundSubscriptions {
        outbounds: HashMap::new(),
    }));

    let add_outbound_subscriptions_loop = async {
        let mut outbound_id = 0;

        while let Some(bcast_payload) = outbound_channels_rx.recv().await {
            let outbound_subscriptions = Arc::clone(&outbound_subscriptions);
            outbound_id += 1;
            outbound_subs_task_tracker.spawn(async move {
                for i in 1..=10_000 {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    match bcast_payload.tx.send(Arc::new(OutboundSubscriptionMessage {
                        outbound_id: outbound_id, // TODO: make this dynamic,
                        iteration: i,
                    })) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Could not send broadcast. Cancelling outbound.");
                            let mut lock = outbound_subscriptions.lock().await;
                            lock.remove(bcast_payload.kind);
                            drop(lock);
                            break;
                        }
                    }
                }
            });
        }
    };

    let add_inbound_subscriptions_loop = async {
        // TODO: add this into the graceful shutdown too.
        let outbound_bcast_channels_tx = Arc::new(Mutex::new(outbound_bcast_channels_tx));

        while let Some(inbound_tx) = rx.recv().await {
            let outbound_subscriptions = Arc::clone(&outbound_subscriptions); // TODO: this could be a &static
            let outbound_bcast_channels_tx = Arc::clone(&outbound_bcast_channels_tx);
            inbound_subs_task_tracker.spawn(async move {
                let mut lock = outbound_subscriptions.lock().await;
                let (outbound_bcast_tx, is_new) = lock.add(inbound_tx.kind.clone()); // TODO: who will drive the outbound?
                drop(lock);

                if is_new {
                    let lock = outbound_bcast_channels_tx.lock().await;
                    lock.send(OutboundChannelSelfPayload {
                        tx: outbound_bcast_tx.clone(),
                        kind: inbound_tx.kind.clone(),
                    })
                    .unwrap();
                    drop(lock);
                }

                let mut outbound_bcast_rx = outbound_bcast_tx.subscribe();

                while let Ok(msg) = outbound_bcast_rx.recv().await {
                    inbound_tx
                        .tx
                        .send(InboundSubscriptionMessage {
                            inbound_id: inbound_tx.id.clone(),
                            iteration: msg.iteration,
                            outbound_id: msg.outbound_id,
                            kind: inbound_tx.kind.clone(),
                        })
                        .await
                        .unwrap();
                }
            });
        }
    };

    let listen_handler = tokio::spawn(listen_ws_connections(listener, tx));

    tokio::select! {
        _ = add_inbound_subscriptions_loop => {
            error!("add inbound subs loop ended")
        },
        _ = add_outbound_subscriptions_loop => {
            error!("add outbound subs loop ended")
        },
        _ = inbound_subs_task_tracker.wait() => {
            error!("subs task tracker ended")
        },
        _ = outbound_subs_task_tracker.wait() => {
            error!("outbound subs tracker ended")
        },
        _ = listen_handler => {
            error!("listen handler ended");
            inbound_subs_task_tracker.close(); // TODO: is this the correct place for this? maybe this should be done only when a cancellation token is called.
        },
    }
}
