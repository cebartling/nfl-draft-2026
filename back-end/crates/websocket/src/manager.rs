use dashmap::DashMap;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::messages::ServerMessage;

/// Type alias for WebSocket sender
pub type WsSender = SplitSink<WebSocketStream<TcpStream>, Message>;

/// Represents a WebSocket connection
#[derive(Debug)]
pub struct Connection {
    pub id: Uuid,
    pub session_id: Uuid,
}

/// Manages WebSocket connections for draft sessions
#[derive(Clone)]
pub struct ConnectionManager {
    /// Maps connection ID to its sender
    connections: Arc<DashMap<Uuid, WsSender>>,
    /// Maps session ID to set of connection IDs
    sessions: Arc<DashMap<Uuid, Vec<Uuid>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        }
    }

    /// Add a new connection to a session
    pub fn add_connection(&self, connection_id: Uuid, session_id: Uuid, sender: WsSender) {
        info!(
            connection_id = %connection_id,
            session_id = %session_id,
            "Adding WebSocket connection"
        );

        // Store the sender
        self.connections.insert(connection_id, sender);

        // Add connection to session
        self.sessions
            .entry(session_id)
            .or_insert_with(Vec::new)
            .push(connection_id);

        debug!(
            session_id = %session_id,
            connection_count = self.sessions.get(&session_id).map(|s| s.len()).unwrap_or(0),
            "Connection added to session"
        );
    }

    /// Remove a connection
    pub fn remove_connection(&self, connection_id: Uuid) {
        info!(connection_id = %connection_id, "Removing WebSocket connection");

        // Remove from connections
        self.connections.remove(&connection_id);

        // Remove from all sessions
        self.sessions.iter_mut().for_each(|mut entry| {
            let session_id = *entry.key();
            entry.value_mut().retain(|id| *id != connection_id);

            if entry.value().is_empty() {
                debug!(session_id = %session_id, "Session has no more connections");
            }
        });

        // Clean up empty sessions
        self.sessions
            .retain(|_, connections| !connections.is_empty());
    }

    /// Broadcast a message to all connections in a session
    pub async fn broadcast_to_session(&self, session_id: Uuid, message: ServerMessage) {
        let json = match message.to_json() {
            Ok(json) => json,
            Err(e) => {
                error!(error = %e, "Failed to serialize server message");
                return;
            }
        };

        let connection_ids = match self.sessions.get(&session_id) {
            Some(ids) => ids.clone(),
            None => {
                warn!(session_id = %session_id, "No connections for session");
                return;
            }
        };

        debug!(
            session_id = %session_id,
            connection_count = connection_ids.len(),
            message_type = ?message,
            "Broadcasting message to session"
        );

        let mut failed_connections = Vec::new();

        for connection_id in &connection_ids {
            if let Some(mut sender) = self.connections.get_mut(connection_id) {
                if let Err(e) = sender.send(Message::Text(json.clone())).await {
                    error!(
                        connection_id = %connection_id,
                        error = %e,
                        "Failed to send message to connection"
                    );
                    failed_connections.push(*connection_id);
                }
            } else {
                warn!(
                    connection_id = %connection_id,
                    "Connection not found in manager"
                );
                failed_connections.push(*connection_id);
            }
        }

        // Remove failed connections
        for connection_id in failed_connections {
            self.remove_connection(connection_id);
        }
    }

    /// Send a message to a specific connection
    pub async fn send_to_connection(&self, connection_id: Uuid, message: ServerMessage) {
        let json = match message.to_json() {
            Ok(json) => json,
            Err(e) => {
                error!(error = %e, "Failed to serialize server message");
                return;
            }
        };

        if let Some(mut sender) = self.connections.get_mut(&connection_id) {
            if let Err(e) = sender.send(Message::Text(json)).await {
                error!(
                    connection_id = %connection_id,
                    error = %e,
                    "Failed to send message to connection"
                );
                self.remove_connection(connection_id);
            }
        } else {
            warn!(
                connection_id = %connection_id,
                "Connection not found in manager"
            );
        }
    }

    /// Get the number of connections in a session
    pub fn session_connection_count(&self, session_id: Uuid) -> usize {
        self.sessions.get(&session_id).map(|s| s.len()).unwrap_or(0)
    }

    /// Get total number of active connections
    pub fn total_connections(&self) -> usize {
        self.connections.len()
    }

    /// Get total number of active sessions
    pub fn total_sessions(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are limited unit tests since we can't easily create WsSender in tests.
    // Full integration tests will be in the acceptance tests.

    #[test]
    fn test_new_manager() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.total_connections(), 0);
        assert_eq!(manager.total_sessions(), 0);
    }

    #[test]
    fn test_session_connection_count_empty() {
        let manager = ConnectionManager::new();
        let session_id = Uuid::new_v4();
        assert_eq!(manager.session_connection_count(session_id), 0);
    }
}
