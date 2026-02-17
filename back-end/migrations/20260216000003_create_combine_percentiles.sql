-- Create combine_percentiles table for storing historical percentile breakpoints
-- Used by the RAS (Relative Athletic Score) engine to score measurements by position

CREATE TABLE combine_percentiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position VARCHAR(10) NOT NULL,
    measurement VARCHAR(30) NOT NULL,
    sample_size INTEGER NOT NULL DEFAULT 0,
    min_value DOUBLE PRECISION NOT NULL,
    p10 DOUBLE PRECISION NOT NULL,
    p20 DOUBLE PRECISION NOT NULL,
    p30 DOUBLE PRECISION NOT NULL,
    p40 DOUBLE PRECISION NOT NULL,
    p50 DOUBLE PRECISION NOT NULL,
    p60 DOUBLE PRECISION NOT NULL,
    p70 DOUBLE PRECISION NOT NULL,
    p80 DOUBLE PRECISION NOT NULL,
    p90 DOUBLE PRECISION NOT NULL,
    max_value DOUBLE PRECISION NOT NULL,
    years_start INTEGER NOT NULL DEFAULT 2000,
    years_end INTEGER NOT NULL DEFAULT 2025,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_position_measurement UNIQUE (position, measurement),
    CONSTRAINT valid_position CHECK (position IN (
        'QB', 'RB', 'WR', 'TE', 'OT', 'IOL', 'EDGE', 'DL', 'LB', 'CB', 'S', 'K', 'P'
    )),
    CONSTRAINT valid_measurement CHECK (measurement IN (
        'forty_yard_dash', 'bench_press', 'vertical_jump', 'broad_jump',
        'three_cone_drill', 'twenty_yard_shuttle', 'arm_length', 'hand_size',
        'wingspan', 'ten_yard_split', 'twenty_yard_split', 'height', 'weight'
    )),
    CONSTRAINT valid_sample_size CHECK (sample_size >= 0),
    CONSTRAINT valid_years CHECK (years_start <= years_end)
);

-- Trigger function for updated_at
CREATE OR REPLACE FUNCTION update_combine_percentiles_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_combine_percentiles_updated_at
    BEFORE UPDATE ON combine_percentiles
    FOR EACH ROW
    EXECUTE FUNCTION update_combine_percentiles_updated_at();

-- Index for fast position lookups
CREATE INDEX idx_combine_percentiles_position ON combine_percentiles(position);
