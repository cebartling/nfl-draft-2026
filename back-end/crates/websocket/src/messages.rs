use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages sent from client to server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to a draft session
    Subscribe { session_id: Uuid },
    /// Make a draft pick
    MakePick { session_id: Uuid, player_id: Uuid },
    /// Propose a trade
    ProposeTrade {
        session_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
        pick_ids: Vec<Uuid>,
    },
    /// Ping to keep connection alive
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Confirmation of successful subscription
    Subscribed { session_id: Uuid },
    /// A pick was made
    PickMade {
        session_id: Uuid,
        pick_id: Uuid,
        team_id: Uuid,
        player_id: Uuid,
        round: i32,
        pick_number: i32,
        player_name: String,
        team_name: String,
    },
    /// Clock update (time remaining for current pick)
    ClockUpdate {
        session_id: Uuid,
        time_remaining: i32,
        current_pick_number: i32,
    },
    /// Draft status changed
    DraftStatus { session_id: Uuid, status: String },
    /// Trade was proposed
    TradeProposed {
        session_id: Uuid,
        trade_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
        from_team_name: String,
        to_team_name: String,
        from_team_picks: Vec<Uuid>,
        to_team_picks: Vec<Uuid>,
        from_team_value: i32,
        to_team_value: i32,
    },
    /// Trade was executed (accepted)
    TradeExecuted {
        session_id: Uuid,
        trade_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
    },
    /// Trade was rejected
    TradeRejected {
        session_id: Uuid,
        trade_id: Uuid,
        rejecting_team_id: Uuid,
    },
    /// Error occurred
    Error { message: String },
    /// Pong response to ping
    Pong,
}

impl ClientMessage {
    pub fn subscribe(session_id: Uuid) -> Self {
        ClientMessage::Subscribe { session_id }
    }

    pub fn make_pick(session_id: Uuid, player_id: Uuid) -> Self {
        ClientMessage::MakePick {
            session_id,
            player_id,
        }
    }

    pub fn propose_trade(
        session_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
        pick_ids: Vec<Uuid>,
    ) -> Self {
        ClientMessage::ProposeTrade {
            session_id,
            from_team_id,
            to_team_id,
            pick_ids,
        }
    }

    pub fn ping() -> Self {
        ClientMessage::Ping
    }

    /// Parse a JSON string into a ClientMessage
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl ServerMessage {
    pub fn subscribed(session_id: Uuid) -> Self {
        ServerMessage::Subscribed { session_id }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pick_made(
        session_id: Uuid,
        pick_id: Uuid,
        team_id: Uuid,
        player_id: Uuid,
        round: i32,
        pick_number: i32,
        player_name: String,
        team_name: String,
    ) -> Self {
        ServerMessage::PickMade {
            session_id,
            pick_id,
            team_id,
            player_id,
            round,
            pick_number,
            player_name,
            team_name,
        }
    }

    pub fn clock_update(session_id: Uuid, time_remaining: i32, current_pick_number: i32) -> Self {
        ServerMessage::ClockUpdate {
            session_id,
            time_remaining,
            current_pick_number,
        }
    }

    pub fn draft_status(session_id: Uuid, status: String) -> Self {
        ServerMessage::DraftStatus { session_id, status }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn trade_proposed(
        session_id: Uuid,
        trade_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
        from_team_name: String,
        to_team_name: String,
        from_team_picks: Vec<Uuid>,
        to_team_picks: Vec<Uuid>,
        from_team_value: i32,
        to_team_value: i32,
    ) -> Self {
        ServerMessage::TradeProposed {
            session_id,
            trade_id,
            from_team_id,
            to_team_id,
            from_team_name,
            to_team_name,
            from_team_picks,
            to_team_picks,
            from_team_value,
            to_team_value,
        }
    }

    pub fn trade_executed(
        session_id: Uuid,
        trade_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
    ) -> Self {
        ServerMessage::TradeExecuted {
            session_id,
            trade_id,
            from_team_id,
            to_team_id,
        }
    }

    pub fn trade_rejected(session_id: Uuid, trade_id: Uuid, rejecting_team_id: Uuid) -> Self {
        ServerMessage::TradeRejected {
            session_id,
            trade_id,
            rejecting_team_id,
        }
    }

    pub fn error(message: String) -> Self {
        ServerMessage::Error { message }
    }

    pub fn pong() -> Self {
        ServerMessage::Pong
    }

    /// Parse a JSON string into a ServerMessage
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_subscribe_serialization() {
        let session_id = Uuid::new_v4();
        let msg = ClientMessage::subscribe(session_id);

        let json = msg.to_json().unwrap();
        let parsed = ClientMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
        assert!(json.contains("\"type\":\"subscribe\""));
    }

    #[test]
    fn test_client_message_make_pick_serialization() {
        let session_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let msg = ClientMessage::make_pick(session_id, player_id);

        let json = msg.to_json().unwrap();
        let parsed = ClientMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
        assert!(json.contains("\"type\":\"make_pick\""));
    }

    #[test]
    fn test_client_message_ping_serialization() {
        let msg = ClientMessage::ping();

        let json = msg.to_json().unwrap();
        let parsed = ClientMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
        assert_eq!(json, "{\"type\":\"ping\"}");
    }

    #[test]
    fn test_server_message_subscribed_serialization() {
        let session_id = Uuid::new_v4();
        let msg = ServerMessage::subscribed(session_id);

        let json = msg.to_json().unwrap();
        let parsed = ServerMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
        assert!(json.contains("\"type\":\"subscribed\""));
    }

    #[test]
    fn test_server_message_pick_made_serialization() {
        let session_id = Uuid::new_v4();
        let pick_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();

        let msg = ServerMessage::pick_made(
            session_id,
            pick_id,
            team_id,
            player_id,
            1,
            1,
            "John Doe".to_string(),
            "Team A".to_string(),
        );

        let json = msg.to_json().unwrap();
        let parsed = ServerMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
        assert!(json.contains("\"type\":\"pick_made\""));
        assert!(json.contains("John Doe"));
    }

    #[test]
    fn test_server_message_clock_update_serialization() {
        let session_id = Uuid::new_v4();
        let msg = ServerMessage::clock_update(session_id, 120, 5);

        let json = msg.to_json().unwrap();
        let parsed = ServerMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
        assert!(json.contains("\"type\":\"clock_update\""));
        assert!(json.contains("\"time_remaining\":120"));
    }

    #[test]
    fn test_server_message_error_serialization() {
        let msg = ServerMessage::error("Something went wrong".to_string());

        let json = msg.to_json().unwrap();
        let parsed = ServerMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("Something went wrong"));
    }

    #[test]
    fn test_server_message_pong_serialization() {
        let msg = ServerMessage::pong();

        let json = msg.to_json().unwrap();
        let parsed = ServerMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
        assert_eq!(json, "{\"type\":\"pong\"}");
    }

    #[test]
    fn test_invalid_json_parsing() {
        let invalid_json = "{\"invalid\": \"message\"}";
        let result = ClientMessage::from_json(invalid_json);
        assert!(result.is_err());
    }
}
