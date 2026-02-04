use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tracing::{error, info, warn};
use websocket::{ClientMessage, ServerMessage};

/// WebSocket upgrade handler
///
/// This is a minimal WebSocket endpoint that accepts connections at `/ws`
/// and handles basic Subscribe and Ping messages.
///
/// ## Current Limitations
///
/// This implementation does NOT integrate with `ConnectionManager` due to a type
/// incompatibility:
/// - Axum provides `axum::extract::ws::WebSocket`
/// - ConnectionManager expects `tokio_tungstenite::WebSocketStream<TcpStream>`
///
/// These types are not directly compatible. Full integration requires:
/// 1. A type adapter to bridge Axum's WebSocket with tokio-tungstenite
/// 2. Session-based connection tracking
/// 3. Broadcasting to all clients in a session
///
/// ## What This Handler Does
///
/// - Accepts WebSocket upgrade requests at `/ws`
/// - Parses incoming JSON messages as `ClientMessage`
/// - Responds to:
///   - `Subscribe`: Returns `Subscribed` confirmation
///   - `Ping`: Returns `Pong`
///   - `MakePick`/`ProposeTrade`: Returns "not implemented" error with guidance to use REST API
/// - Logs all connections and messages for debugging
///
/// ## Future Work
///
/// To enable full bidirectional communication:
/// 1. Create an adapter that wraps Axum's WebSocket and implements the interface expected by ConnectionManager
/// 2. Register connections with ConnectionManager on Subscribe
/// 3. Listen for broadcast messages from ConnectionManager
/// 4. Unregister connections on disconnect
///
pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    info!("WebSocket connection established");

    let (mut sender, mut receiver) = socket.split();

    // Process incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received WebSocket message: {}", text);

                // Parse the client message
                match ClientMessage::from_json(&text) {
                    Ok(client_msg) => {
                        // Handle the message and generate a response
                        let response = handle_client_message(client_msg).await;

                        // Send the response
                        if let Ok(response_json) = response.to_json() {
                            if let Err(e) = sender.send(Message::Text(response_json.into())).await {
                                error!("Failed to send WebSocket message: {}", e);
                                break;
                            }
                        } else {
                            error!("Failed to serialize server message");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse client message: {}", e);
                        let error_msg =
                            ServerMessage::error(format!("Invalid message format: {}", e));
                        if let Ok(error_json) = error_msg.to_json() {
                            let _ = sender.send(Message::Text(error_json.into())).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket client disconnected");
                break;
            }
            Ok(Message::Ping(data)) => {
                // Respond to WebSocket protocol ping with pong
                if let Err(e) = sender.send(Message::Pong(data)).await {
                    error!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // Ignore pong messages
            }
            Ok(Message::Binary(_)) => {
                warn!("Received binary message (not supported)");
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket connection closed");
}

async fn handle_client_message(msg: ClientMessage) -> ServerMessage {
    match msg {
        ClientMessage::Subscribe { session_id } => {
            info!("Client subscribed to session: {}", session_id);
            // TODO: Register connection with ConnectionManager when type adapter is implemented
            ServerMessage::subscribed(session_id)
        }
        ClientMessage::Ping => {
            info!("Received ping");
            ServerMessage::pong()
        }
        ClientMessage::MakePick { .. } => {
            warn!("MakePick not implemented via WebSocket");
            ServerMessage::error(
                "MakePick is not yet implemented via WebSocket. Please use the REST API endpoint: POST /api/v1/sessions/:id/picks".to_string()
            )
        }
        ClientMessage::ProposeTrade { .. } => {
            warn!("ProposeTrade not implemented via WebSocket");
            ServerMessage::error(
                "ProposeTrade is not yet implemented via WebSocket. Please use the REST API endpoint: POST /api/v1/trades".to_string()
            )
        }
    }
}
