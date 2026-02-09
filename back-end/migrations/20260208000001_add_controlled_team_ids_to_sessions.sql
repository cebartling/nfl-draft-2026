ALTER TABLE draft_sessions
ADD COLUMN controlled_team_ids UUID[] NOT NULL DEFAULT '{}';
