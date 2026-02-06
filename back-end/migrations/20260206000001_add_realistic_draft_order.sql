-- Add trade/compensatory metadata to draft_picks
ALTER TABLE draft_picks
    ADD COLUMN original_team_id UUID REFERENCES teams(id) ON DELETE RESTRICT,
    ADD COLUMN is_compensatory BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN notes TEXT;

CREATE INDEX idx_draft_picks_original_team_id ON draft_picks(original_team_id);

-- Make picks_per_round nullable (NULL = realistic draft with variable round sizes)
ALTER TABLE drafts ALTER COLUMN picks_per_round DROP NOT NULL;
ALTER TABLE drafts ALTER COLUMN picks_per_round DROP DEFAULT;
