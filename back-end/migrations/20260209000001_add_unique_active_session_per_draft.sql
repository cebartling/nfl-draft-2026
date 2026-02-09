-- Prevent multiple active sessions for the same draft
CREATE UNIQUE INDEX idx_one_active_session_per_draft
ON draft_sessions(draft_id)
WHERE status IN ('NotStarted', 'InProgress', 'Paused');
