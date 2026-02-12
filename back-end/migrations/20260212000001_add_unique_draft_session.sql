-- Add UNIQUE constraint on draft_sessions.draft_id to prevent duplicate sessions per draft.
-- The application-level check is racy (check-then-create); this enforces atomically at the DB.
ALTER TABLE draft_sessions ADD CONSTRAINT draft_sessions_draft_id_unique UNIQUE (draft_id);
