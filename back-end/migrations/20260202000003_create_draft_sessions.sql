-- Draft Sessions: Real-time draft room management
CREATE TABLE draft_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    draft_id UUID NOT NULL REFERENCES drafts(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'NotStarted',
    current_pick_number INT NOT NULL DEFAULT 1,
    time_per_pick_seconds INT NOT NULL DEFAULT 300,
    auto_pick_enabled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    CONSTRAINT draft_sessions_status_check CHECK (status IN ('NotStarted', 'InProgress', 'Paused', 'Completed')),
    CONSTRAINT draft_sessions_current_pick_positive CHECK (current_pick_number > 0),
    CONSTRAINT draft_sessions_time_per_pick_positive CHECK (time_per_pick_seconds > 0)
);

-- Index for querying sessions by draft
CREATE INDEX idx_draft_sessions_draft_id ON draft_sessions(draft_id);

-- Index for querying active sessions
CREATE INDEX idx_draft_sessions_status ON draft_sessions(status);

-- Trigger to update updated_at timestamp for draft_sessions
CREATE OR REPLACE FUNCTION update_draft_sessions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER draft_sessions_updated_at
    BEFORE UPDATE ON draft_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_draft_sessions_updated_at();

-- Draft Events: Event sourcing for complete audit trail
CREATE TABLE draft_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES draft_sessions(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT draft_events_type_check CHECK (event_type IN (
        'SessionCreated',
        'SessionStarted',
        'SessionPaused',
        'SessionResumed',
        'SessionCompleted',
        'PickMade',
        'ClockUpdate',
        'TradeProposed',
        'TradeExecuted'
    ))
);

-- Index for querying events by session (for replay)
CREATE INDEX idx_draft_events_session_id ON draft_events(session_id);

-- Index for querying events by type
CREATE INDEX idx_draft_events_type ON draft_events(event_type);

-- Index for chronological ordering
CREATE INDEX idx_draft_events_created_at ON draft_events(created_at);

-- Composite index for efficient session event queries
CREATE INDEX idx_draft_events_session_created ON draft_events(session_id, created_at);
