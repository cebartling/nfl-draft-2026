-- Multi-source big board rankings tables
-- Stores rankings from external sources (Tankathon, Walter Football, etc.)

CREATE TABLE ranking_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    url VARCHAR(500),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE prospect_rankings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ranking_source_id UUID NOT NULL REFERENCES ranking_sources(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    rank INTEGER NOT NULL CHECK (rank > 0),
    scraped_at DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (ranking_source_id, player_id)
);

CREATE INDEX idx_prospect_rankings_player ON prospect_rankings(player_id);
CREATE INDEX idx_prospect_rankings_source ON prospect_rankings(ranking_source_id);
CREATE INDEX idx_prospect_rankings_rank ON prospect_rankings(ranking_source_id, rank);
