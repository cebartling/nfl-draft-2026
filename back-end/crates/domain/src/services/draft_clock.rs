use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, Interval};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::errors::DomainResult;

/// Represents the state of a draft clock
#[derive(Debug, Clone)]
pub struct ClockState {
    pub session_id: Uuid,
    pub time_remaining: i32,
    pub is_running: bool,
    pub current_pick_number: i32,
}

/// Draft clock that counts down for each pick
pub struct DraftClock {
    state: Arc<RwLock<ClockState>>,
}

impl DraftClock {
    /// Create a new draft clock
    pub fn new(session_id: Uuid, time_per_pick: i32, current_pick_number: i32) -> Self {
        let state = ClockState {
            session_id,
            time_remaining: time_per_pick,
            is_running: false,
            current_pick_number,
        };

        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// Start the clock countdown
    pub async fn start(&self) {
        let mut state = self.state.write().await;
        state.is_running = true;
        info!(
            session_id = %state.session_id,
            time_remaining = state.time_remaining,
            "Draft clock started"
        );
    }

    /// Pause the clock
    pub async fn pause(&self) {
        let mut state = self.state.write().await;
        state.is_running = false;
        info!(
            session_id = %state.session_id,
            time_remaining = state.time_remaining,
            "Draft clock paused"
        );
    }

    /// Reset the clock for a new pick
    pub async fn reset(&self, time_per_pick: i32, pick_number: i32) {
        let mut state = self.state.write().await;
        state.time_remaining = time_per_pick;
        state.current_pick_number = pick_number;
        state.is_running = true;
        info!(
            session_id = %state.session_id,
            pick_number = pick_number,
            time_remaining = time_per_pick,
            "Draft clock reset for new pick"
        );
    }

    /// Tick the clock (decrease by 1 second)
    /// Returns true if time expired
    pub async fn tick(&self) -> bool {
        let mut state = self.state.write().await;

        if !state.is_running {
            return false;
        }

        if state.time_remaining > 0 {
            state.time_remaining -= 1;
            debug!(
                session_id = %state.session_id,
                time_remaining = state.time_remaining,
                "Clock tick"
            );
            false
        } else {
            state.is_running = false;
            warn!(
                session_id = %state.session_id,
                pick_number = state.current_pick_number,
                "Draft clock expired"
            );
            true
        }
    }

    /// Get the current clock state
    pub async fn get_state(&self) -> ClockState {
        self.state.read().await.clone()
    }

    /// Get time remaining
    pub async fn time_remaining(&self) -> i32 {
        self.state.read().await.time_remaining
    }

    /// Check if clock is running
    pub async fn is_running(&self) -> bool {
        self.state.read().await.is_running
    }

    /// Check if clock has expired
    pub async fn is_expired(&self) -> bool {
        let state = self.state.read().await;
        state.time_remaining <= 0 && !state.is_running
    }

    /// Add time to the clock (for extensions)
    pub async fn add_time(&self, seconds: i32) -> DomainResult<()> {
        let mut state = self.state.write().await;
        state.time_remaining += seconds;
        info!(
            session_id = %state.session_id,
            added_seconds = seconds,
            new_time_remaining = state.time_remaining,
            "Time added to draft clock"
        );
        Ok(())
    }

    /// Set time directly (for testing or manual adjustments)
    pub async fn set_time(&self, seconds: i32) {
        let mut state = self.state.write().await;
        state.time_remaining = seconds;
        debug!(
            session_id = %state.session_id,
            time_remaining = seconds,
            "Clock time set directly"
        );
    }
}

/// Clock manager that handles tick intervals
pub struct ClockManager {
    clock: Arc<DraftClock>,
    interval: Interval,
}

impl ClockManager {
    /// Create a new clock manager with 1-second tick interval
    pub fn new(clock: Arc<DraftClock>) -> Self {
        let mut int = interval(Duration::from_secs(1));
        // Skip the first immediate tick
        int.reset();

        Self {
            clock,
            interval: int,
        }
    }

    /// Run the clock manager (call this in a spawned task)
    /// Callback is called on each tick with (session_id, time_remaining, expired)
    pub async fn run<F>(&mut self, mut on_tick: F)
    where
        F: FnMut(Uuid, i32, bool) + Send,
    {
        loop {
            self.interval.tick().await;

            let is_running = self.clock.is_running().await;
            if !is_running {
                continue;
            }

            let expired = self.clock.tick().await;
            let state = self.clock.get_state().await;

            on_tick(state.session_id, state.time_remaining, expired);

            // If expired, stop running
            if expired {
                break;
            }
        }
    }

