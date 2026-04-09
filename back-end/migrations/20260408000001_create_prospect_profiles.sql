CREATE TABLE prospect_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    source VARCHAR(50) NOT NULL,
    grade_tier VARCHAR(20),
    overall_rank INTEGER CHECK (overall_rank IS NULL OR overall_rank > 0),
    position_rank INTEGER NOT NULL CHECK (position_rank > 0),
    year_class VARCHAR(8),
    birthday DATE,
    jersey_number VARCHAR(8),
    height_raw VARCHAR(8),
    nfl_comparison TEXT,
    background TEXT,
    summary TEXT,
    strengths JSONB NOT NULL DEFAULT '[]'::jsonb,
    weaknesses JSONB NOT NULL DEFAULT '[]'::jsonb,
    college_stats JSONB,
    scraped_at DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (player_id, source)
);

CREATE INDEX idx_prospect_profiles_player ON prospect_profiles(player_id);
CREATE INDEX idx_prospect_profiles_source ON prospect_profiles(source);
CREATE INDEX idx_prospect_profiles_overall_rank ON prospect_profiles(overall_rank) WHERE overall_rank IS NOT NULL;
