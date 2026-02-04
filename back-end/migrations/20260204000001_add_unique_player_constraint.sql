-- Add unique constraint on players to prevent duplicate entries
-- A player is uniquely identified by their name and draft year
CREATE UNIQUE INDEX idx_unique_player_name_year
    ON players(first_name, last_name, draft_year);

COMMENT ON INDEX idx_unique_player_name_year IS 'Ensures no duplicate players with same name in the same draft year';
