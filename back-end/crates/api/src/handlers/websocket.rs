use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;
use websocket::{ClientMessage, ServerMessage};

use crate::state::AppState;

/// WebSocket upgrade handler
///
/// Accepts WebSocket connections at `/ws` and bridges them to the
/// `ConnectionManager` via an mpsc channel. When a client sends a
/// `Subscribe` message, the connection is registered with the
/// ConnectionManager so that `broadcast_to_session()` messages
/// reach this client.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let conn_id = Uuid::new_v4();
    info!(connection_id = %conn_id, "WebSocket connection established");

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create an mpsc channel that the ConnectionManager will send to.
    // A forwarding task reads from the channel and writes to the Axum WS sender.
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_sender.send(Message::Text(msg.into())).await {
                error!(connection_id = %conn_id, error = %e, "Forward task: failed to send to WS");
                break;
            }
        }
    });

    // Process incoming messages from the client
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!(connection_id = %conn_id, "Received WebSocket message: {}", text);

                match ClientMessage::from_json(&text) {
                    Ok(client_msg) => {
                        handle_client_message(conn_id, client_msg, &state, &tx).await;
                    }
                    Err(e) => {
                        warn!(connection_id = %conn_id, "Failed to parse client message: {}", e);
                        let error_msg =
                            ServerMessage::error(format!("Invalid message format: {}", e));
                        if let Ok(json) = error_msg.to_json() {
                            let _ = tx.send(json);
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!(connection_id = %conn_id, "WebSocket client disconnected");
                break;
            }
            Ok(Message::Ping(data)) => {
                // Send pong via the channel so it goes through the forwarding task
                if let Ok(json) = ServerMessage::pong().to_json() {
                    let _ = tx.send(json);
                }
                // Also echo back at protocol level — but we no longer hold ws_sender directly.
                // The Axum WS layer handles protocol-level pong automatically.
                let _ = data; // consume
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Binary(_)) => {
                warn!(connection_id = %conn_id, "Received binary message (not supported)");
            }
            Err(e) => {
                error!(connection_id = %conn_id, "WebSocket error: {}", e);
                break;
            }
        }
    }

    // Cleanup: unregister from ConnectionManager and abort forwarding task
    state.ws_manager.remove_connection(conn_id);
    forward_task.abort();
    info!(connection_id = %conn_id, "WebSocket connection closed");
}

async fn handle_client_message(
    conn_id: Uuid,
    msg: ClientMessage,
    state: &AppState,
    tx: &mpsc::UnboundedSender<String>,
) {
    match msg {
        ClientMessage::Subscribe { session_id } => {
            info!(connection_id = %conn_id, session_id = %session_id, "Client subscribed to session");

            // Register this connection with the ConnectionManager
            state
                .ws_manager
                .add_connection(conn_id, session_id, tx.clone());

            // Send confirmation back through the channel
            let response = ServerMessage::subscribed(session_id);
            if let Ok(json) = response.to_json() {
                let _ = tx.send(json);
            }
        }
        ClientMessage::Ping => {
            info!(connection_id = %conn_id, "Received ping");
            let response = ServerMessage::pong();
            if let Ok(json) = response.to_json() {
                let _ = tx.send(json);
            }
        }
        ClientMessage::MakePick { .. } => {
            warn!(connection_id = %conn_id, "MakePick not implemented via WebSocket");
            let response = ServerMessage::error(
                "MakePick is not yet implemented via WebSocket. Please use the REST API endpoint: POST /api/v1/sessions/:id/picks".to_string(),
            );
            if let Ok(json) = response.to_json() {
                let _ = tx.send(json);
            }
        }
        ClientMessage::ProposeTrade { .. } => {
            warn!(connection_id = %conn_id, "ProposeTrade not implemented via WebSocket");
            let response = ServerMessage::error(
                "ProposeTrade is not yet implemented via WebSocket. Please use the REST API endpoint: POST /api/v1/trades".to_string(),
            );
            if let Ok(json) = response.to_json() {
                let _ = tx.send(json);
            }
        }
    }
}
