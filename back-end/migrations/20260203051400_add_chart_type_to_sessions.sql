-- Add chart_type column to draft_sessions table
-- This enables per-session trade value chart selection

ALTER TABLE draft_sessions
ADD COLUMN chart_type VARCHAR(50) NOT NULL DEFAULT 'JimmyJohnson';

-- Add comment for documentation
COMMENT ON COLUMN draft_sessions.chart_type IS
'Trade value chart type to use for this session. Options: JimmyJohnson, RichHill, ChaseStudartAV, FitzgeraldSpielberger, PffWar, SurplusValue';

-- Add check constraint for valid values
-- Note: This hardcodes chart types at the database level, creating coupling between
-- schema and code. However, this provides defense-in-depth data validation and is
-- acceptable since chart types are infrequent additions. Adding a new chart type
-- requires both code changes and a migration, ensuring deliberate, coordinated updates.
ALTER TABLE draft_sessions
ADD CONSTRAINT valid_chart_type CHECK (
    chart_type IN (
        'JimmyJohnson',
        'RichHill',
        'ChaseStudartAV',
        'FitzgeraldSpielberger',
        'PffWar',
        'SurplusValue'
    )
);
