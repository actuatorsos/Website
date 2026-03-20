//! WebSocket Live Updates Handler
//!
//! Broadcasts SurrealDB live query events to connected WebSocket clients.

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast;

/// Query params for selecting which updates to receive.
#[derive(Deserialize)]
pub struct LiveUpdateParams {
    /// Subscribe to asset updates.
    #[serde(default)]
    pub assets: bool,
    /// Subscribe to repair updates.
    #[serde(default)]
    pub repairs: bool,
    /// Subscribe to device updates.
    #[serde(default)]
    pub devices: bool,
    /// Subscribe to notification updates.
    #[serde(default)]
    pub notifications: bool,
}

/// State containing broadcast receivers.
#[derive(Clone)]
pub struct LiveUpdateState {
    /// Asset updates sender.
    pub asset_tx: broadcast::Sender<String>,
    /// Repair updates sender.
    pub repair_tx: broadcast::Sender<String>,
    /// Device updates sender.
    pub device_tx: broadcast::Sender<String>,
    /// Notification updates sender.
    pub notification_tx: broadcast::Sender<String>,
}

/// WebSocket handler for live updates.
///
/// GET /ws/live?assets=true&repairs=true&devices=false
pub async fn live_updates_handler(
    ws: WebSocketUpgrade,
    State(state): State<LiveUpdateState>,
    Query(params): Query<LiveUpdateParams>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, params))
}

async fn handle_socket(socket: WebSocket, state: LiveUpdateState, params: LiveUpdateParams) {
    let (mut sender, mut receiver) = socket.split();

    // Create receivers for selected subscriptions
    let mut asset_rx = if params.assets {
        Some(state.asset_tx.subscribe())
    } else {
        None
    };

    let mut repair_rx = if params.repairs {
        Some(state.repair_tx.subscribe())
    } else {
        None
    };

    let mut device_rx = if params.devices {
        Some(state.device_tx.subscribe())
    } else {
        None
    };

    let mut notification_rx = if params.notifications {
        Some(state.notification_tx.subscribe())
    } else {
        None
    };

    tracing::info!(
        "WebSocket client connected: assets={}, repairs={}, devices={}, notifications={}",
        params.assets,
        params.repairs,
        params.devices,
        params.notifications
    );

    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        tracing::debug!("Received from client: {}", text);
                        // Could handle client commands here (e.g., change subscriptions)
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("WebSocket client disconnected");
                        break;
                    }
                    _ => {}
                }
            }

            // Forward asset updates
            update = async {
                match &mut asset_rx {
                    Some(rx) => rx.recv().await.ok(),
                    None => std::future::pending().await,
                }
            } => {
                if let Some(json) = update {
                    let msg = format!(r#"{{"type":"asset","data":{}}}"#, json);
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }

            // Forward repair updates
            update = async {
                match &mut repair_rx {
                    Some(rx) => rx.recv().await.ok(),
                    None => std::future::pending().await,
                }
            } => {
                if let Some(json) = update {
                    let msg = format!(r#"{{"type":"repair","data":{}}}"#, json);
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }

            // Forward device updates
            update = async {
                match &mut device_rx {
                    Some(rx) => rx.recv().await.ok(),
                    None => std::future::pending().await,
                }
            } => {
                if let Some(json) = update {
                    let msg = format!(r#"{{"type":"device","data":{}}}"#, json);
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }

            // Forward notification updates
            update = async {
                match &mut notification_rx {
                    Some(rx) => rx.recv().await.ok(),
                    None => std::future::pending().await,
                }
            } => {
                if let Some(json) = update {
                    let msg = format!(r#"{{"type":"notification","data":{}}}"#, json);
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}
