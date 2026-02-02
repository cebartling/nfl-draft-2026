-- Create players table
CREATE TABLE players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    position VARCHAR(10) NOT NULL,
    college VARCHAR(100),
    height_inches INTEGER CHECK (height_inches IS NULL OR (height_inches >= 60 AND height_inches <= 90)),
    weight_pounds INTEGER CHECK (weight_pounds IS NULL OR (weight_pounds >= 150 AND weight_pounds <= 400)),
    draft_year INTEGER NOT NULL CHECK (draft_year >= 1936 AND draft_year <= 2100),
    draft_eligible BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_position CHECK (
        position IN ('QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C', 'DE', 'DT', 'LB', 'CB', 'S', 'K', 'P')
    )
);

-- Create indexes for common queries
CREATE INDEX idx_players_position ON players(position);
CREATE INDEX idx_players_draft_year ON players(draft_year);
CREATE INDEX idx_players_draft_eligible ON players(draft_eligible, draft_year) WHERE draft_eligible = true;
CREATE INDEX idx_players_name ON players(last_name, first_name);

-- Add comment
COMMENT ON TABLE players IS 'NFL Draft eligible players with their physical attributes and college information';
