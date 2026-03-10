use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tracing::{error, info, warn};
use uuid::Uuid;
use websocket::{ClientMessage, ServerMessage};

use crate::state::AppState;

/// WebSocket upgrade handler
///
/// Accepts WebSocket connections at `/ws`, registers them with the ConnectionManager
/// on Subscribe, and multiplexes inbound client messages with outbound server-push
/// messages via an mpsc channel.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let connection_id = Uuid::new_v4();
    info!(connection_id = %connection_id, "WebSocket connection established");

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Channel for server-push messages (ConnectionManager → this handler → WS client)
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let mut subscribed_session_id: Option<Uuid> = None;

    loop {
        tokio::select! {
            // Outbound: forward server-push messages to the WS client
            Some(msg) = rx.recv() => {
                if let Err(e) = ws_sender.send(Message::Text(msg.into())).await {
                    error!(connection_id = %connection_id, error = %e, "Failed to forward server message to WS client");
                    break;
                }
            }
            // Inbound: handle messages from the WS client
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match ClientMessage::from_json(&text) {
                            Ok(client_msg) => {
                                match client_msg {
                                    ClientMessage::Subscribe { session_id } => {
                                        info!(connection_id = %connection_id, session_id = %session_id, "Client subscribing to session");

                                        // Register with ConnectionManager
                                        state.ws_manager.add_connection(connection_id, session_id, tx.clone());
                                        subscribed_session_id = Some(session_id);

                                        // Send Subscribed confirmation directly
                                        let response = ServerMessage::subscribed(session_id);
                                        if let Ok(json) = response.to_json() {
                                            if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                                                error!(connection_id = %connection_id, error = %e, "Failed to send Subscribed response");
                                                break;
                                            }
                                        }
                                    }
                                    ClientMessage::Ping => {
                                        let response = ServerMessage::pong();
                                        if let Ok(json) = response.to_json() {
                                            if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                                                error!(connection_id = %connection_id, error = %e, "Failed to send Pong");
                                                break;
                                            }
                                        }
                                    }
                                    ClientMessage::MakePick { .. } => {
                                        warn!(connection_id = %connection_id, "MakePick not implemented via WebSocket");
                                        let response = ServerMessage::error(
                                            "MakePick is not yet implemented via WebSocket. Please use the REST API endpoint: POST /api/v1/sessions/:id/picks".to_string()
                                        );
                                        if let Ok(json) = response.to_json() {
                                            let _ = ws_sender.send(Message::Text(json.into())).await;
                                        }
                                    }
                                    ClientMessage::ProposeTrade { .. } => {
                                        warn!(connection_id = %connection_id, "ProposeTrade not implemented via WebSocket");
                                        let response = ServerMessage::error(
                                            "ProposeTrade is not yet implemented via WebSocket. Please use the REST API endpoint: POST /api/v1/trades".to_string()
                                        );
                                        if let Ok(json) = response.to_json() {
                                            let _ = ws_sender.send(Message::Text(json.into())).await;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(connection_id = %connection_id, error = %e, "Failed to parse client message");
                                let error_msg = ServerMessage::error(format!("Invalid message format: {}", e));
                                if let Ok(json) = error_msg.to_json() {
                                    let _ = ws_sender.send(Message::Text(json.into())).await;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!(connection_id = %connection_id, "WebSocket client disconnected");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                            error!(connection_id = %connection_id, error = %e, "Failed to send pong");
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Binary(_))) => {
                        warn!(connection_id = %connection_id, "Received binary message (not supported)");
                    }
                    Some(Err(e)) => {
                        error!(connection_id = %connection_id, error = %e, "WebSocket error");
                        break;
                    }
                    None => {
                        // Stream ended
                        break;
                    }
                }
            }
        }
    }

    // Clean up connection on disconnect
    if subscribed_session_id.is_some() {
        state.ws_manager.remove_connection(connection_id);
    }
    info!(connection_id = %connection_id, "WebSocket connection closed");
}
