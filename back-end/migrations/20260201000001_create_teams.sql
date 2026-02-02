-- Create teams table
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    abbreviation VARCHAR(5) NOT NULL UNIQUE,
    city VARCHAR(100) NOT NULL,
    conference VARCHAR(3) NOT NULL CHECK (conference IN ('AFC', 'NFC')),
    division VARCHAR(10) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_afc_division CHECK (
        conference != 'AFC' OR division IN ('AFC East', 'AFC North', 'AFC South', 'AFC West')
    ),
    CONSTRAINT valid_nfc_division CHECK (
        conference != 'NFC' OR division IN ('NFC East', 'NFC North', 'NFC South', 'NFC West')
    )
);

-- Create indexes for common queries
CREATE INDEX idx_teams_conference ON teams(conference);
CREATE INDEX idx_teams_division ON teams(division);
CREATE INDEX idx_teams_abbreviation ON teams(abbreviation);

-- Add comment
COMMENT ON TABLE teams IS 'NFL teams with their conference and division information';
