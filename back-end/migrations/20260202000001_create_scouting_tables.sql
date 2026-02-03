-- Create combine_results table for player performance measurements
CREATE TABLE combine_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    forty_yard_dash DECIMAL(4,2),
    bench_press INTEGER,
    vertical_jump DECIMAL(4,1),
    broad_jump INTEGER,
    three_cone_drill DECIMAL(4,2),
    twenty_yard_shuttle DECIMAL(4,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One combine result per player per year
    CONSTRAINT unique_player_combine UNIQUE (player_id, year),

    -- Validation checks
    CONSTRAINT valid_year CHECK (year >= 2000 AND year <= 2100),
    CONSTRAINT valid_forty_dash CHECK (forty_yard_dash IS NULL OR (forty_yard_dash >= 4.0 AND forty_yard_dash <= 6.0)),
    CONSTRAINT valid_bench CHECK (bench_press IS NULL OR (bench_press >= 0 AND bench_press <= 50)),
    CONSTRAINT valid_vertical CHECK (vertical_jump IS NULL OR (vertical_jump >= 20 AND vertical_jump <= 50)),
    CONSTRAINT valid_broad CHECK (broad_jump IS NULL OR (broad_jump >= 80 AND broad_jump <= 150)),
    CONSTRAINT valid_three_cone CHECK (three_cone_drill IS NULL OR (three_cone_drill >= 6.0 AND three_cone_drill <= 9.0)),
    CONSTRAINT valid_shuttle CHECK (twenty_yard_shuttle IS NULL OR (twenty_yard_shuttle >= 3.5 AND twenty_yard_shuttle <= 6.0))
);

-- Create scouting_reports table for team-specific player evaluations
CREATE TABLE scouting_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    grade DECIMAL(3,1) NOT NULL,
    notes TEXT,
    fit_grade VARCHAR(1),
    injury_concern BOOLEAN NOT NULL DEFAULT FALSE,
    character_concern BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One report per team per player
    CONSTRAINT unique_team_player_report UNIQUE (team_id, player_id),

    -- Grade validation
    CONSTRAINT valid_grade CHECK (grade >= 0.0 AND grade <= 10.0),
    CONSTRAINT valid_fit_grade CHECK (fit_grade IS NULL OR fit_grade IN ('A', 'B', 'C', 'D', 'F'))
);

-- Create team_needs table for position priorities
CREATE TABLE team_needs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    position VARCHAR(10) NOT NULL,
    priority INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One priority per team per position
    CONSTRAINT unique_team_position UNIQUE (team_id, position),

    -- Priority validation (1-10)
    CONSTRAINT valid_priority CHECK (priority >= 1 AND priority <= 10),

    -- Position validation (same as players table)
    CONSTRAINT valid_position CHECK (
        position IN ('QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C', 'DE', 'DT', 'LB', 'CB', 'S', 'K', 'P')
    )
);

-- Create indexes for common queries
CREATE INDEX idx_combine_player ON combine_results(player_id);
CREATE INDEX idx_combine_year ON combine_results(year);

CREATE INDEX idx_scouting_player ON scouting_reports(player_id);
CREATE INDEX idx_scouting_team ON scouting_reports(team_id);
CREATE INDEX idx_scouting_grade ON scouting_reports(grade DESC);

CREATE INDEX idx_team_needs_team ON team_needs(team_id);
CREATE INDEX idx_team_needs_priority ON team_needs(team_id, priority);
CREATE INDEX idx_team_needs_position ON team_needs(position);

-- Trigger to update updated_at timestamp for combine_results
CREATE OR REPLACE FUNCTION update_combine_results_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER combine_results_updated_at
    BEFORE UPDATE ON combine_results
    FOR EACH ROW
    EXECUTE FUNCTION update_combine_results_updated_at();

-- Trigger to update updated_at timestamp for scouting_reports
CREATE OR REPLACE FUNCTION update_scouting_reports_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER scouting_reports_updated_at
    BEFORE UPDATE ON scouting_reports
    FOR EACH ROW
    EXECUTE FUNCTION update_scouting_reports_updated_at();

-- Trigger to update updated_at timestamp for team_needs
CREATE OR REPLACE FUNCTION update_team_needs_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER team_needs_updated_at
    BEFORE UPDATE ON team_needs
    FOR EACH ROW
    EXECUTE FUNCTION update_team_needs_updated_at();

-- Add comments
COMMENT ON TABLE combine_results IS 'NFL Combine performance measurements for draft-eligible players';
COMMENT ON TABLE scouting_reports IS 'Team-specific player evaluations with grades and scouting notes';
COMMENT ON TABLE team_needs IS 'Position priorities for each team to guide draft decisions';
