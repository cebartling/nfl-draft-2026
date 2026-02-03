use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    SessionCreated,
    SessionStarted,
    SessionPaused,
    SessionResumed,
    SessionCompleted,
    PickMade,
    ClockUpdate,
    TradeProposed,
    TradeExecuted,
    TradeRejected,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::SessionCreated => write!(f, "SessionCreated"),
            EventType::SessionStarted => write!(f, "SessionStarted"),
            EventType::SessionPaused => write!(f, "SessionPaused"),
            EventType::SessionResumed => write!(f, "SessionResumed"),
            EventType::SessionCompleted => write!(f, "SessionCompleted"),
            EventType::PickMade => write!(f, "PickMade"),
            EventType::ClockUpdate => write!(f, "ClockUpdate"),
            EventType::TradeProposed => write!(f, "TradeProposed"),
            EventType::TradeExecuted => write!(f, "TradeExecuted"),
            EventType::TradeRejected => write!(f, "TradeRejected"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SessionCreated" => Ok(EventType::SessionCreated),
            "SessionStarted" => Ok(EventType::SessionStarted),
            "SessionPaused" => Ok(EventType::SessionPaused),
            "SessionResumed" => Ok(EventType::SessionResumed),
            "SessionCompleted" => Ok(EventType::SessionCompleted),
            "PickMade" => Ok(EventType::PickMade),
            "ClockUpdate" => Ok(EventType::ClockUpdate),
            "TradeProposed" => Ok(EventType::TradeProposed),
            "TradeExecuted" => Ok(EventType::TradeExecuted),
            "TradeRejected" => Ok(EventType::TradeRejected),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid event type: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftEvent {
    pub id: Uuid,
    pub session_id: Uuid,
    pub event_type: EventType,
    pub event_data: JsonValue,
    pub created_at: DateTime<Utc>,
}

impl DraftEvent {
    pub fn new(session_id: Uuid, event_type: EventType, event_data: JsonValue) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            event_type,
            event_data,
            created_at: Utc::now(),
        }
    }

    pub fn session_created(session_id: Uuid, draft_id: Uuid, settings: JsonValue) -> Self {
        let data = serde_json::json!({
            "draft_id": draft_id,
            "settings": settings,
        });
        Self::new(session_id, EventType::SessionCreated, data)
    }

    pub fn session_started(session_id: Uuid) -> Self {
        Self::new(session_id, EventType::SessionStarted, serde_json::json!({}))
    }

    pub fn session_paused(session_id: Uuid) -> Self {
        Self::new(session_id, EventType::SessionPaused, serde_json::json!({}))
    }

    pub fn session_resumed(session_id: Uuid) -> Self {
        Self::new(session_id, EventType::SessionResumed, serde_json::json!({}))
    }

    pub fn session_completed(session_id: Uuid) -> Self {
        Self::new(
            session_id,
            EventType::SessionCompleted,
            serde_json::json!({}),
        )
    }

    pub fn pick_made(
        session_id: Uuid,
        pick_id: Uuid,
        team_id: Uuid,
        player_id: Uuid,
        round: i32,
        pick_number: i32,
    ) -> Self {
        let data = serde_json::json!({
            "pick_id": pick_id,
            "team_id": team_id,
            "player_id": player_id,
            "round": round,
            "pick_number": pick_number,
        });
        Self::new(session_id, EventType::PickMade, data)
    }

    pub fn clock_update(session_id: Uuid, time_remaining: i32) -> Self {
        let data = serde_json::json!({
            "time_remaining": time_remaining,
        });
        Self::new(session_id, EventType::ClockUpdate, data)
    }

    pub fn trade_proposed(
        session_id: Uuid,
        trade_id: Uuid,
        from_team: Uuid,
        to_team: Uuid,
    ) -> Self {
        let data = serde_json::json!({
            "trade_id": trade_id,
            "from_team": from_team,
            "to_team": to_team,
        });
        Self::new(session_id, EventType::TradeProposed, data)
    }

    pub fn trade_executed(session_id: Uuid, trade_id: Uuid) -> Self {
        let data = serde_json::json!({
            "trade_id": trade_id,
        });
        Self::new(session_id, EventType::TradeExecuted, data)
    }

    pub fn trade_rejected(session_id: Uuid, trade_id: Uuid, rejecting_team_id: Uuid) -> Self {
        let data = serde_json::json!({
            "trade_id": trade_id,
            "rejecting_team_id": rejecting_team_id,
        });
        Self::new(session_id, EventType::TradeRejected, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::SessionCreated.to_string(), "SessionCreated");
        assert_eq!(EventType::PickMade.to_string(), "PickMade");
        assert_eq!(EventType::ClockUpdate.to_string(), "ClockUpdate");
    }

    #[test]
    fn test_event_type_from_str() {
        use std::str::FromStr;

        assert_eq!(
            EventType::from_str("SessionCreated").unwrap(),
            EventType::SessionCreated
        );
        assert_eq!(
            EventType::from_str("PickMade").unwrap(),
            EventType::PickMade
        );
        assert!(EventType::from_str("Invalid").is_err());
    }

    #[test]
    fn test_create_session_created_event() {
        let session_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let settings = serde_json::json!({"time_per_pick": 300});

        let event = DraftEvent::session_created(session_id, draft_id, settings);

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.event_type, EventType::SessionCreated);
        assert!(event.event_data["draft_id"].is_string());
        assert_eq!(event.event_data["settings"]["time_per_pick"], 300);
    }

    #[test]
    fn test_create_pick_made_event() {
        let session_id = Uuid::new_v4();
        let pick_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();

        let event = DraftEvent::pick_made(session_id, pick_id, team_id, player_id, 1, 1);

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.event_type, EventType::PickMade);
        assert_eq!(event.event_data["round"], 1);
        assert_eq!(event.event_data["pick_number"], 1);
    }

    #[test]
    fn test_create_clock_update_event() {
        let session_id = Uuid::new_v4();
        let event = DraftEvent::clock_update(session_id, 120);

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.event_type, EventType::ClockUpdate);
        assert_eq!(event.event_data["time_remaining"], 120);
    }

    #[test]
    fn test_lifecycle_events() {
        let session_id = Uuid::new_v4();

        let started = DraftEvent::session_started(session_id);
        assert_eq!(started.event_type, EventType::SessionStarted);

        let paused = DraftEvent::session_paused(session_id);
        assert_eq!(paused.event_type, EventType::SessionPaused);

        let resumed = DraftEvent::session_resumed(session_id);
        assert_eq!(resumed.event_type, EventType::SessionResumed);

        let completed = DraftEvent::session_completed(session_id);
        assert_eq!(completed.event_type, EventType::SessionCompleted);
    }
}
