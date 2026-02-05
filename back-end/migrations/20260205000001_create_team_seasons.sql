-- Create team_seasons table for tracking team records and draft positions
CREATE TABLE team_seasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    season_year INTEGER NOT NULL,
    wins INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    ties INTEGER NOT NULL DEFAULT 0,
    playoff_result VARCHAR(50),
    draft_position INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One record per team per season
    CONSTRAINT unique_team_season UNIQUE (team_id, season_year),

    -- Validation checks
    CONSTRAINT valid_season_year CHECK (season_year >= 1920 AND season_year <= 2100),
    CONSTRAINT valid_wins CHECK (wins >= 0 AND wins <= 17),
    CONSTRAINT valid_losses CHECK (losses >= 0 AND losses <= 17),
    CONSTRAINT valid_ties CHECK (ties >= 0 AND ties <= 17),
    CONSTRAINT valid_games_played CHECK (wins + losses + ties <= 17),
    CONSTRAINT valid_draft_position CHECK (draft_position IS NULL OR (draft_position >= 1 AND draft_position <= 32)),
    CONSTRAINT valid_playoff_result CHECK (
        playoff_result IS NULL OR playoff_result IN (
            'MissedPlayoffs', 'WildCard', 'Divisional', 'Conference', 'SuperBowlLoss', 'SuperBowlWin'
        )
    )
);

-- Create indexes for common queries
CREATE INDEX idx_team_seasons_team ON team_seasons(team_id);
CREATE INDEX idx_team_seasons_year ON team_seasons(season_year);
CREATE INDEX idx_team_seasons_draft_position ON team_seasons(season_year, draft_position);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_team_seasons_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER team_seasons_updated_at
    BEFORE UPDATE ON team_seasons
    FOR EACH ROW
    EXECUTE FUNCTION update_team_seasons_updated_at();

-- Add comment
COMMENT ON TABLE team_seasons IS 'Team season records including wins, losses, ties, playoff results, and draft positions';
