-- Add TradeRejected to the allowed event types in draft_events table
ALTER TABLE draft_events DROP CONSTRAINT IF EXISTS draft_events_type_check;

ALTER TABLE draft_events ADD CONSTRAINT draft_events_type_check CHECK (event_type IN (
    'SessionCreated',
    'SessionStarted',
    'SessionPaused',
    'SessionResumed',
    'SessionCompleted',
    'PickMade',
    'ClockUpdate',
    'TradeProposed',
    'TradeExecuted',
    'TradeRejected'
));