    /// Run the clock manager with async callback
    pub async fn run_async<F, Fut>(&mut self, mut on_tick: F)
    where
        F: FnMut(Uuid, i32, bool) -> Fut + Send,
        Fut: std::future::Future<Output = ()> + Send,
    {
        loop {
            self.interval.tick().await;

            let is_running = self.clock.is_running().await;
            if !is_running {
                continue;
            }

            let expired = self.clock.tick().await;
            let state = self.clock.get_state().await;

            on_tick(state.session_id, state.time_remaining, expired).await;

            // If expired, stop running
            if expired {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_clock() {
        let session_id = Uuid::new_v4();
        let clock = DraftClock::new(session_id, 300, 1);

        let state = clock.get_state().await;
        assert_eq!(state.session_id, session_id);
        assert_eq!(state.time_remaining, 300);
        assert!(!state.is_running);
        assert_eq!(state.current_pick_number, 1);
    }

    #[tokio::test]
    async fn test_start_and_pause() {
        let clock = DraftClock::new(Uuid::new_v4(), 300, 1);

        // Initially not running
        assert!(!clock.is_running().await);

        // Start clock
        clock.start().await;
        assert!(clock.is_running().await);

        // Pause clock
        clock.pause().await;
        assert!(!clock.is_running().await);
    }

    #[tokio::test]
    async fn test_tick() {
        let clock = DraftClock::new(Uuid::new_v4(), 5, 1);
        clock.start().await;

        // Tick 5 times
        for i in 0..5 {
            let expired = clock.tick().await;
            assert!(!expired, "Clock should not expire on tick {}", i);
            assert_eq!(clock.time_remaining().await, 4 - i);
        }

        // Final tick should expire
        let expired = clock.tick().await;
        assert!(expired);
        assert_eq!(clock.time_remaining().await, 0);
        assert!(!clock.is_running().await);
    }

    #[tokio::test]
    async fn test_tick_when_paused() {
        let clock = DraftClock::new(Uuid::new_v4(), 10, 1);

        // Don't start the clock
        let expired = clock.tick().await;
        assert!(!expired);
        assert_eq!(clock.time_remaining().await, 10); // Should not change
    }

    #[tokio::test]
    async fn test_reset() {
        let clock = DraftClock::new(Uuid::new_v4(), 300, 1);
        clock.start().await;

        // Tick a few times
        clock.tick().await;
        clock.tick().await;
        clock.tick().await;

        // Reset for next pick
        clock.reset(300, 2).await;

        let state = clock.get_state().await;
        assert_eq!(state.time_remaining, 300);
        assert_eq!(state.current_pick_number, 2);
        assert!(state.is_running);
    }

    #[tokio::test]
    async fn test_add_time() {
        let clock = DraftClock::new(Uuid::new_v4(), 60, 1);

        // Add 30 seconds
        clock.add_time(30).await.unwrap();
        assert_eq!(clock.time_remaining().await, 90);

        // Can add negative to subtract
        clock.add_time(-10).await.unwrap();
        assert_eq!(clock.time_remaining().await, 80);
    }

    #[tokio::test]
    async fn test_set_time() {
        let clock = DraftClock::new(Uuid::new_v4(), 300, 1);

        clock.set_time(120).await;
        assert_eq!(clock.time_remaining().await, 120);
    }

    #[tokio::test]
    async fn test_is_expired() {
        let clock = DraftClock::new(Uuid::new_v4(), 2, 1);
        clock.start().await;

        assert!(!clock.is_expired().await);

        // Tick until expired
        clock.tick().await;
        clock.tick().await;
        clock.tick().await;

        assert!(clock.is_expired().await);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_clock_manager() {
        let session_id = Uuid::new_v4();
        let clock = Arc::new(DraftClock::new(session_id, 3, 1));
        clock.start().await;

        let mut manager = ClockManager::new(clock.clone());

        let mut tick_count = 0;
        let mut did_expire = false;

        // Run manager until expiration
        manager.run(|sid, time, expired| {
            assert_eq!(sid, session_id);
            tick_count += 1;

            // On the last tick, time will be 0 and expired will be true
            if expired {
                assert_eq!(time, 0);
                did_expire = true;
            }
        }).await;

        // Should have at least 3 ticks (one for each second)
        assert!(tick_count >= 3, "Expected at least 3 ticks, got {}", tick_count);
        assert!(did_expire);
        assert_eq!(clock.time_remaining().await, 0);
    }
}
