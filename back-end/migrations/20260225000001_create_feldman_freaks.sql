CREATE TABLE feldman_freaks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    year INTEGER NOT NULL CHECK (year >= 2020 AND year <= 2030),
    rank INTEGER NOT NULL CHECK (rank > 0),
    description TEXT NOT NULL,
    article_url VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (player_id, year)
);

CREATE INDEX idx_feldman_freaks_player ON feldman_freaks(player_id);
CREATE INDEX idx_feldman_freaks_year_rank ON feldman_freaks(year, rank);
