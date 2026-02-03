-- Create draft_strategies table for AI draft engine
CREATE TABLE draft_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    draft_id UUID NOT NULL REFERENCES drafts(id) ON DELETE CASCADE,
    bpa_weight INTEGER NOT NULL DEFAULT 60,
    need_weight INTEGER NOT NULL DEFAULT 40,
    position_values JSONB,
    risk_tolerance INTEGER NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_team_draft_strategy UNIQUE (team_id, draft_id),
    CONSTRAINT valid_weights CHECK (bpa_weight >= 0 AND bpa_weight <= 100 AND need_weight >= 0 AND need_weight <= 100),
    CONSTRAINT weights_sum_100 CHECK (bpa_weight + need_weight = 100),
    CONSTRAINT valid_risk_tolerance CHECK (risk_tolerance >= 0 AND risk_tolerance <= 10)
);

CREATE INDEX idx_draft_strategies_team ON draft_strategies(team_id);
CREATE INDEX idx_draft_strategies_draft ON draft_strategies(draft_id);
