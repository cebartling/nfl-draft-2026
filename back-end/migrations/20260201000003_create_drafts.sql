-- Create drafts table for draft year and status tracking
CREATE TABLE drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    year INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL CHECK (status IN ('NotStarted', 'InProgress', 'Paused', 'Completed')),
    rounds INTEGER NOT NULL DEFAULT 7,
    picks_per_round INTEGER NOT NULL DEFAULT 32,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure unique draft per year
    CONSTRAINT unique_draft_year UNIQUE (year)
);

-- Create draft_picks table for draft order and picks
CREATE TABLE draft_picks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    draft_id UUID NOT NULL REFERENCES drafts(id) ON DELETE CASCADE,
    round INTEGER NOT NULL CHECK (round > 0),
    pick_number INTEGER NOT NULL CHECK (pick_number > 0),
    overall_pick INTEGER NOT NULL CHECK (overall_pick > 0),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE RESTRICT,
    player_id UUID REFERENCES players(id) ON DELETE RESTRICT,
    picked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure unique pick per draft
    CONSTRAINT unique_draft_pick UNIQUE (draft_id, overall_pick),
    CONSTRAINT unique_draft_round_pick UNIQUE (draft_id, round, pick_number)
);

-- Create indexes for common queries
CREATE INDEX idx_draft_picks_draft_id ON draft_picks(draft_id);
CREATE INDEX idx_draft_picks_team_id ON draft_picks(team_id);
CREATE INDEX idx_draft_picks_player_id ON draft_picks(player_id);
CREATE INDEX idx_draft_picks_round ON draft_picks(draft_id, round);
CREATE INDEX idx_drafts_year ON drafts(year);
CREATE INDEX idx_drafts_status ON drafts(status);

-- Partial unique index to ensure player can only be picked once per draft (when player_id is not null)
CREATE UNIQUE INDEX idx_unique_player_per_draft ON draft_picks(draft_id, player_id) WHERE player_id IS NOT NULL;

-- Trigger to update updated_at timestamp for drafts
CREATE OR REPLACE FUNCTION update_drafts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER drafts_updated_at
    BEFORE UPDATE ON drafts
    FOR EACH ROW
    EXECUTE FUNCTION update_drafts_updated_at();

-- Trigger to update updated_at timestamp for draft_picks
CREATE OR REPLACE FUNCTION update_draft_picks_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER draft_picks_updated_at
    BEFORE UPDATE ON draft_picks
    FOR EACH ROW
    EXECUTE FUNCTION update_draft_picks_updated_at();
