-- Pick trades: Trade proposals between teams
CREATE TABLE pick_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES draft_sessions(id) ON DELETE CASCADE,
    from_team_id UUID NOT NULL REFERENCES teams(id) ON DELETE RESTRICT,
    to_team_id UUID NOT NULL REFERENCES teams(id) ON DELETE RESTRICT,
    status VARCHAR(20) NOT NULL DEFAULT 'Proposed',
    from_team_value INTEGER NOT NULL,  -- Total trade value for from_team's picks
    to_team_value INTEGER NOT NULL,    -- Total trade value for to_team's picks
    value_difference INTEGER NOT NULL, -- Absolute difference in trade values
    proposed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    responded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pick_trades_status_check CHECK (status IN ('Proposed', 'Accepted', 'Rejected')),
    CONSTRAINT pick_trades_different_teams CHECK (from_team_id != to_team_id)
);

-- Pick trade details: Individual picks in the trade
CREATE TABLE pick_trade_details (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id UUID NOT NULL REFERENCES pick_trades(id) ON DELETE CASCADE,
    pick_id UUID NOT NULL REFERENCES draft_picks(id) ON DELETE RESTRICT,
    direction VARCHAR(20) NOT NULL,
    pick_value INTEGER NOT NULL,  -- Individual pick value from chart
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pick_trade_details_direction_check CHECK (direction IN ('FromTeam', 'ToTeam')),
    CONSTRAINT pick_trade_details_unique_pick UNIQUE (trade_id, pick_id)
);

-- Indexes for common queries
CREATE INDEX idx_pick_trades_session_id ON pick_trades(session_id);
CREATE INDEX idx_pick_trades_from_team ON pick_trades(from_team_id);
CREATE INDEX idx_pick_trades_to_team ON pick_trades(to_team_id);
CREATE INDEX idx_pick_trades_status ON pick_trades(status);
CREATE INDEX idx_pick_trades_pending_to_team ON pick_trades(to_team_id, status) WHERE status = 'Proposed';
CREATE INDEX idx_pick_trade_details_trade_id ON pick_trade_details(trade_id);
CREATE INDEX idx_pick_trade_details_pick_id ON pick_trade_details(pick_id);

-- Auto-update timestamp trigger
CREATE OR REPLACE FUNCTION update_pick_trades_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER pick_trades_updated_at
    BEFORE UPDATE ON pick_trades
    FOR EACH ROW
    EXECUTE FUNCTION update_pick_trades_updated_at();
