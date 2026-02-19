use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::messages::ServerMessage;

/// Type alias for WebSocket sender — an unbounded channel sender carrying JSON strings.
/// The WS handler bridges this to the actual Axum WebSocket sink.
pub type WsSender = mpsc::UnboundedSender<String>;

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
            .or_default()
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
            "Broadcasting message to session"
        );

        let mut failed_connections = Vec::new();

        for connection_id in &connection_ids {
            if let Some(sender) = self.connections.get(connection_id) {
                if let Err(e) = sender.send(json.clone()) {
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

        let failed = if let Some(sender) = self.connections.get(&connection_id) {
            if let Err(e) = sender.send(json) {
                error!(
                    connection_id = %connection_id,
                    error = %e,
                    "Failed to send message to connection"
                );
                true
            } else {
                false
            }
        } else {
            warn!(
                connection_id = %connection_id,
                "Connection not found in manager"
            );
            false
        };

        // Remove outside the DashMap borrow to avoid deadlock
        if failed {
            self.remove_connection(connection_id);
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

    #[test]
    fn test_add_connection() {
        let manager = ConnectionManager::new();
        let conn_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let (tx, _rx) = mpsc::unbounded_channel::<String>();

        manager.add_connection(conn_id, session_id, tx);

        assert_eq!(manager.total_connections(), 1);
        assert_eq!(manager.total_sessions(), 1);
        assert_eq!(manager.session_connection_count(session_id), 1);
    }

    #[test]
    fn test_remove_connection() {
        let manager = ConnectionManager::new();
        let conn_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let (tx, _rx) = mpsc::unbounded_channel::<String>();

        manager.add_connection(conn_id, session_id, tx);
        manager.remove_connection(conn_id);

        assert_eq!(manager.total_connections(), 0);
        assert_eq!(manager.total_sessions(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_to_session_delivers_json() {
        let manager = ConnectionManager::new();
        let conn_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();

        manager.add_connection(conn_id, session_id, tx);

        let msg = ServerMessage::pong();
        manager.broadcast_to_session(session_id, msg).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received, r#"{"type":"pong"}"#);
    }

    #[tokio::test]
    async fn test_broadcast_to_multiple_connections() {
        let manager = ConnectionManager::new();
        let session_id = Uuid::new_v4();

        let (tx1, mut rx1) = mpsc::unbounded_channel::<String>();
        let (tx2, mut rx2) = mpsc::unbounded_channel::<String>();
        let conn1 = Uuid::new_v4();
        let conn2 = Uuid::new_v4();

        manager.add_connection(conn1, session_id, tx1);
        manager.add_connection(conn2, session_id, tx2);

        let msg = ServerMessage::pong();
        manager.broadcast_to_session(session_id, msg).await;

        let r1 = rx1.recv().await.unwrap();
        let r2 = rx2.recv().await.unwrap();
        assert_eq!(r1, r#"{"type":"pong"}"#);
        assert_eq!(r2, r#"{"type":"pong"}"#);
    }

    #[tokio::test]
    async fn test_send_to_connection_delivers_json() {
        let manager = ConnectionManager::new();
        let conn_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();

        manager.add_connection(conn_id, session_id, tx);

        let msg = ServerMessage::pong();
        manager.send_to_connection(conn_id, msg).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received, r#"{"type":"pong"}"#);
    }

    #[tokio::test]
    async fn test_broadcast_removes_closed_channel() {
        let manager = ConnectionManager::new();
        let session_id = Uuid::new_v4();

        let (tx1, rx1) = mpsc::unbounded_channel::<String>();
        let (tx2, mut rx2) = mpsc::unbounded_channel::<String>();
        let conn1 = Uuid::new_v4();
        let conn2 = Uuid::new_v4();

        manager.add_connection(conn1, session_id, tx1);
        manager.add_connection(conn2, session_id, tx2);

        // Drop receiver for conn1 — sending to tx1 will fail
        drop(rx1);

        let msg = ServerMessage::pong();
        manager.broadcast_to_session(session_id, msg).await;

        // conn1 should have been cleaned up
        assert_eq!(manager.total_connections(), 1);
        assert_eq!(manager.session_connection_count(session_id), 1);

        // conn2 should still receive
        let received = rx2.recv().await.unwrap();
        assert_eq!(received, r#"{"type":"pong"}"#);
    }

    #[tokio::test]
    async fn test_send_to_connection_removes_closed_channel() {
        let manager = ConnectionManager::new();
        let conn_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let (tx, rx) = mpsc::unbounded_channel::<String>();

        manager.add_connection(conn_id, session_id, tx);

        // Drop receiver — sending will fail
        drop(rx);

        let msg = ServerMessage::pong();
        manager.send_to_connection(conn_id, msg).await;

        // Connection should have been cleaned up
        assert_eq!(manager.total_connections(), 0);
        assert_eq!(manager.total_sessions(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_no_connections() {
        let manager = ConnectionManager::new();
        let session_id = Uuid::new_v4();

        // Should not panic
        let msg = ServerMessage::pong();
        manager.broadcast_to_session(session_id, msg).await;
    }
}
